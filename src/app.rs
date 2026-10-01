use std::io::Cursor;

use crate::items::{Item, ItemSlot, fetch_items};
use eframe::egui;
use egui_async::{Bind, StateWithData, egui::AsyncButton};
use rfd::FileHandle;
use rlbuddy_upk_swapper::{RlAesKey, Upk};

#[derive(Debug, Default)]
struct ChoosingState {
    search_appearance: String,
    chosen_appearance: Option<Item>,
    search_replaced: String,
    chosen_replaced: Option<Item>,
    slot_filter: ItemSlot,
}

#[derive(Debug)]
enum AppStage {
    LoadingItems(Bind<Vec<Item>, anyhow::Error>),
    Choosing {
        all_items: Vec<Item>,
        state: ChoosingState,
    },
    PickingGameDir {
        chosen_replaced: Item,
        replaced_file_bind: Bind<Option<FileHandle>, ()>,
        chosen_appearance: Item,
        appearance_file_bind: Bind<Option<FileHandle>, ()>,
    },
    ReadingFiles {
        chosen_replaced: Item,
        replaced_file: FileHandle,
        replaced_file_contents_bind: Bind<Vec<u8>, ()>,
        chosen_appearance: Item,
        appearance_file: FileHandle,
        appearance_file_contents_bind: Bind<Vec<u8>, ()>,
    },
    Swapping {
        chosen_replaced: Item,
        replaced_file_contents: Vec<u8>,
        chosen_appearance: Item,
        appearance_file_contents: Vec<u8>,
    },
    Done {
        chosen_replaced: Item,
        chosen_appearance: Item,
        upk_download: Vec<u8>,
        write_to_file_handle_bind: Bind<Option<FileHandle>, ()>,
        writing_to_file_bind: Bind<(), std::io::Error>, // just for the executor
    },
    Error(String),
}

impl Default for AppStage {
    fn default() -> Self {
        Self::LoadingItems(Bind::new(true))
    }
}

impl AppStage {
    fn render_and_update(self, ui: &mut egui::Ui) -> Self {
        match self {
            Self::LoadingItems(mut bind) => {
                ui.spinner();
                ui.label("Loading items...");

                if let Some(result) = bind.read_or_request_or_error(fetch_items, ui) {
                    Self::Choosing {
                        all_items: result.clone(),
                        state: ChoosingState::default(),
                    }
                } else {
                    Self::LoadingItems(bind)
                }
            }
            Self::Choosing {
                all_items,
                mut state,
            } => {
                egui::ComboBox::from_label("Slot filter")
                    .selected_text(state.slot_filter.as_str())
                    .show_ui(ui, |ui| {
                        for slot in &[
                            ItemSlot::Antenna,
                            ItemSlot::Body,
                            ItemSlot::Boost,
                            ItemSlot::Decal,
                            ItemSlot::Explosion,
                            ItemSlot::PaintFinish,
                            ItemSlot::Topper,
                            ItemSlot::Trail,
                            ItemSlot::Wheel,
                        ] {
                            ui.selectable_value(&mut state.slot_filter, *slot, slot.as_str());
                        }
                    });

                ui.add_space(8.0);

                item_select(
                    ui,
                    "Replaced Item",
                    &mut state.search_replaced,
                    state.slot_filter,
                    &mut state.chosen_replaced,
                    &all_items,
                );
                item_select(
                    ui,
                    "Appearance Item",
                    &mut state.search_appearance,
                    state.slot_filter,
                    &mut state.chosen_appearance,
                    &all_items,
                );

                if ui
                    .add_enabled(
                        state.chosen_replaced.is_some() && state.chosen_appearance.is_some(),
                        egui::Button::new("OK"),
                    )
                    .clicked()
                {
                    let chosen_replaced = state.chosen_replaced.take().unwrap();
                    let chosen_appearance = state.chosen_appearance.take().unwrap();
                    Self::PickingGameDir {
                        chosen_replaced,
                        replaced_file_bind: Bind::default(),
                        chosen_appearance,
                        appearance_file_bind: Bind::default(),
                    }
                } else {
                    Self::Choosing { all_items, state }
                }
            }
            Self::PickingGameDir {
                chosen_replaced,
                mut replaced_file_bind,
                chosen_appearance,
                mut appearance_file_bind,
            } => {
                ui.horizontal(|ui| {
                    ui.label("Pick");
                    ui.strong(chosen_replaced.package.filename());
                    ui.label("from CookedPCConsole");
                });
                file_picker_button(
                    ui,
                    &mut replaced_file_bind,
                    Some(chosen_replaced.package.filename().as_str()),
                );

                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label("Pick");
                    ui.strong(chosen_appearance.package.filename());
                    ui.label("from CookedPCConsole");
                });
                file_picker_button(
                    ui,
                    &mut appearance_file_bind,
                    Some(chosen_appearance.package.filename().as_str()),
                );

                ui.add_space(12.0);
                // #1: finised
                // #2: successful
                // #3: exists
                // check first instead of using `take` so it doesnt take
                // the replaced bind if appearance is still None
                if let Some(Ok(Some(_))) = replaced_file_bind.read()
                    && let Some(Ok(Some(_))) = appearance_file_bind.read()
                {
                    let replaced_file = replaced_file_bind.take_ok().unwrap().unwrap();
                    let appearance_file = appearance_file_bind.take_ok().unwrap().unwrap();

                    Self::ReadingFiles {
                        chosen_replaced,
                        replaced_file,
                        replaced_file_contents_bind: Bind::default(),
                        chosen_appearance,
                        appearance_file,
                        appearance_file_contents_bind: Bind::default(),
                    }
                } else {
                    Self::PickingGameDir {
                        chosen_replaced,
                        replaced_file_bind,
                        chosen_appearance,
                        appearance_file_bind,
                    }
                }
            }
            Self::ReadingFiles {
                chosen_replaced,
                replaced_file,
                mut replaced_file_contents_bind,
                chosen_appearance,
                appearance_file,
                mut appearance_file_contents_bind,
            } => {
                let replaced_file_async_handle = replaced_file.clone();
                let replaced_file_contents =
                    replaced_file_contents_bind.read_or_request(async move || {
                        let content = replaced_file_async_handle.read().await;
                        Ok(content)
                    });

                let appearance_file_async_handle = appearance_file.clone();
                let appearance_file_contents =
                    appearance_file_contents_bind.read_or_request(async move || {
                        let content = appearance_file_async_handle.read().await;
                        Ok(content)
                    });

                if let Some(Ok(_)) = replaced_file_contents
                    && let Some(Ok(_)) = appearance_file_contents
                {
                    let replaced_file_contents = replaced_file_contents_bind.take_ok().unwrap();
                    let appearance_file_contents = appearance_file_contents_bind.take_ok().unwrap();
                    Self::Swapping {
                        chosen_replaced,
                        replaced_file_contents,
                        chosen_appearance,
                        appearance_file_contents,
                    }
                } else {
                    Self::ReadingFiles {
                        chosen_replaced,
                        replaced_file,
                        replaced_file_contents_bind,
                        chosen_appearance,
                        appearance_file,
                        appearance_file_contents_bind,
                    }
                }
            }
            Self::Swapping {
                chosen_replaced,
                replaced_file_contents,
                chosen_appearance,
                appearance_file_contents,
            } => {
                ui.label("Processing swap...");
                ui.label("This takes a few seconds and will probably freeze the page.");

                let replaced_key = RlAesKey::from_base64(&chosen_replaced.key).unwrap();
                let replaced_upk = match Upk::new(
                    &mut Cursor::new(replaced_file_contents),
                    &chosen_replaced.package.name(),
                    &replaced_key,
                ) {
                    Ok(u) => u,
                    Err(e) => {
                        return Self::Error(format!("{e:?}"));
                    }
                };

                let appearance_key = RlAesKey::from_base64(&chosen_appearance.key).unwrap();
                let mut appearance_upk = match Upk::new(
                    &mut Cursor::new(appearance_file_contents),
                    &chosen_appearance.package.name(),
                    &appearance_key,
                ) {
                    Ok(u) => u,
                    Err(e) => {
                        return Self::Error(format!("{e:?}"));
                    }
                };
                appearance_upk.pretend_to_be(&replaced_upk);

                let upk_download = match appearance_upk.serialize() {
                    Ok(s) => s,
                    Err(e) => {
                        return Self::Error(format!("{e:?}"));
                    }
                };

                Self::Done {
                    chosen_replaced,
                    chosen_appearance,
                    upk_download,
                    write_to_file_handle_bind: Bind::default(),
                    writing_to_file_bind: Bind::default(),
                }
            }
            Self::Done {
                chosen_replaced,
                chosen_appearance,
                mut upk_download,
                mut write_to_file_handle_bind,
                mut writing_to_file_bind,
            } => {
                ui.label("Swapped successfully! First,");

                {
                    let filename = chosen_replaced.package.filename();
                    AsyncButton::new(&mut write_to_file_handle_bind, "open the download prompt")
                        .show(ui, || async move {
                            Ok(rfd::AsyncFileDialog::new()
                                .set_file_name(filename)
                                .save_file()
                                .await)
                        });

                    if let Some(Some(handle)) = write_to_file_handle_bind.take_ok() {
                        let bytes = std::mem::take(&mut upk_download);
                        let handle = handle.clone();
                        writing_to_file_bind.request(async move { handle.write(&bytes).await });
                    }
                }

                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    ui.label("Once it's downloaded, go to the ");
                    ui.strong("CookedPCConsole");
                    ui.label("folder within your game files.");
                });
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    ui.label("Now, in the folder, find");
                    ui.strong(chosen_replaced.package.filename());
                    ui.label(", and rename it to");
                    ui.strong(chosen_replaced.package.backup_filename());
                    ui.label("(to back it up).");
                });
                ui.horizontal_wrapped(|ui| {
                    ui.label("Finally, move the file you downloaded,");
                    ui.strong(chosen_replaced.package.filename());
                    ui.label(", into the CookedPCConsole.");
                });
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    ui.label("Now, when Rocket League tries to load");
                    ui.strong(chosen_replaced.package.filename());
                    ui.label(", it actually loads the data in");
                    ui.strong(&chosen_appearance.name);
                    ui.label("! Pretty cool, right?");
                });

                Self::Done {
                    chosen_replaced,
                    chosen_appearance,
                    upk_download,
                    write_to_file_handle_bind,
                    writing_to_file_bind,
                }
            }
            Self::Error(err) => {
                ui.colored_label(ui.visuals().error_fg_color, &err);
                Self::Error(err)
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct UpkSwapperApp {
    stage: AppStage,
}

impl UpkSwapperApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        UpkSwapperApp::default()
    }
}

impl eframe::App for UpkSwapperApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.plugin_or_default::<egui_async::EguiAsyncPlugin>();
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("upk swapper");
            ui.separator();

            self.stage = std::mem::take(&mut self.stage).render_and_update(ui);
        });
    }
}

fn item_select(
    ui: &mut egui::Ui,
    text: &str,
    search: &mut String,
    filter: ItemSlot,
    chosen: &mut Option<Item>,
    items: &[Item],
) {
    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(search)
                .hint_text("Filter")
                .desired_width(125.0),
        );
        egui::ComboBox::from_label(text)
            .selected_text(chosen.as_ref().map(|a| a.name.as_str()).unwrap_or("None"))
            .show_ui(ui, |ui| {
                for item in items
                    .iter()
                    .filter(|i| i.slot == filter && i.name.to_lowercase().contains(search.as_str()))
                {
                    if ui
                        .selectable_label(
                            chosen.as_ref().map(|i| i.id) == Some(item.id),
                            item.name.as_str(),
                        )
                        .clicked()
                    {
                        chosen.replace(item.clone());
                    }
                }
            });
    });
}

fn file_picker_button(
    ui: &mut egui::Ui,
    bind: &mut Bind<Option<FileHandle>, ()>,
    filename_requirement: Option<&str>,
) {
    let mut should_clear = false;
    ui.horizontal(|ui| match bind.state() {
        StateWithData::Finished(Some(handle)) => {
            let filename = handle.file_name();

            if let Some(req) = filename_requirement
                && req != &filename
            {
                should_clear = true;
            } else if ui.button("Clear").clicked() {
                should_clear = true;
            }

            ui.label(filename);
        }
        _ => {
            AsyncButton::new(bind, "Choose file")
                .pending_text("")
                .show(ui, pick_file);
            ui.label("No file selected");
        }
    });

    if should_clear {
        bind.fill(Ok(None));
    }
}

async fn pick_file() -> Result<Option<FileHandle>, ()> {
    Ok(rfd::AsyncFileDialog::new()
        .add_filter("UPK File", &["upk"])
        .pick_file()
        .await)
}
