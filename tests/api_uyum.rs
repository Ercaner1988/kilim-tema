//! Geriye uyum (ADR 0001 §Göç): kilim-tema'nın b62fcac'taki TÜM genel adları aynı yolla erişilebilir kalmalı
//! (pasli yol bağımlılığı, kesfuzzunun/nazar/el-fihrist git rev'leri). Sınama DERLEMEYLE geçer: bir ad
//! silinir ya da yer değiştirirse bu dosya derlenmez.

#![allow(unused_imports)]

use kilim_tema::{
    acik, alan_satiri, anahtar_ciz, birim, boncuk_dugme, boncuk_resmi, boncuklu, cerceve, dokular,
    etkin_paket, etkin_paketi_kaydet, etkin_paketi_sil, etkin_paketi_yukle, etkin_varyant,
    gecis_uygula, iki_temayi_kur, isaret_ciz, kagit_ciz, kagit_ciz_yuvarlak, kenarlik_ciz, koyu,
    kutu, paketi_kur, sabit_genislik, sarma_kur, secim, tablo, tema_secici, tema_secici_kaynakli,
    Aydinlik, Boncuk, Dokular, Gorunum, IslevTanimi, Kenarlik, Kilim, TemaBetigi, TemaKaynagi,
    TemaPaketi, TemaTercihi, Varyant, KALINLIK, KOYU_KAGIT,
};

#[test]
fn eski_adlar_ayni_imzayla_calisir() {
    // İmzaları da sabitler (yanlış tür derlenmez).
    let _: fn() -> Kilim = acik;
    let _: fn() -> Kilim = koyu;
    let _: fn(&egui::Context) = iki_temayi_kur;
    let _: fn(&egui::Context, f32) = gecis_uygula;
    let _: fn(&egui::Context) = sarma_kur;
    let _: fn(Option<TemaPaketi>) = paketi_kur;
    let _: fn() -> Option<TemaPaketi> = etkin_paket;
    let _: fn(&egui::Context) -> Varyant = etkin_varyant;
    let _: fn(&str) -> Result<TemaBetigi, String> = TemaBetigi::coz;
    type IslevFn =
        fn(&mut egui::Ui, IslevTanimi, fn(&mut egui::Ui) -> egui::Response) -> egui::Response;
    let _: IslevFn = kilim_tema::islev;
    let k = acik();
    let _ = k.gorsel(false);
    let _ = k.durum_rengi(Some("yesil"));
    let _ = Varyant::KirmiziYesil.vurgu(true);
    let _ = TemaTercihi::varsayilan(Varyant::CamGobegiAltin).kod();
    assert_eq!(KALINLIK, 32.0);
}
