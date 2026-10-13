//! Kilim paleti — paslı-beyin `tuval::tema`'dan taşındı (2026-10-01). Açık/koyu tema.

use egui::Color32;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Kilim {
    pub zem: Color32,
    pub yuzey: Color32,
    pub cizgi: Color32,
    pub cizgi_koyu: Color32,
    pub murekkep: Color32,
    pub soluk: Color32,
    pub cini: Color32,
    pub mercan: Color32,
    pub hardal: Color32,
    pub yesil: Color32,
    pub sari: Color32,
    pub kirmizi: Color32,
    pub gri: Color32,
    /// Tema ile ANTİTETİK sabit kontrast (koyu temada beyaz, açıkta siyah) —
    /// dolgu/ısı/dil rengi ne olursa olsun yapısal kutu/katman sınırları bu
    /// renkle çizilir ki komşu kutular birbirinden net ayrışsın (Ercan,
    /// 2026-09-24). Durum (git) rengi gibi ANLAM taşıyan çerçeveler bu kuralın
    /// DIŞINDA tutulur.
    pub kontrast: Color32,
}

fn hex(s: &str) -> Color32 {
    let s = s.trim_start_matches('#');
    let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(0);
    Color32::from_rgb(r, g, b)
}

/// Etkin tema paketinin açık paleti; paket yoksa yerleşik Kilim.
pub fn acik() -> Kilim {
    crate::paket::etkin_palet(false).unwrap_or_else(yerlesik_acik)
}

/// Etkin tema paketinin koyu paleti; paket yoksa yerleşik Kilim.
pub fn koyu() -> Kilim {
    crate::paket::etkin_palet(true).unwrap_or_else(yerlesik_koyu)
}

pub(crate) fn yerlesik_acik() -> Kilim {
    Kilim {
        zem: hex("#F4ECDB"),
        yuzey: hex("#FCF8EF"),
        cizgi: hex("#D8CBB2"),
        cizgi_koyu: hex("#B9A88C"),
        murekkep: hex("#3A2D21"),
        soluk: hex("#7A6A55"),
        cini: hex("#16537E"),
        mercan: hex("#C04F42"),
        hardal: hex("#D9A441"),
        yesil: hex("#2E7D4F"),
        sari: hex("#A8730A"),
        kirmizi: hex("#B23A32"),
        gri: hex("#8A9598"),
        kontrast: Color32::BLACK,
    }
}

pub(crate) fn yerlesik_koyu() -> Kilim {
    Kilim {
        zem: hex("#241F19"),
        yuzey: hex("#2E2820"),
        cizgi: hex("#4A4034"),
        cizgi_koyu: hex("#6E604C"),
        murekkep: hex("#EDE4D3"),
        soluk: hex("#B3A48C"),
        cini: hex("#7FB2D9"),
        mercan: hex("#E07A6E"),
        hardal: hex("#E4B45C"),
        yesil: hex("#5FB98A"),
        sari: hex("#D9A441"),
        kirmizi: hex("#E06A5F"),
        gri: hex("#9AA6A9"),
        kontrast: Color32::WHITE,
    }
}

impl Kilim {
    /// Durum (yesil/sari/kirmizi/gri) → kare rengi.
    pub fn durum_rengi(&self, durum: Option<&str>) -> Color32 {
        match durum {
            Some("yesil") => self.yesil,
            Some("sari") => self.sari,
            Some("kirmizi") => self.kirmizi,
            _ => self.gri,
        }
    }

    /// Metin kutusu zemini: koyuda zemden bir tık KOYU (zemle aynı olursa kutu panelde kaybolur), açıkta
    /// yüzey (zemden açık). `Visuals::extreme_bg_color` ve `Rol::Giris` aynı değeri kullanır.
    pub fn giris_zemini(&self, koyu_mu: bool) -> Color32 {
        if koyu_mu {
            self.zem.lerp_to_gamma(Color32::BLACK, 0.3)
        } else {
            self.yuzey
        }
    }

    pub fn uygula(&self, ctx: &egui::Context, koyu_mu: bool) {
        ctx.set_visuals(self.gorsel(koyu_mu));
    }

    pub fn gorsel(&self, koyu_mu: bool) -> egui::Visuals {
        let mut v = if koyu_mu {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        v.panel_fill = self.zem;
        v.window_fill = self.yuzey;
        v.override_text_color = Some(self.murekkep);
        v.widgets.noninteractive.bg_fill = self.yuzey;
        v.widgets.noninteractive.bg_stroke.color = self.cizgi;
        // Onay kutusu, radyo ve kaydırma tutamacının dolgusu: zeminden (yuzey/zem)
        // ayrışmalı, yoksa işaretsiz kutu pencerede görünmez. Düğmeler weak_bg_fill kullanır.
        v.widgets.inactive.bg_fill = self.cizgi_koyu;
        v.widgets.inactive.bg_stroke.color = self.cizgi;
        v.widgets.hovered.bg_stroke.color = self.cini;
        v.hyperlink_color = self.cini;
        v.selection.bg_fill = self.cini.gamma_multiply(0.35);
        // Anlamsal renkler (kunye-egui, sohbet-egui gibi temadan habersiz bileşenler `ui.visuals()`
        // üzerinden okur): hata/uyarı paletten, yüzeyde AA okunur. Metin kutusu zemini ve şeritli satır
        // dolgusu da paletten (egui'nin düz siyah/beyaz varsayılanları yerine).
        v.error_fg_color = yazi_rengi_ayarla(self.kirmizi, self);
        v.warn_fg_color = yazi_rengi_ayarla(self.sari, self);
        v.extreme_bg_color = self.giris_zemini(koyu_mu);
        v.faint_bg_color = self.cizgi.gamma_multiply(0.35);
        v
    }
}

/// Açık ve koyu Kilim görsellerini egui'ye birlikte kurar; hangisinin görüneceğini
/// `TemaTercihi::uygula` (ThemePreference) seçer.
pub fn iki_temayi_kur(ctx: &egui::Context) {
    ctx.set_visuals_of(egui::Theme::Light, acik().gorsel(false));
    ctx.set_visuals_of(egui::Theme::Dark, koyu().gorsel(true));
}

/// "Geçiş hızı" tercihi — egui'nin yerleşik widget'larının (CollapsingHeader vb.)
/// ve boncuk düğmenin okuduğu tek global süre.
pub fn gecis_uygula(ctx: &egui::Context, saniye: f32) {
    ctx.all_styles_mut(|s| s.animation_time = saniye);
}

// ------------------------------------------------------------------ erişilebilirlik yardımcıları

fn parlaklik(c: Color32) -> f32 {
    let d = |v: u8| {
        let v = f32::from(v) / 255.0;
        if v <= 0.03928 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * d(c.r()) + 0.7152 * d(c.g()) + 0.0722 * d(c.b())
}

/// WCAG 2.x karşıtlık oranı (1..21); iki renk de opak varsayılır.
pub fn karsitlik(a: Color32, b: Color32) -> f32 {
    let (x, y) = (parlaklik(a), parlaklik(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

/// `zemin` üstünde en okunur metin rengi (paletin mürekkebi, yüzeyi, beyaz, siyah arasından).
pub fn okunur_metin(zemin: Color32, k: &Kilim) -> Color32 {
    [k.murekkep, k.yuzey, Color32::WHITE, Color32::BLACK]
        .into_iter()
        .max_by(|a, b| karsitlik(*a, zemin).total_cmp(&karsitlik(*b, zemin)))
        .unwrap_or(k.murekkep)
}

/// `renk`i `zemin` üstünde okunacak yöne (palet mürekkebi okunuyorsa o, yoksa `okunur_metin`) kaydırarak
/// en az `hedef` karşıtlığa getirir.
pub fn karsitlik_ayarla(renk: Color32, zemin: Color32, hedef: f32, k: &Kilim) -> Color32 {
    let yon = if karsitlik(k.murekkep, zemin) >= hedef {
        k.murekkep
    } else {
        okunur_metin(zemin, k)
    };
    (0..=20)
        .map(|i| renk.lerp_to_gamma(yon, i as f32 * 0.05))
        .find(|c| karsitlik(*c, zemin) >= hedef)
        .unwrap_or(yon)
}

/// Anlamlı yazı rengi (hata, uyarı, vurgu...): hem `zem` hem `yuzey` üstünde AA okunur. Mürekkebe doğru
/// kaydırma her iki zeminde de karşıtlığı tek yönde artırır, o yüzden iki ayar art arda yeter.
pub fn yazi_rengi_ayarla(renk: Color32, k: &Kilim) -> Color32 {
    karsitlik_ayarla(karsitlik_ayarla(renk, k.zem, 4.5, k), k.yuzey, 4.5, k)
}
