//! Görsel doğrulama (ölçüm değil, insan gözü için): rol işaretli örnek arayüzü dört temada PNG'ye çizer.
//! `WGPU_BACKEND=vulkan KILIM_GORSEL_DIZIN=<dizin> cargo test --test gorsel -- --ignored`.

use egui_kittest::Harness;
use kilim_tema::{
    cerceve, dugme, giydir, rol_rengi, Aydinlik, Gorunum, Rol, RolVer, TemaTercihi, Varyant,
};

fn arayuz(ui: &mut egui::Ui, tercih: TemaTercihi) {
    cerceve(ui, &tercih, |ui| {
        egui::Panel::top("ust").rol(Rol::UstCubuk).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.heading("roller");
                ui.label("menü · araçlar · ara");
            });
        });
        egui::Panel::bottom("alt")
            .rol(Rol::DurumCubugu)
            .show(ui, |ui| {
                ui.label("durum çubuğu: küçük, soluk, tek satır; çok uzun bir ileti sığmazsa sonu kesilir ve çubuk yükselmez");
            });
        egui::Panel::left("sol")
            .resizable(true)
            .default_size(220.0)
            .rol(Rol::YanPanel)
            .show(ui, |ui| {
                for i in 0..8 {
                    let _ = ui.selectable_label(i == 2, format!("liste öğesi {i}"));
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
                ui.colored_label(rol_rengi(ui, Rol::Hata), "hata rengi");
                ui.colored_label(rol_rengi(ui, Rol::Uyari), "uyarı rengi");
                ui.colored_label(rol_rengi(ui, Rol::Basari), "başarı rengi");
                egui::Frame::group(ui.style())
                    .rol(Rol::Kart)
                    .show(ui, |ui| {
                        ui.label("kart: ince çizgili, yuvarlak köşeli çerçeve");
                    });
                ui.text_edit_singleline(&mut String::from("metin kutusu"));
            });
    });
}

#[test]
#[ignore = "görsel çıktı üretir; WGPU gerekir"]
fn roller_dort_temada_png() {
    let dizin = std::env::var("KILIM_GORSEL_DIZIN").expect("KILIM_GORSEL_DIZIN");
    for (ad, g, koyu) in [
        ("kilim-acik", Gorunum::Kilim, false),
        ("kilim-koyu", Gorunum::Kilim, true),
        ("sade-acik", Gorunum::Sade, false),
        ("sade-koyu", Gorunum::Sade, true),
    ] {
        let tercih = TemaTercihi {
            gorunum: g,
            varyant: Varyant::CamGobegiAltin,
            aydinlik: if koyu { Aydinlik::Koyu } else { Aydinlik::Acik },
        };
        let mut h = Harness::builder()
            .with_size(egui::vec2(900.0, 560.0))
            .with_theme(if koyu {
                egui::Theme::Dark
            } else {
                egui::Theme::Light
            })
            .build_ui(move |ui| arayuz(ui, tercih));
        giydir(&h.ctx, &tercih);
        h.run_steps(3);
        let resim = h.render().expect("çizim");
        resim.save(format!("{dizin}/{ad}.png")).expect("kayıt");
    }
}
