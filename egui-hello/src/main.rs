use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 200.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Klipper Touch",
        options,
        Box::new(|_cc| Ok(Box::<MyApp>::default())),
    )
}

#[derive(Default)]
struct MyApp {
    clicks: u32,
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Hello, egui!");
            if ui.button("Klick mich").clicked() {
                self.clicks += 1;
            }
            ui.label(format!("Klicks: {}", self.clicks));
        });
    }
}
