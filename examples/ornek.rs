//! `cargo run --release --example ornek` — temanın canlı önizlemesi.

use kilim_tema::{boncuk_dugme, cerceve, Boncuk, Varyant};

struct Ornek {
    varyant: Varyant,
    tiklama: u32,
}

impl eframe::App for Ornek {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let varyant = self.varyant;
        cerceve(ui, varyant, |ui| {
            egui::Panel::top("ust").show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("kilim-tema önizleme");
                    ui.separator();
                    ui.selectable_value(
                        &mut self.varyant,
                        Varyant::CamGobegiAltin,
                        "Cam göbeği + altın",
                    );
                    ui.selectable_value(
                        &mut self.varyant,
                        Varyant::KirmiziYesil,
                        "Kırmızı + yeşil",
                    );
                });
            });
            egui::Panel::left("sol").default_size(220.0).show(ui, |ui| {
                ui.heading("Uygulamalar");
                for ad in ["Paslı Beyin", "bge-embed-rs", "kervan", "el-fihrist"] {
                    ui.label(ad);
                }
            });
            egui::CentralPanel::default().show(ui, |ui| {
                ui.label("Semerkant kâğıdı zemin döşenir, lekeler gerilir; kenarlık birimi tam sayıda tekrar eder.");
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    for (b, s) in [(Boncuk::Firuze, "+"), (Boncuk::Akik, "×"), (Boncuk::Lapis, "?"), (Boncuk::Telkari, "="), (Boncuk::Sirma, "★"), (Boncuk::Tesbih, "⟳"), (Boncuk::Nazar, "")] {
                        if boncuk_dugme(ui, b, s, 40.0).clicked() {
                            self.tiklama += 1;
                        }
                    }
                    ui.add_enabled_ui(false, |ui| boncuk_dugme(ui, Boncuk::Kehribar, "!", 40.0));
                });
                ui.label(format!("Tıklama: {}", self.tiklama));
            });
        });
    }
}

fn main() -> eframe::Result {
    let secenek = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 640.0])
            .with_transparent(false)
            .with_title("kilim-tema önizleme"),
        ..Default::default()
    };
    eframe::run_native(
        "kilim-tema önizleme",
        secenek,
        Box::new(|cc| {
            kilim_tema::acik().uygula(&cc.egui_ctx, false);
            Ok(Box::new(Ornek {
                varyant: Varyant::CamGobegiAltin,
                tiklama: 0,
            }))
        }),
    )
}
