use crate::items::{Item, ItemSlot, fetch_items};
use eframe::egui;
use egui_async::{Bind, StateWithData, egui::AsyncButton};
use rfd::FileHandle;

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
}

impl Default for AppStage {
    fn default() -> Self {
        Self::LoadingItems(Bind::new(true))
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

            match &mut self.stage {
                AppStage::LoadingItems(bind) => {
                    ui.spinner();
                    ui.label("Loading items...");

                    if let Some(result) = bind.read_or_request_or_error(fetch_items, ui) {
                        self.stage = AppStage::Choosing {
                            all_items: result.clone(),
                            state: ChoosingState::default(),
                        }
                    }
                }
                AppStage::Choosing { all_items, state } => {
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
                        all_items,
                    );
                    item_select(
                        ui,
                        "Appearance Item",
                        &mut state.search_appearance,
                        state.slot_filter,
                        &mut state.chosen_appearance,
                        all_items,
                    );
                }
            }
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

fn file_picker_button(ui: &mut egui::Ui, bind: &mut Bind<Option<FileHandle>, ()>) {
    let mut should_clear = false;
    ui.horizontal(|ui| match bind.state() {
        StateWithData::Finished(Some(handle)) => {
            if ui.button("Clear").clicked() {
                should_clear = true;
            }
            ui.label(handle.file_name());
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
    Ok(rfd::AsyncFileDialog::new().pick_file().await)
}
