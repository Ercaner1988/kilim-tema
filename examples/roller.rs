//! `cargo run --example roller` — rol işaretçilerinin canlı önizlemesi. Hata ayıklama yapısında
//! `KILIM_ROL_IZ=1 cargo run --example roller` her bölgeyi rol adıyla çerçeveler (ADR 0001).
//! Araç tarafı gereken tek şey: açılışta `giydir`, bölgelerde `.rol(..)`.

use kilim_tema::{
    cerceve, dugme, giydir, rol_rengi, tema_secici, Rol, RolVer, TemaTercihi, Varyant,
};

const UYGULAMA: &str = "roller";

struct Roller {
    tercih: TemaTercihi,
    secili: usize,
}

impl eframe::App for Roller {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let tercih = self.tercih;
        cerceve(ui, &tercih, |ui| {
            egui::Panel::top("ust").rol(Rol::UstCubuk).show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.heading("roller");
                    if tema_secici(ui, &mut self.tercih) {
                        self.tercih.kaydet(UYGULAMA);
                    }
                });
            });
            egui::Panel::bottom("alt").rol(Rol::DurumCubugu).show(ui, |ui| {
                ui.label("durum çubuğu: küçük, soluk, tek satır; çok uzun bir ileti sığmazsa sonu \"…\" ile kesilir ve çubuk yükselmez");
            });
            egui::Panel::left("sol")
                .resizable(true)
                .default_size(240.0)
                .rol(Rol::YanPanel)
                .show(ui, |ui| {
                    for i in 0..12 {
                        if ui
                            .selectable_label(self.secili == i, format!("liste öğesi {i}"))
                            .clicked()
                        {
                            self.secili = i;
                        }
                    }
                });
            egui::CentralPanel::default()
                .rol(Rol::Zemin)
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        let _ = ui.add(dugme(ui, Rol::DugmeBirincil, "Kaydet"));
                        let _ = ui.add(dugme(ui, Rol::DugmeIkincil, "Vazgeç"));
                        let _ = ui.add(dugme(ui, Rol::DugmeTehlikeli, "Sil"));
                    });
                    ui.colored_label(
                        rol_rengi(ui, Rol::Hata),
                        "hata rengi (gerçek arka plana göre okunur)",
                    );
                    ui.colored_label(rol_rengi(ui, Rol::Uyari), "uyarı rengi");
                    ui.colored_label(rol_rengi(ui, Rol::Basari), "başarı rengi");
                    egui::Frame::group(ui.style())
                        .rol(Rol::Kart)
                        .show(ui, |ui| {
                            ui.label("kart: ince çizgili, yuvarlak köşeli çerçeve");
                        });
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
            .with_title("kilim-tema roller"),
        ..Default::default()
    };
    eframe::run_native(
        "kilim-tema roller",
        secenek,
        Box::new(|cc| {
            let tercih =
                TemaTercihi::yukle(UYGULAMA, TemaTercihi::varsayilan(Varyant::CamGobegiAltin));
            giydir(&cc.egui_ctx, &tercih);
            Ok(Box::new(Roller { tercih, secili: 0 }))
        }),
    )
}
