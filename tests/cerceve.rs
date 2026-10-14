//! `cerceve_kalinlikli`: içerik kenarlığın İÇİNDE, verilen kalınlık kadar daraltılmış alanda çizilir;
//! `cerceve` aynı işi `KALINLIK` ile yapar; Sade görünümde kenarlık yoktur.

use std::cell::Cell;
use std::rc::Rc;

use egui::Rect;
use egui_kittest::Harness;
use kilim_tema::{cerceve, cerceve_kalinlikli, Gorunum, TemaTercihi, Varyant, KALINLIK};

/// Kök alanı ve içeriğin gördüğü alan.
fn olc(sar: impl Fn(&mut egui::Ui, &mut dyn FnMut(&mut egui::Ui)) + 'static) -> (Rect, Rect) {
    let dis = Rc::new(Cell::new(Rect::NOTHING));
    let ic = Rc::new(Cell::new(Rect::NOTHING));
    let (d, i) = (dis.clone(), ic.clone());
    let mut h = Harness::builder()
        .with_size(egui::vec2(400.0, 300.0))
        .build_ui(move |ui| {
            d.set(ui.max_rect());
            sar(ui, &mut |ui| i.set(ui.max_rect()));
        });
    h.run_steps(2);
    (dis.get(), ic.get())
}

#[test]
fn ince_kenarlik_verilen_kalinlikla_daraltir() {
    let t = TemaTercihi::varsayilan(Varyant::CamGobegiAltin);
    let (dis, ic) = olc(move |ui, f| cerceve_kalinlikli(ui, &t, 6.0, |ui| f(ui)));
    assert_eq!(ic, dis.shrink(6.0));
}

#[test]
fn cerceve_kalinligi_varsayilan() {
    let t = TemaTercihi::varsayilan(Varyant::CamGobegiAltin);
    let (dis, ic) = olc(move |ui, f| cerceve(ui, &t, |ui| f(ui)));
    assert_eq!(ic, dis.shrink(KALINLIK));
}

#[test]
fn sade_gorunumde_kenarlik_yok() {
    let mut t = TemaTercihi::varsayilan(Varyant::CamGobegiAltin);
    t.gorunum = Gorunum::Sade;
    let (dis, ic) = olc(move |ui, f| cerceve_kalinlikli(ui, &t, 6.0, |ui| f(ui)));
    assert_eq!(ic, dis);
}
