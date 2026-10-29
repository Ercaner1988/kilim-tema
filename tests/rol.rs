//! Roller ve `.rol(..)` işaretçisi (ADR 0001). Her sınama bir iddiayı ÖLÇER; bozma kanıtları ADR'de.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use egui::{epaint::RectShape, Color32, Shape, Ui};
use egui_kittest::Harness;
use kilim_tema::{acik, karsitlik, koyu, Rol, RolVer, Varyant};

fn parcala(s: &Shape, cikti: &mut Vec<Shape>) {
    match s {
        Shape::Vec(v) => v.iter().for_each(|x| parcala(x, cikti)),
        d => cikti.push(d.clone()),
    }
}

/// Karedeki bütün düz şekiller.
fn sekiller(h: &Harness<'_>) -> Vec<Shape> {
    let mut v = Vec::new();
    h.output()
        .shapes
        .iter()
        .for_each(|c| parcala(&c.shape, &mut v));
    v
}

fn dolgulu_dortgenler(h: &Harness<'_>) -> Vec<RectShape> {
    sekiller(h)
        .into_iter()
        .filter_map(|s| {
            if let Shape::Rect(r) = s {
                Some(r)
            } else {
                None
            }
        })
        .collect()
}

fn iskelet(ui: &mut Ui) {
    egui::Panel::top("ust").rol(Rol::UstCubuk).show(ui, |ui| {
        ui.label("menü");
    });
    egui::Panel::bottom("alt")
        .rol(Rol::DurumCubugu)
        .show(ui, |ui| {
            ui.label("durum");
        });
    egui::Panel::left("sol").rol(Rol::YanPanel).show(ui, |ui| {
        ui.label("liste");
    });
    egui::CentralPanel::default()
        .rol(Rol::Zemin)
        .show(ui, |ui| {
            ui.label("orta");
        });
}

// ---------------------------------------------------------------- 1. kural tablosu: okunurluk

/// Yarı saydam dolgu, görünür rengiyle (yüzey ya da zem üstünde) değerlendirilir.
fn gorunur(dolgu: Color32, taban: Color32) -> Color32 {
    let c = taban.blend(dolgu);
    Color32::from_rgb(c.r(), c.g(), c.b())
}

#[test]
fn her_rol_her_temada_okunur() {
    for (k, koyu_mu) in [(acik(), false), (koyu(), true)] {
        for v in [Varyant::CamGobegiAltin, Varyant::KirmiziYesil] {
            for kilim in [false, true] {
                for rol in Rol::HEPSI {
                    let s = rol.stili_hesapla(&k, koyu_mu, v, kilim);
                    let metin = s.metin.unwrap_or(k.murekkep);
                    for taban in [k.zem, k.yuzey] {
                        let zemin = gorunur(s.dolgu, taban);
                        let oran = karsitlik(metin, zemin);
                        assert!(
                            oran >= 4.5,
                            "{rol:?} koyu={koyu_mu} {v:?} kilim={kilim}: metin {metin:?} / zemin {zemin:?} = {oran:.2}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn bolge_dolgulari_ayristirilabilir() {
    // Aynı karede yan yana duran bölgeler birbirinden ve pencere zemininden ayrışmalı: dolgular çift çift
    // farklı olmalı (kâğıt görünümünde saydamlık nedeniyle yalnız YanPanel/UstCubuk/Kart/Yuzey için bakılır).
    for (k, koyu_mu) in [(acik(), false), (koyu(), true)] {
        let kural = |r: Rol| {
            r.stili_hesapla(&k, koyu_mu, Varyant::CamGobegiAltin, false)
                .dolgu
        };
        assert_ne!(
            kural(Rol::DurumCubugu),
            kural(Rol::YanPanel),
            "durum çubuğu ≠ yan panel"
        );
        assert_ne!(kural(Rol::Zemin), kural(Rol::YanPanel), "zemin ≠ yan panel");
        let ayirt = |r: Rol| {
            let s = r.stili_hesapla(&k, koyu_mu, Varyant::CamGobegiAltin, false);
            (s.dolgu, s.cizgi.color, s.cizgi.width.to_bits())
        };
        assert_ne!(
            ayirt(Rol::Giris),
            ayirt(Rol::Kart),
            "giriş ≠ kart (dolgu ya da çizgi)"
        );
        assert_ne!(kural(Rol::DugmeBirincil), kural(Rol::DugmeTehlikeli));
    }
}

// ---------------------------------------------------------------- 2. işaretçi: defter, çerçeve, uyum

#[test]
fn isaretli_bolgeler_deftere_dogru_rect_ve_kapla_yazilir() {
    let mut h = Harness::builder()
        .with_size(egui::vec2(900.0, 600.0))
        .build_ui(iskelet);
    h.run();
    let k = kilim_tema::rol_kayitlari(&h.ctx);
    let roller: Vec<Rol> = k.iter().map(|x| x.rol).collect();
    assert_eq!(
        roller,
        [Rol::UstCubuk, Rol::DurumCubugu, Rol::YanPanel, Rol::Zemin]
    );
    assert!(k.iter().all(|x| x.uyumlu), "{k:?}");
    let by = |r: Rol| k.iter().find(|x| x.rol == r).unwrap().rect;
    // Geometri: üst çubuk pencerenin üstünde, durum altta, yan panel bu ikisinin arasında solda.
    assert!(by(Rol::UstCubuk).top() < 40.0 && by(Rol::UstCubuk).width() > 800.0);
    assert!(by(Rol::DurumCubugu).bottom() > 560.0);
    assert!(by(Rol::YanPanel).left() < 20.0 && by(Rol::YanPanel).top() > by(Rol::UstCubuk).top());
    assert!(by(Rol::Zemin).left() > by(Rol::YanPanel).left());
    assert!(kilim_tema::plan_metni(&h.ctx).contains("YanPanel"));
}

#[test]
fn rol_cercevenin_dolgusunu_belirler() {
    let mut h = Harness::builder()
        .with_size(egui::vec2(900.0, 600.0))
        .with_theme(egui::Theme::Light)
        .build_ui(iskelet);
    h.run();
    let k = acik();
    let beklenen = |r: Rol| {
        r.stili_hesapla(&k, false, Varyant::CamGobegiAltin, false)
            .dolgu
    };
    let dortgenler = dolgulu_dortgenler(&h);
    for rol in [Rol::UstCubuk, Rol::DurumCubugu, Rol::YanPanel, Rol::Zemin] {
        let kayit = kilim_tema::rol_kayitlari(&h.ctx)
            .into_iter()
            .find(|x| x.rol == rol)
            .unwrap();
        assert!(
            dortgenler
                .iter()
                .any(|d| d.fill == beklenen(rol) && d.rect.contains_rect(kayit.rect)),
            "{rol:?}: dolgu {:?} çizilmedi",
            beklenen(rol)
        );
    }
}

#[test]
fn uyumsuz_rol_yakalanir() {
    let mut h = Harness::builder()
        .with_size(egui::vec2(600.0, 400.0))
        .build_ui(|ui| {
            // Üst panele yan panel rolü verilmiş: işaretçi hatası.
            egui::Panel::top("yanlis")
                .rol(Rol::YanPanel)
                .show(ui, |ui| {
                    ui.label("x");
                });
        });
    h.run();
    let k = kilim_tema::rol_kayitlari(&h.ctx);
    assert_eq!(k.len(), 1);
    assert!(!k[0].uyumlu, "{k:?}");
    assert!(kilim_tema::plan_metni(&h.ctx).contains("UYUMSUZ"));
}

#[test]
fn defter_her_karede_sifirlanir() {
    let n = Rc::new(Cell::new(0usize));
    let m = n.clone();
    let mut h = Harness::builder()
        .with_size(egui::vec2(600.0, 400.0))
        .build_ui(move |ui| {
            m.set(m.get() + 1);
            egui::Panel::left("s").rol(Rol::YanPanel).show(ui, |ui| {
                ui.label("x");
            });
        });
    h.run_steps(5);
    assert_eq!(
        kilim_tema::rol_kayitlari(&h.ctx).len(),
        1,
        "5 kare sonra hâlâ tek kayıt"
    );
    assert!(n.get() >= 5);
}

#[test]
fn isaretleme_bosta_yeniden_cizim_istemez() {
    // Harness::run, yeniden çizim istenmeyene dek koşar ve `max_steps`i aşarsa panik atar: işaretçi her
    // karede `request_repaint` isteseydi bu sınama düşerdi.
    let mut h = Harness::builder()
        .with_size(egui::vec2(900.0, 600.0))
        .build_ui(iskelet);
    let adim = h.run();
    assert!(adim <= 3, "boşta {adim} kare çizildi");
    assert!(
        !h.ctx.has_requested_repaint(),
        "boşta yeniden çizim istendi"
    );
}

// ---------------------------------------------------------------- 3. rolden otomatik yerleşim kararları

// egui_kittest 0.37 eksik glifi panikle reddeder (⚠ varsayılan yazı tiplerinde yok): gerçek iletideki simge "!" ile yazıldı.
const UZUN: &str = "! .github:2 Node.js 20 is deprecated. The following actions target Node.js 20 but are being forced to run on Node.js 24: actions/checkout@v4. Ayrıntı: https://github.blog/changelog/2025-09-19";

/// Sağ panelin iç genişliği her karede ölçülür.
fn panel_genislikleri(rol: Option<Rol>) -> Vec<f32> {
    let olcum = Rc::new(RefCell::new(Vec::new()));
    let o = olcum.clone();
    let mut h = Harness::builder()
        .with_size(egui::vec2(1600.0, 800.0))
        .build_ui(move |ui| {
            let panel = egui::Panel::right("detay")
                .resizable(true)
                .default_size(360.0);
            let govde = |ui: &mut Ui| {
                // Sarılamayan 900 px'lik öğe + yatay satırda uzun metin: kalkansız panel şişer.
                egui::ScrollArea::vertical()
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        ui.horizontal(|ui| ui.label(UZUN));
                        ui.allocate_exact_size(egui::vec2(900.0, 10.0), egui::Sense::hover());
                    });
                // Rolsüz: panel içi `Ui`'nin genişliği. (Rollüde içerik, genişliği koruyan alt `Ui`'de çizilir ve o
                // `Ui` taşan öğeyle büyür; panel genişliği deftere yazılan bölge rect'idir.)
                if rol.is_none() {
                    o.borrow_mut().push(ui.max_rect().width());
                }
            };
            match rol {
                Some(r) => {
                    panel.rol(r).show(ui, govde);
                    let k = kilim_tema::rol_kayitlari(ui.ctx());
                    o.borrow_mut().push(k[0].rect.width());
                }
                None => {
                    panel.show(ui, govde);
                }
            }
        });
    for _ in 0..10 {
        h.run_steps(1);
    }
    let v = olcum.borrow().clone();
    v
}

#[test]
fn yan_panel_rolu_genisligi_kendiliginden_sabitler() {
    let rolsuz = panel_genislikleri(None);
    let rollu = panel_genislikleri(Some(Rol::YanPanel));
    let degisim = |v: &[f32]| {
        v.iter().cloned().fold(f32::MIN, f32::max) - v.iter().cloned().fold(f32::MAX, f32::min)
    };
    assert!(
        rolsuz.iter().any(|&g| g > 400.0),
        "kontrol: rolsüz panel varsayılanın (360) üstüne şişmeli: {rolsuz:?}"
    );
    assert!(
        degisim(&rollu) < 1.0,
        "rollü panelin genişliği kareler arası değişmemeli: {rollu:?}"
    );
    assert!(
        rollu.iter().all(|&g| g <= 361.0),
        "varsayılan genişliği aşmamalı: {rollu:?}"
    );
}

#[test]
fn durum_cubugu_rolu_uzun_metni_tek_satirda_keser() {
    // `sarma_kur` ÇAĞRILMADI: rol kendi sarma kuralını getirir. Uzun metin durum çubuğunu yükseltmemeli.
    let yukseklik = |metin: &'static str| {
        let m = Rc::new(Cell::new(0.0f32));
        let mm = m.clone();
        let mut h = Harness::builder()
            .with_size(egui::vec2(500.0, 300.0))
            .build_ui(move |ui| {
                egui::Panel::bottom("d")
                    .rol(Rol::DurumCubugu)
                    .show(ui, |ui| {
                        ui.label(metin);
                        mm.set(ui.min_rect().height());
                    });
            });
        h.run_steps(3);
        m.get()
    };
    let (kisa, uzun) = (yukseklik("hazır"), yukseklik(UZUN));
    assert!(
        (kisa - uzun).abs() < 0.5,
        "durum çubuğu yüksekliği: kısa {kisa}, uzun {uzun}"
    );
}

// ---------------------------------------------------------------- 4. bağlam: bagil_rol, rol_rengi

#[test]
fn bagil_rol_ic_ice_ui_lerden_bulunur() {
    let sonuc = Rc::new(RefCell::new(Vec::new()));
    let s = sonuc.clone();
    let mut h = Harness::builder()
        .with_size(egui::vec2(900.0, 500.0))
        .build_ui(move |ui| {
            let s2 = s.clone();
            egui::Panel::left("sol")
                .rol(Rol::YanPanel)
                .show(ui, move |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.horizontal(|ui| {
                            s2.borrow_mut().push(("sol-içi", kilim_tema::bagil_rol(ui)))
                        });
                    });
                });
            egui::CentralPanel::default().show(ui, |ui| {
                s.borrow_mut()
                    .push(("rolsüz-merkez", kilim_tema::bagil_rol(ui)));
            });
        });
    h.run_steps(2);
    let v = sonuc.borrow().clone();
    assert!(v.contains(&("sol-içi", Some(Rol::YanPanel))), "{v:?}");
    assert!(v.contains(&("rolsüz-merkez", None)), "{v:?}");
}

#[test]
fn rol_rengi_gercek_arka_plana_gore_ayarlanir() {
    // Açık temada koyu dolgulu bir çerçeve içinde: statik `Uyari` rengi (açık yüzey için seçilmişti) burada
    // okunmaz; `rol_rengi` çerçevenin gerçek dolgusuna bakar (`ui.stack().bg_color()`).
    let koyu_dolgu = Color32::from_rgb(25, 22, 20);
    let sonuc = Rc::new(Cell::new((Color32::BLACK, Color32::BLACK)));
    let s = sonuc.clone();
    let mut h = Harness::builder()
        .with_size(egui::vec2(400.0, 300.0))
        .with_theme(egui::Theme::Light)
        .build_ui(move |ui| {
            egui::Frame::new().fill(koyu_dolgu).show(ui, |ui| {
                s.set((
                    kilim_tema::rol_rengi(ui, Rol::Uyari),
                    Rol::Uyari.stili(ui).metin.unwrap(),
                ));
                ui.label("x");
            });
        });
    h.run_steps(2);
    let (uyarlanan, statik) = sonuc.get();
    assert!(
        karsitlik(statik, koyu_dolgu) < 4.5,
        "kontrol: statik renk koyu zeminde okunmamalı ({:.2})",
        karsitlik(statik, koyu_dolgu)
    );
    assert!(
        karsitlik(uyarlanan, koyu_dolgu) >= 4.5,
        "uyarlanan {uyarlanan:?} = {:.2}",
        karsitlik(uyarlanan, koyu_dolgu)
    );
}

// ---------------------------------------------------------------- 5. araç esnekliği

#[test]
fn arac_rolu_ezebilir_cerceveyi_degistirebilir() {
    let ozel = Color32::from_rgb(1, 2, 3);
    let ctx_ezme = Color32::from_rgb(4, 5, 6);
    let mut h = Harness::builder()
        .with_size(egui::vec2(900.0, 600.0))
        .build_ui(move |ui| {
            // Bölge başına ezme, bağlam ezmesinden ÖNCE kazanır (daha özgül).
            egui::Panel::top("ust")
                .rol(Rol::UstCubuk)
                .duzenle(move |s| s.dolgu = ozel)
                .show(ui, |ui| {
                    ui.label("a");
                });
            egui::Panel::bottom("alt")
                .rol(Rol::DurumCubugu)
                .show(ui, |ui| {
                    ui.label("b");
                });
            egui::Panel::left("sol")
                .rol(Rol::YanPanel)
                .frame(egui::Frame::new().fill(Color32::from_rgb(7, 8, 9)))
                .show(ui, |ui| {
                    ui.label("c");
                });
        });
    kilim_tema::rol_ezme(&h.ctx, Rol::DurumCubugu, move |s| s.dolgu = ctx_ezme);
    h.run();
    let doldur: Vec<Color32> = dolgulu_dortgenler(&h).into_iter().map(|d| d.fill).collect();
    assert!(doldur.contains(&ozel), "duzenle");
    assert!(doldur.contains(&ctx_ezme), "rol_ezme");
    assert!(doldur.contains(&Color32::from_rgb(7, 8, 9)), "frame");
    // Aracın kendi çerçevesi verildiğinde de rol sabit_genislik kuralını sürdürür.
    assert!(kilim_tema::rol_kayitlari(&h.ctx)
        .iter()
        .any(|k| k.rol == Rol::YanPanel && k.sabit_genislik));
}

#[test]
fn rollu_dugme_dolgusunu_rolden_alir() {
    let mut h = Harness::builder()
        .with_size(egui::vec2(400.0, 200.0))
        .with_theme(egui::Theme::Light)
        .build_ui(|ui| {
            let _ = ui.add(kilim_tema::dugme(ui, Rol::DugmeTehlikeli, "Sil"));
            let _ = ui.add(kilim_tema::dugme(ui, Rol::DugmeBirincil, "Kaydet"));
        });
    h.run_steps(2);
    let k = acik();
    let v = Varyant::CamGobegiAltin;
    let dolgular: Vec<Color32> = dolgulu_dortgenler(&h).into_iter().map(|d| d.fill).collect();
    assert!(dolgular.contains(&k.kirmizi), "tehlikeli = palet kırmızısı");
    assert!(
        dolgular.contains(&v.vurgu(false)),
        "birincil = varyant vurgusu"
    );
}

// ---------------------------------------------------------------- 6. işaretsiz araçlar için palet

#[test]
fn gorsel_anlamsal_renkleri_paletten_ve_okunur_verir() {
    for (k, koyu_mu) in [(acik(), false), (koyu(), true)] {
        let v = k.gorsel(koyu_mu);
        for (ad, c) in [("hata", v.error_fg_color), ("uyarı", v.warn_fg_color)] {
            for (zad, z) in [("panel", v.panel_fill), ("pencere", v.window_fill)] {
                let oran = karsitlik(c, z);
                assert!(oran >= 4.5, "{ad} / {zad} koyu={koyu_mu}: {oran:.2}");
            }
        }
    }
}

// ---------------------------------------------------------------- 7. KILIM_ROL_IZ bindirmesi

fn metinler(h: &Harness<'_>) -> Vec<String> {
    sekiller(h)
        .into_iter()
        .filter_map(|s| {
            if let Shape::Text(t) = s {
                Some(t.galley.text().to_string())
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn rol_izi_bindirmesi_rol_adlarini_yazar_ve_ekleyince_acilir() {
    let mut kapali = Harness::builder()
        .with_size(egui::vec2(900.0, 600.0))
        .build_ui(iskelet);
    kapali.run();
    assert!(
        !metinler(&kapali).iter().any(|t| t == "YanPanel"),
        "bindirme kapalıyken rol adı çizilmemeli"
    );

    let mut acik_h = Harness::builder()
        .with_size(egui::vec2(900.0, 600.0))
        .build_ui(iskelet);
    kilim_tema::rol_izi_ekle(&acik_h.ctx);
    acik_h.run();
    let m = metinler(&acik_h);
    for ad in ["UstCubuk", "DurumCubugu", "YanPanel", "Zemin"] {
        assert!(m.iter().any(|t| t == ad), "{ad} bindirmede yok: {m:?}");
    }
}

// ---------------------------------------------------------------- 8. giydir ve görünüm

#[test]
fn giydir_gorunume_gore_saydam_ya_da_opak_panel_cizer() {
    use kilim_tema::{Aydinlik, Gorunum, TemaTercihi};
    for (g, saydam) in [(Gorunum::Kilim, true), (Gorunum::Sade, false)] {
        let mut h = Harness::builder()
            .with_size(egui::vec2(900.0, 600.0))
            .with_theme(egui::Theme::Light)
            .build_ui(iskelet);
        let t = TemaTercihi {
            gorunum: g,
            varyant: Varyant::CamGobegiAltin,
            aydinlik: Aydinlik::Acik,
        };
        kilim_tema::giydir(&h.ctx, &t);
        h.run();
        assert_eq!(kilim_tema::etkin_gorunum(&h.ctx), g);
        let dolgu = Rol::YanPanel.stili_ctx(&h.ctx, false).dolgu;
        assert_eq!(
            dolgu.a() < 255,
            saydam,
            "{g:?}: yan panel dolgusu {dolgu:?}"
        );
        let yan = kilim_tema::rol_kayitlari(&h.ctx)
            .into_iter()
            .find(|k| k.rol == Rol::YanPanel)
            .unwrap();
        assert!(
            dolgulu_dortgenler(&h)
                .iter()
                .any(|d| d.fill == dolgu && d.rect.contains_rect(yan.rect)),
            "{g:?}: yan panel dolgusu çizilmedi"
        );
        assert_eq!(
            h.ctx.global_style().wrap_mode,
            Some(egui::TextWrapMode::Wrap),
            "giydir sarma_kur'u da kurar"
        );
    }
}

// ---------------------------------------------------------------- 9. Frame rolleri

#[test]
fn kart_ve_acilir_cerceveleri_rolden_gelir() {
    let mut h = Harness::builder()
        .with_size(egui::vec2(400.0, 300.0))
        .with_theme(egui::Theme::Light)
        .build_ui(|ui| {
            egui::Frame::group(ui.style())
                .rol(Rol::Kart)
                .show(ui, |ui| {
                    ui.label("kart");
                });
            egui::Frame::popup(ui.style())
                .rol(Rol::Acilir)
                .show(ui, |ui| {
                    ui.label("açılır");
                });
        });
    h.run();
    let k = acik();
    let v = Varyant::CamGobegiAltin;
    let kart = Rol::Kart.stili_hesapla(&k, false, v, false);
    let acilir = Rol::Acilir.stili_hesapla(&k, false, v, false);
    let d = dolgulu_dortgenler(&h);
    assert!(
        d.iter()
            .any(|r| r.fill == kart.dolgu && r.stroke.color == kart.cizgi.color),
        "kart çerçevesi çizilmedi"
    );
    assert!(
        d.iter()
            .any(|r| r.fill == acilir.dolgu && r.stroke.color == acilir.cizgi.color),
        "açılır çerçevesi çizilmedi"
    );
    let roller: Vec<Rol> = kilim_tema::rol_kayitlari(&h.ctx)
        .iter()
        .map(|x| x.rol)
        .collect();
    assert_eq!(roller, [Rol::Kart, Rol::Acilir]);
}

// ---------------------------------------------------------------- 10. sabit yükseklikli çubuklar

#[test]
fn ust_cubuk_rolu_sabit_yukseklikli_cubugun_icerigini_kisaltmaz() {
    // kesfuzzunun menü çubuğu `exact_size(24.0)`: rol çerçevesinin dikey iç boşluğu egui'nin varsayılanından
    // büyükse içerik alanı daralır ve düğmeler kırpılır.
    let yukseklik = |rollu: bool| {
        let y = Rc::new(Cell::new(0.0f32));
        let yy = y.clone();
        let mut h = Harness::builder()
            .with_size(egui::vec2(800.0, 300.0))
            .build_ui(move |ui| {
                let panel = egui::Panel::top("menu").exact_size(24.0);
                let govde = |ui: &mut Ui| yy.set(ui.max_rect().height());
                if rollu {
                    panel.rol(Rol::UstCubuk).show(ui, govde);
                } else {
                    panel.show(ui, govde);
                }
            });
        h.run_steps(3);
        y.get()
    };
    let (duz, rollu) = (yukseklik(false), yukseklik(true));
    assert!(
        rollu >= duz - 0.5,
        "rollü çubuğun içerik yüksekliği {rollu} < düz egui {duz}"
    );
}

// ---------------------------------------------------------------- 11. metin kutusu zemini

#[test]
fn metin_kutusu_zemini_panelden_ayrisir_ve_yazi_okunur() {
    for (k, koyu_mu) in [(acik(), false), (koyu(), true)] {
        let v = k.gorsel(koyu_mu);
        let ayrisma = karsitlik(v.extreme_bg_color, v.panel_fill);
        assert!(
            ayrisma >= 1.05,
            "koyu={koyu_mu}: kutu zemini panelden ayrışmıyor ({ayrisma:.3})"
        );
        let yazi = karsitlik(k.murekkep, v.extreme_bg_color);
        assert!(
            yazi >= 4.5,
            "koyu={koyu_mu}: kutuda yazı okunmuyor ({yazi:.2})"
        );
    }
}

// ---------------------------------------------------------------- 12. ret kanıtı: işaretsiz panel ayırt edilemez

#[test]
fn isaretsiz_bes_panel_ayni_cerceveyi_alir_stil_bunlari_ayiramaz() {
    // ADR 0001 §Saptama-(a): egui panel çerçevesini ÜST `Ui`'nin stilinden, içerik kapanışı çalışmadan ÖNCE
    // çözer ve beş panel türü (üst, alt, sol, sağ, merkez) aynı `panel_fill`i alır. Bu yüzden hiçbir küresel
    // `Style` onları ayıramaz; ayrım yalnız çağrı yerindeki işaretçiyle (`.rol`) olur.
    let mut h = Harness::builder()
        .with_size(egui::vec2(900.0, 600.0))
        .with_theme(egui::Theme::Light)
        .build_ui(|ui| {
            egui::Panel::top("a").show(ui, |ui| {
                ui.label("a");
            });
            egui::Panel::bottom("b").show(ui, |ui| {
                ui.label("b");
            });
            egui::Panel::left("c").show(ui, |ui| {
                ui.label("c");
            });
            egui::Panel::right("d").show(ui, |ui| {
                ui.label("d");
            });
            egui::CentralPanel::default().show(ui, |ui| {
                ui.label("e");
            });
        });
    kilim_tema::iki_temayi_kur(&h.ctx);
    h.run();
    let panel = acik().gorsel(false).panel_fill;
    let buyuk: Vec<Color32> = dolgulu_dortgenler(&h)
        .into_iter()
        .filter(|r| r.rect.area() > 400.0 && r.fill != Color32::TRANSPARENT)
        .map(|r| r.fill)
        .collect();
    assert!(buyuk.len() >= 5, "beş panelin dolgusu bulunmalı: {buyuk:?}");
    assert!(
        buyuk.iter().all(|&f| f == panel),
        "beş panel de aynı panel_fill'i almalı: {buyuk:?}"
    );
}
