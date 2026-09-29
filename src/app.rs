use eframe::egui;

pub struct UpkSwapperApp {}

impl UpkSwapperApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        UpkSwapperApp {}
    }
}

impl eframe::App for UpkSwapperApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("upk swapper");
        });
    }
}
