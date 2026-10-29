//! Taşmasız yerleşim kuralları (`kilim_tema::{sarma_kur, sabit_genislik, alan_satiri, tablo}`).
//! Her sınama birkaç kare koşar ve ölçüyü HER karede alır: kusur tek karelik değil, kareler arası kaymadır.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use egui::Ui;
use egui_kittest::Harness;

// egui_kittest 0.37 eksik glifi panikle reddeder (⚠ varsayılan yazı tiplerinde yok): gerçek iletideki simge "!" ile yazıldı.
const UZUN: &str = "! .github:2 Node.js 20 is deprecated. The following actions target Node.js 20 but are being forced to run on Node.js 24: actions/checkout@v4. Ayrıntı: https://github.blog/changelog/2025-09-19";
const ORTA: &str = "Ercaner1988/ibn-unNedim · git@github.com:Ercaner1988/ibn-unNedim.git";
const KARE: usize = 12;
/// Sınama penceresinde sağ panelin varsayılan genişliği.
const PANEL: f32 = 360.0;

/// Sağda yeniden boyutlanabilir panel; `icerik(ui, kare)` panelin içini çizer ve bir ölçü döndürür.
/// Her karenin (panel iç genişliği, ölçü) çifti döner.
fn panelde(sar: bool, icerik: impl Fn(&mut Ui, usize) -> f32 + 'static) -> Vec<(f32, f32)> {
    let olcum = Rc::new(RefCell::new(Vec::new()));
    let kare = Rc::new(Cell::new(0));
    let hazir = Rc::new(Cell::new(false));
    let (o, k, hz) = (olcum.clone(), kare.clone(), hazir.clone());
    let mut h = Harness::builder()
        .with_size(egui::vec2(1600.0, 800.0))
        .build_ui(move |ui| {
            // Uygulama sarma_kur'u ilk kareden ÖNCE çağırır; kurulum karesi de hiçbir şey çizmez.
            if !hz.get() {
                return;
            }
            egui::Panel::right("detay")
                .resizable(true)
                .default_size(PANEL)
                .show(ui, |ui| {
                    let deger = icerik(ui, k.get());
                    o.borrow_mut().push((ui.max_rect().width(), deger));
                });
            k.set(k.get() + 1);
        });
    if sar {
        kilim_tema::sarma_kur(&h.ctx);
    }
    hazir.set(true);
    for _ in 0..KARE {
        h.run_steps(1);
    }
    let v = olcum.borrow().clone();
    v
}

fn genislikler(v: &[(f32, f32)]) -> Vec<f32> {
    v.iter().map(|(g, _)| g.round()).collect()
}

#[test]
fn panel_genisligi_icerikten_bagimsiz_sismez_daralmaz() {
    // Sarılmayan uzun satır (sarma_kur yok) + 900 px'lik öğe: kalkansız panel şişer.
    let sisiren = panelde(false, |ui, _| {
        kilim_tema::sabit_genislik(ui, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.horizontal(|ui| ui.label(UZUN));
                    ui.allocate_exact_size(egui::vec2(900.0, 10.0), egui::Sense::hover());
                });
        });
        0.0
    });
    // Dolduran öğe yok, içerik dar: kalkansız panel içeriğe dek daralır.
    let daraltan = panelde(false, |ui, _| {
        kilim_tema::sabit_genislik(ui, |ui| ui.label("dar"));
        0.0
    });
    for (ad, v) in [("şişiren", &sisiren), ("daraltan", &daraltan)] {
        let g = genislikler(v);
        assert!(
            g.iter().all(|&x| (x - g[0]).abs() < 1.0 && x <= PANEL),
            "{ad}: panel genişliği kareler arasında değişmemeli ve varsayılanı aşmamalı: {g:?}"
        );
        assert!(g[0] > PANEL - 40.0, "{ad}: panel daralmamalı: {g:?}");
    }
}

#[test]
fn sarma_kur_yatay_satirdaki_metni_panel_icinde_sarar() {
    // Sağ panel taşmada SOLA doğru şişer (metnin sağ kenarı pencerede kalır): ölçü etiketin ve panelin
    // genişliğidir, varsayılan panel genişliğini aşmamalı.
    let v = panelde(true, |ui, _| {
        ui.horizontal(|ui| ui.label(UZUN)).inner.rect.width()
    });
    for (i, (panel, etiket)) in v.iter().enumerate() {
        assert!(
            *panel <= PANEL && *etiket <= *panel,
            "kare {i}: yatay satırdaki metin sarılmalı (etiket {etiket}, panel {panel})"
        );
    }
}

/// İçerik 6. karede değişir ("başka öğe seçildi"); değişimden sonraki İLK kare ile sonraki kareler aynı
/// yükseklikte olmalı (ara kare yok = içerik sıçramaz).
fn degisince_ilk_karede_oturur(ad: &str, v: &[(f32, f32)]) {
    let y: Vec<f32> = v.iter().map(|(_, h)| h.round()).collect();
    let degisim = 6;
    assert!(
        y[degisim..].iter().all(|&h| h == y[degisim]),
        "{ad}: içerik değişince yükseklik ilk karede oturmalı: {y:?}"
    );
    assert!(
        y[1..degisim].iter().all(|&h| h == y[1]),
        "{ad}: ilk içerik oturmalı: {y:?}"
    );
}

#[test]
fn alan_satiri_icerik_degisince_ilk_karede_oturur() {
    let v = panelde(true, |ui, kare| {
        let bas = ui.cursor().top();
        for k in ["Başlık", "URL", "Özet"] {
            kilim_tema::alan_satiri(ui, 90.0, k, |ui| {
                ui.label(if kare >= 6 { ORTA } else { UZUN })
            });
        }
        ui.label("son").rect.top() - bas
    });
    degisince_ilk_karede_oturur("alan_satiri", &v);
}

#[test]
fn tablo_hucresi_uzayinca_bayat_genislikte_sarilmaz() {
    let v = panelde(true, |ui, kare| {
        let bas = ui.cursor().top();
        kilim_tema::tablo(ui, egui::Grid::new("t").num_columns(3), |ui| {
            for _ in 0..3 {
                ui.label(if kare >= 6 {
                    "2026-10-11 17:33"
                } else {
                    "2026-10-11"
                });
                ui.label("claude");
                ui.label("12 KB");
                ui.end_row();
            }
        });
        ui.label("son").rect.top() - bas
    });
    degisince_ilk_karede_oturur("tablo", &v);
    let y: Vec<f32> = v.iter().map(|(_, h)| h.round()).collect();
    assert_eq!(
        y[2],
        y[KARE - 1],
        "hücre uzayınca satır yüksekliği değişmemeli (sarılmamalı): {y:?}"
    );
}

#[test]
fn birim_sarmali_satirda_combobox_tasirmaz() {
    // Dar panelde yan yana üç ComboBox: birimle her biri sığmazsa alt satıra iner, hiçbiri taşmaz.
    let v = panelde(true, |ui, _| {
        let sag = ui.max_rect().right();
        let mut en_sag: f32 = 0.0;
        ui.horizontal_wrapped(|ui| {
            for (i, metin) in ["claude (varsayılan model)", "Varsayılan", "Varsayılan"]
                .into_iter()
                .enumerate()
            {
                let r = kilim_tema::birim(ui, 170.0, |ui| {
                    egui::ComboBox::from_id_salt(i)
                        .selected_text(metin)
                        .wrap_mode(egui::TextWrapMode::Truncate)
                        .show_ui(ui, |_| {})
                        .response
                });
                en_sag = en_sag.max(r.rect.right());
            }
        });
        en_sag - sag
    });
    for (i, (_, tasma)) in v.iter().enumerate() {
        assert!(*tasma <= 0.5, "kare {i}: ComboBox satırı {tasma} px taştı");
    }
}
