use crate::items::{Item, fetch_items};
use eframe::egui;
use egui_async::{Bind, StateWithData, egui::AsyncButton};
use rfd::FileHandle;

#[derive(Debug, Default)]
struct AppState {
    appearance_upk: Bind<Option<FileHandle>, ()>,
    replacement_upk: Bind<Option<FileHandle>, ()>,
}

#[derive(Debug)]
enum AppStage {
    LoadingItems(Bind<Vec<Item>, anyhow::Error>),
    SelectingItems { all_items: Vec<Item> },
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
                        self.stage = AppStage::SelectingItems {
                            all_items: result.clone(),
                        }
                    }
                }
                AppStage::SelectingItems { all_items } => {
                    ui.label(format!("qty items: {}", all_items.len()));
                }
            }
        });
    }
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
