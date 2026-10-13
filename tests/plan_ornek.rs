//! Gerçek araçların bölge iskeletleri, işaretçiler konmuş hâliyle (ADR 0001 §Araç talepleri) çalışma zamanında
//! koşturulur ve kendiliğinden yerleşim planı basılır (`cargo test --test plan_ornek -- --nocapture`).
//! İskeletler kaynak dosyalardan bire bir türetilmiştir (dosya:satır yorumlarda); içerik sadeleştirilmiştir.

use egui::Ui;
use egui_kittest::Harness;
use kilim_tema::{Rol, RolVer};

/// kesfuzzunun-gui/src/lib.rs:1141-1440
fn kesf(ui: &mut Ui) {
    egui::Panel::top("ana_menu_cubugu")
        .exact_size(24.0)
        .rol(Rol::UstCubuk)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let _ = ui.button("Dosya");
            });
        });
    egui::Panel::top("arac_cubugu")
        .exact_size(41.0)
        .rol(Rol::UstCubuk)
        .show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                let _ = ui.button("Ara");
            });
        });
    egui::Panel::bottom("durum_cubugu")
        .exact_size(24.0)
        .rol(Rol::DurumCubugu)
        .show(ui, |ui| {
            ui.label("hazır");
        });
    egui::Panel::left("sol_koleksiyon_paneli")
        .resizable(true)
        .min_size(200.0)
        .rol(Rol::YanPanel)
        .show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| ui.label("koleksiyonlar"));
        });
    egui::Panel::right("sag_detay_paneli")
        .resizable(true)
        .min_size(320.0)
        .rol(Rol::YanPanel)
        .show(ui, |ui| {
            ui.label("detay");
        });
    egui::CentralPanel::default()
        .rol(Rol::Zemin)
        .show(ui, |ui| {
            ui.label("eser tablosu");
        });
    egui::Window::new("Koleksiyona ekle")
        .rol(Rol::Pencere)
        .show(ui.ctx(), |ui| {
            ui.label("seçin");
        });
}

/// pasli-beyin/pencere-egui/src/app.rs:158-512
fn pasli(ui: &mut Ui) {
    egui::Panel::top("ust_serit")
        .rol(Rol::UstCubuk)
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                let _ = ui.button("☰ araçlar");
            });
        });
    egui::Panel::top("gorunum")
        .rol(Rol::UstCubuk)
        .show(ui, |ui| {
            ui.label("görünüm ayarları");
        });
    egui::Panel::bottom("durum_cubugu")
        .rol(Rol::DurumCubugu)
        .show(ui, |ui| {
            ui.label("durum");
        });
    let mut acik = true;
    egui::Panel::left("cekmece")
        .resizable(true)
        .default_size(300.0)
        .rol(Rol::YanPanel)
        .show_collapsible(ui, &mut acik, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| ui.label("çekmece"));
        });
    egui::Panel::right("detay_panel")
        .resizable(true)
        .rol(Rol::YanPanel)
        .show(ui, |ui| {
            ui.label("detay");
        });
    egui::CentralPanel::default()
        .rol(Rol::Zemin)
        .show(ui, |ui| {
            ui.label("treemap");
        });
    egui::Window::new("YZ oturum izleri")
        .rol(Rol::Pencere)
        .show(ui.ctx(), |ui| {
            ui.label("izler");
        });
}

fn plan(f: fn(&mut Ui)) -> (String, Vec<kilim_tema::Kayit>) {
    let mut h = Harness::builder()
        .with_size(egui::vec2(1280.0, 800.0))
        .with_theme(egui::Theme::Light)
        .build_ui(f);
    h.run();
    (
        kilim_tema::plan_metni(&h.ctx),
        kilim_tema::rol_kayitlari(&h.ctx),
    )
}

fn roller(k: &[kilim_tema::Kayit]) -> Vec<Rol> {
    k.iter().map(|x| x.rol).collect()
}

#[test]
fn kesf_iskeleti_plani() {
    let (metin, k) = plan(kesf);
    println!("--- kesfuzzunun-gui (işaretli) ---\n{metin}");
    assert_eq!(
        roller(&k),
        [
            Rol::UstCubuk,
            Rol::UstCubuk,
            Rol::DurumCubugu,
            Rol::YanPanel,
            Rol::YanPanel,
            Rol::Zemin,
            Rol::Pencere
        ]
    );
    assert!(k.iter().all(|x| x.uyumlu), "{metin}");
    // İki yan panelde sabit_genislik kendiliğinden açık; çubuklarda ve zeminde kapalı.
    let sabit: Vec<Rol> = k
        .iter()
        .filter(|x| x.sabit_genislik)
        .map(|x| x.rol)
        .collect();
    assert_eq!(sabit, [Rol::YanPanel, Rol::YanPanel]);
}

#[test]
fn pasli_iskeleti_plani() {
    let (metin, k) = plan(pasli);
    println!("--- pasli-beyin pencere-egui (işaretli) ---\n{metin}");
    assert_eq!(
        roller(&k),
        [
            Rol::UstCubuk,
            Rol::UstCubuk,
            Rol::DurumCubugu,
            Rol::YanPanel,
            Rol::YanPanel,
            Rol::Zemin,
            Rol::Pencere
        ]
    );
    assert!(k.iter().all(|x| x.uyumlu), "{metin}");
}

/// nazar/crates/nazar-masaustu/src/arayuz.rs:152-170 (araç kendi çerçevesini ve ayırıcı çizgisini veriyor).
fn nazar(ui: &mut Ui) {
    let cerceve = egui::Frame::NONE.inner_margin(egui::Margin::symmetric(6, 3));
    egui::Panel::top("ust")
        .rol(Rol::UstCubuk)
        .frame(cerceve)
        .show_separator_line(false)
        .show(ui, |ui| {
            ui.label("arama");
        });
    egui::Panel::bottom("alt")
        .rol(Rol::DurumCubugu)
        .frame(cerceve)
        .show_separator_line(false)
        .show(ui, |ui| {
            ui.label("durum");
        });
    egui::CentralPanel::no_frame()
        .rol(Rol::Zemin)
        .frame(egui::Frame::NONE)
        .show(ui, |ui| {
            ui.label("liste");
        });
}

#[test]
fn nazar_iskeleti_kendi_cercevesini_ve_ayiriciyi_korur() {
    let mut h = Harness::builder()
        .with_size(egui::vec2(800.0, 500.0))
        .with_theme(egui::Theme::Light)
        .build_ui(nazar);
    h.run();
    let k = kilim_tema::rol_kayitlari(&h.ctx);
    println!(
        "--- nazar (işaretli) ---\n{}",
        kilim_tema::plan_metni(&h.ctx)
    );
    assert_eq!(roller(&k), [Rol::UstCubuk, Rol::DurumCubugu, Rol::Zemin]);
    // Aracın çerçevesi (şeffaf, 6x3 iç boşluk) rol çerçevesini (8x4) ezdi: sınama penceresinin 8 px kenar
    // boşluğu + aracın 6 px'i = 14 (rol çerçevesi olsaydı 16).
    let ust = k[0].rect;
    assert!(
        (ust.left() - 14.0).abs() < 0.6,
        "üst çubuk iç boşluğu aracın 6 px'i olmalı: {ust:?}"
    );
    // Rol dolgusu (UstCubuk: yüzey) çizilmemeli: araç çerçevesi dolgusuz.
    let yuzey = kilim_tema::acik().yuzey;
    let dolgular: Vec<_> = h
        .output()
        .shapes
        .iter()
        .filter_map(|c| {
            if let egui::Shape::Rect(r) = &c.shape {
                Some(r.fill)
            } else {
                None
            }
        })
        .collect();
    assert!(
        !dolgular.contains(&yuzey),
        "aracın çerçevesi varken rol dolgusu çizilmemeli"
    );
}
