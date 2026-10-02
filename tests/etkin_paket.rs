//! Etkin paket küresel durumdur; bu dosya ayrı süreçte çalışır ki birim sınamalarını
//! (yerleşik palet beklerler) bozmasın.

use egui::Color32;
use kilim_tema::{acik, koyu, etkin_paket, paketi_kur, TemaPaketi, Varyant};

#[test]
fn etkin_paket_palet_ve_vurguyu_degistirir_ve_geri_alinir() {
    let yerlesik_zem = acik().zem;
    let yerlesik_vurgu = Varyant::CamGobegiAltin.vurgu(false);
    assert!(etkin_paket().is_none());

    let mut p = TemaPaketi::yerlesik("deneme", "Deneme", Varyant::KirmiziYesil);
    p.acik.zem = Color32::from_rgb(1, 2, 3);
    p.koyu.cini = Color32::from_rgb(4, 5, 6);
    p.vurgu_acik = Color32::from_rgb(7, 8, 9);
    paketi_kur(Some(p));

    assert_eq!(acik().zem, Color32::from_rgb(1, 2, 3));
    assert_eq!(koyu().cini, Color32::from_rgb(4, 5, 6));
    assert_eq!(Varyant::CamGobegiAltin.vurgu(false), Color32::from_rgb(7, 8, 9));
    assert_eq!(Varyant::CamGobegiAltin.yerlesik_vurgu(false), yerlesik_vurgu);
    assert_eq!(etkin_paket().unwrap().id, "deneme");

    paketi_kur(None);
    assert_eq!(acik().zem, yerlesik_zem);
    assert_eq!(Varyant::CamGobegiAltin.vurgu(false), yerlesik_vurgu);
}
