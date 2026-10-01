//! Kilim paleti — paslı-beyin `tuval::tema`'dan taşındı (2026-10-01). Açık/koyu tema.

use egui::Color32;

#[derive(Clone, Copy)]
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

pub fn acik() -> Kilim {
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

pub fn koyu() -> Kilim {
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

    pub fn uygula(&self, ctx: &egui::Context, koyu_mu: bool) {
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
        v.widgets.inactive.bg_fill = self.yuzey;
        v.widgets.inactive.bg_stroke.color = self.cizgi;
        v.widgets.hovered.bg_stroke.color = self.cini;
        v.hyperlink_color = self.cini;
        v.selection.bg_fill = self.cini.gamma_multiply(0.35);
        ctx.set_visuals(v);
    }
}

/// "Geçiş hızı" tercihi — egui'nin yerleşik widget'larının (CollapsingHeader vb.)
/// ve boncuk düğmenin okuduğu tek global süre.
pub fn gecis_uygula(ctx: &egui::Context, saniye: f32) {
    ctx.all_styles_mut(|s| s.animation_time = saniye);
}
