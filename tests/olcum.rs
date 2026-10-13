//! Maliyet ölçümü (ADR 0001): `.rol(..)` işaretçisi ve içgözlem bedelleri. Hata ayıklama yapısı anlamsızdır:
//! `cargo test --release --test olcum -- --ignored --nocapture`.

use std::hint::black_box;
use std::time::{Duration, Instant};

use kilim_tema::{Rol, RolVer};

fn iskelet(ui: &mut egui::Ui, rollu: bool, n: usize) {
    macro_rules! bolge {
        ($panel:expr, $rol:expr, $ui:expr, $f:expr) => {
            if rollu {
                $panel.rol($rol).show($ui, $f);
            } else {
                $panel.show($ui, $f);
            }
        };
    }
    bolge!(egui::Panel::top("u"), Rol::UstCubuk, ui, |ui| {
        ui.horizontal(|ui| {
            let _ = ui.button("a");
        });
    });
    bolge!(egui::Panel::bottom("d"), Rol::DurumCubugu, ui, |ui| {
        ui.label("durum");
    });
    bolge!(egui::Panel::left("s"), Rol::YanPanel, ui, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            for i in 0..n {
                let _ = ui.selectable_label(i == 2, format!("öğe {i}"));
            }
        });
    });
    if rollu {
        egui::CentralPanel::default()
            .rol(Rol::Zemin)
            .show(ui, |ui| ui.label("orta"));
    } else {
        egui::CentralPanel::default().show(ui, |ui| ui.label("orta"));
    }
}

fn kare_suresi(rollu: bool, n: usize, izle: bool) -> Duration {
    let ctx = egui::Context::default();
    if izle {
        kilim_tema::rol_izi_ekle(&ctx);
    }
    let girdi = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(1200.0, 800.0),
        )),
        ..Default::default()
    };
    for _ in 0..5 {
        let _ = ctx.run_ui(girdi.clone(), |ui| iskelet(ui, rollu, n));
    }
    let k = 200;
    let t = Instant::now();
    for _ in 0..k {
        black_box(ctx.run_ui(girdi.clone(), |ui| iskelet(ui, rollu, n)));
    }
    t.elapsed() / k
}

#[test]
#[ignore = "ölçüm: yalnız --release ile anlamlı"]
fn rol_isaretcisi_kare_maliyeti() {
    // Gürültüye karşı: dönüşümlü 7 tur, her ölçümün en küçüğü.
    for n in [20, 300] {
        let (mut a, mut b, mut c) = (Duration::MAX, Duration::MAX, Duration::MAX);
        for _ in 0..7 {
            a = a.min(kare_suresi(false, n, false));
            b = b.min(kare_suresi(true, n, false));
            c = c.min(kare_suresi(true, n, true));
        }
        println!("n={n}: işaretsiz={a:?} | .rol() ile (4 bölge)={b:?} (fark {:?}) | + bindirme eklentisi={c:?}", b.saturating_sub(a));
    }
}

// ---------------------------------------------------------------- çalışma zamanı içgözlem bedeli

fn kalabalik(ui: &mut egui::Ui, n: usize) {
    egui::Panel::top("ust").show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.heading("başlık");
            let _ = ui.button("ara");
        });
    });
    egui::Panel::left("sol").default_size(250.0).show(ui, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            for i in 0..n {
                let _ = ui.selectable_label(i == 2, format!("öğe {i}"));
            }
        });
    });
    egui::CentralPanel::default().show(ui, |ui| {
        for i in 0..n {
            ui.horizontal(|ui| {
                ui.label(format!("satır {i}"));
                let _ = ui.button("x");
            });
        }
    });
}

/// (kare süresi, o karede `prev_pass.widgets` listesini dolaşma süresi, widget sayısı)
fn icgozlem(n: usize, accesskit: bool) -> (Duration, Duration, usize) {
    let ctx = egui::Context::default();
    if accesskit {
        ctx.enable_accesskit();
    }
    let girdi = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(1200.0, 800.0),
        )),
        ..Default::default()
    };
    for _ in 0..3 {
        let _ = ctx.run_ui(girdi.clone(), |ui| kalabalik(ui, n));
    }
    let k = 60;
    let (mut kare, mut okuma, mut say) = (Duration::ZERO, Duration::ZERO, 0usize);
    for _ in 0..k {
        let t = Instant::now();
        black_box(ctx.run_ui(girdi.clone(), |ui| kalabalik(ui, n)));
        kare += t.elapsed();
        let t = Instant::now();
        say = ctx.viewport(|v| {
            let mut s = 0usize;
            let mut alan = 0f32;
            for (_, rs) in v.prev_pass.widgets.layers() {
                for r in rs {
                    s += 1;
                    if !r.sense.interactive() && r.rect.width() > 200.0 && r.rect.height() > 20.0 {
                        alan += black_box(r.rect.area());
                    }
                }
            }
            black_box(alan);
            s
        });
        okuma += t.elapsed();
    }
    (kare / k, okuma / k, say)
}

#[test]
#[ignore = "ölçüm: yalnız --release ile anlamlı"]
fn icgozlem_maliyeti() {
    for n in [50, 300, 1000] {
        let (taban, okuma, say) = icgozlem(n, false);
        let (ak, _, _) = icgozlem(n, true);
        println!(
            "n={n} (widget {say}): taban={taban:?} | accesskit açık={ak:?} ({:+.0}%) | widget listesini dolaşma={okuma:?}",
            (ak.as_secs_f64() / taban.as_secs_f64() - 1.0) * 100.0
        );
    }
}
