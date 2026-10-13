//! # kilim-palet
//!
//! `kilim-tema`nın renk çekirdeği: Kilim paleti (açık/koyu), tema paketi (`<uygulama>.tema`),
//! kenarlık varyantı ve WCAG karşıtlık yardımcıları. Egui çizimi/dokusu YOKTUR (yalnız `Color32` ve
//! `Visuals`). Araçlar bunu doğrudan değil `kilim_tema::*` yeniden dışa aktarımıyla kullanır; sandık
//! sınırı (1500 kod satırı) yüzünden ayrıldı, API değişmedi.

mod paket;
mod renk;

use egui::Color32;

pub use paket::{
    etkin_paket, etkin_paketi_kaydet, etkin_paketi_sil, etkin_paketi_yukle, paketi_kur,
    uygulama_dosyasi, TemaKaynagi, TemaPaketi, SEMA,
};
pub use renk::{
    acik, gecis_uygula, iki_temayi_kur, karsitlik, karsitlik_ayarla, koyu, okunur_metin,
    yazi_rengi_ayarla, Kilim,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Varyant {
    CamGobegiAltin,
    KirmiziYesil,
}

impl Varyant {
    /// Seçim/vurgu rengi: etkin tema paketi varsa onun, yoksa kenarlıktan (cam göbeği →
    /// firuze, kırmızı-yeşil → kök boya).
    /// Açıkta üstüne açık yazı, koyuda koyu yazı gelir (ikisi de WCAG AA, sınamada).
    pub fn vurgu(self, koyu_mu: bool) -> Color32 {
        paket::etkin_vurgu(koyu_mu).unwrap_or_else(|| self.yerlesik_vurgu(koyu_mu))
    }

    /// Etkin tema paketine bakmadan koddaki vurgu.
    pub fn yerlesik_vurgu(self, koyu_mu: bool) -> Color32 {
        match (self, koyu_mu) {
            (Varyant::CamGobegiAltin, false) => Color32::from_rgb(0x0E, 0x7C, 0x86),
            (Varyant::CamGobegiAltin, true) => Color32::from_rgb(0x5C, 0xC3, 0xC9),
            (Varyant::KirmiziYesil, false) => Color32::from_rgb(0xA8, 0x32, 0x2B),
            (Varyant::KirmiziYesil, true) => Color32::from_rgb(0xE0, 0x7A, 0x6E),
        }
    }
}
