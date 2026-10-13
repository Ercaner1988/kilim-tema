//! Roller TEK kaynaktan (palet) türer: etkin tema paketi değişince rol renkleri de değişir. Etkin paket
//! küresel durumdur; bu dosya ayrı süreçte koşar (öteki sınamalar yerleşik palet bekler).

use egui::Color32;
use kilim_tema::{acik, paketi_kur, Rol, TemaPaketi, Varyant};

#[test]
fn tema_paketi_rol_renklerini_de_degistirir() {
    let v = Varyant::CamGobegiAltin;
    let once = acik();
    let eski = Rol::YanPanel.stili_hesapla(&once, false, v, false).dolgu;
    assert_eq!(eski, once.yuzey);

    let mut p = TemaPaketi::yerlesik("deneme", "Deneme", v);
    p.acik.yuzey = Color32::from_rgb(200, 210, 220);
    p.acik.kirmizi = Color32::from_rgb(120, 0, 0);
    paketi_kur(Some(p));

    let yeni = acik();
    assert_eq!(
        Rol::YanPanel.stili_hesapla(&yeni, false, v, false).dolgu,
        Color32::from_rgb(200, 210, 220)
    );
    assert_eq!(
        Rol::DugmeTehlikeli
            .stili_hesapla(&yeni, false, v, false)
            .dolgu,
        Color32::from_rgb(120, 0, 0)
    );
    // Bağlamdan türeyen kural da paketi izler (araç kodu hiçbir şey yapmadan).
    let ctx = egui::Context::default();
    assert_eq!(
        Rol::YanPanel.stili_ctx(&ctx, false).dolgu,
        Color32::from_rgb(200, 210, 220)
    );
    paketi_kur(None);
    assert_eq!(Rol::YanPanel.stili_ctx(&ctx, false).dolgu, once.yuzey);
}
