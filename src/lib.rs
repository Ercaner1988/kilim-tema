//! # kilim-tema
//!
//! Paslı Beyin, bge-embed-rs, kervan ve el-fihrist'in ortak egui teması.
//! Katman kuralları (bkz. `Desktop\arayuz-tema-istemi.md` §5):
//! - kâğıt lifi **döşenir** (pencerenin sol üstüne sabit, gerilmez);
//! - leke/sararma **gerilir** (bulanık, gerilmesi görünmez);
//! - kilim kenarlık **9 dilim**: köşe sabit, kenar birimi tam sayıda tekrar eder;
//! - boncuk düğme **sabit boyutlu**, durumlar ton/ölçekle kodda türetilir.

mod doku;
mod dugme;
mod renk;
mod tercih;

pub use doku::{dokular, Dokular, Kenarlik};
pub use dugme::{anahtar_ciz, boncuk_resmi, boncuklu, isaret_ciz, kutu, secim};
pub use renk::{acik, gecis_uygula, iki_temayi_kur, koyu, Kilim};
pub use tercih::{tema_secici, Aydinlik, Gorunum, TemaTercihi};

use egui::epaint::{Mesh, Vertex};
use egui::{
    pos2, vec2, Align2, Color32, FontId, Painter, Pos2, Rect, Response, Sense, Shape, Ui, UiBuilder,
};

/// Kenarlık kalınlığı (mantıksal px). Dokular 64 px: 2x ekranda birebir.
pub const KALINLIK: f32 = 32.0;
/// Kâğıt karosu 1 doku pikseli = 0.5 mantıksal px (2x ekranda birebir).
const KAGIT_OLCEK: f32 = 0.5;
/// Koyu temada kâğıda çarpılan ton: kâğıt `koyu().yuzey`den bir tık açık kalır,
/// kutular/kartlar onun üstünde koyu durur (doku görünür, açık mürekkep okunur).
pub const KOYU_KAGIT: Color32 = Color32::from_rgb(60, 52, 43);
pub(crate) const UV_TAM: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Varyant {
    CamGobegiAltin,
    KirmiziYesil,
}

impl Varyant {
    /// Seçim/vurgu rengi kenarlıktan: cam göbeği → firuze, kırmızı-yeşil → kök boya.
    /// Açıkta üstüne açık yazı, koyuda koyu yazı gelir (ikisi de WCAG AA, sınamada).
    pub fn vurgu(self, koyu_mu: bool) -> Color32 {
        match (self, koyu_mu) {
            (Varyant::CamGobegiAltin, false) => Color32::from_rgb(0x0E, 0x7C, 0x86),
            (Varyant::CamGobegiAltin, true) => Color32::from_rgb(0x5C, 0xC3, 0xC9),
            (Varyant::KirmiziYesil, false) => Color32::from_rgb(0xA8, 0x32, 0x2B),
            (Varyant::KirmiziYesil, true) => Color32::from_rgb(0xE0, 0x7A, 0x6E),
        }
    }
}

const VARYANT_ID: &str = "kilim-tema-varyant";

/// `TemaTercihi::uygula`nın son kurduğu kenarlık varyantı (hiç kurulmadıysa cam göbeği).
pub fn etkin_varyant(ctx: &egui::Context) -> Varyant {
    ctx.data(|d| d.get_temp(egui::Id::new(VARYANT_ID)))
        .unwrap_or(Varyant::CamGobegiAltin)
}

pub(crate) fn varyanti_kaydet(ctx: &egui::Context, v: Varyant) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(VARYANT_ID), v));
    for (tema, koyu_mu) in [(egui::Theme::Light, false), (egui::Theme::Dark, true)] {
        ctx.style_mut_of(tema, |s| {
            s.visuals.selection.bg_fill = v.vurgu(koyu_mu).gamma_multiply(0.45);
            s.visuals.widgets.hovered.bg_stroke.color = v.vurgu(koyu_mu);
        });
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Boncuk {
    Firuze,
    Akik,
    Kehribar,
    Lapis,
    Telkari,
    Sirma,
    /// Merkezi gözün kendisi: simgesiz, özel düğme.
    Nazar,
    Tesbih,
    /// Krem kâğıtta zor seçilir: koyu zeminde kullanın.
    Sedef,
}

impl Boncuk {
    pub fn simge_rengi(self) -> Color32 {
        match self {
            Boncuk::Akik | Boncuk::Lapis | Boncuk::Tesbih | Boncuk::Nazar => {
                Color32::from_rgb(0xFC, 0xF8, 0xEF)
            }
            _ => Color32::from_rgb(0x3A, 0x2D, 0x21),
        }
    }
}

/// Kâğıt zemini (döşenmiş lif + gerilmiş leke) `rect`'e çizer. `ton` çarpılır:
/// açık temada `WHITE`, koyuda [`KOYU_KAGIT`] (doku kalır, zemin koyulaşır).
pub fn kagit_ciz(painter: &Painter, rect: Rect, ton: Color32) {
    let d = dokular(painter.ctx());
    let karo = d.kagit.size_vec2() * KAGIT_OLCEK;
    // UV ekran konumundan: karo pencerenin sol üstüne sabit, panel/rect değişse de kaymaz.
    let uv = Rect::from_min_max(
        (rect.min.to_vec2() / karo).to_pos2(),
        (rect.max.to_vec2() / karo).to_pos2(),
    );
    painter.image(d.kagit.id(), rect, uv, ton);
    painter.image(d.leke.id(), rect, UV_TAM, ton);
}

fn dortgen(m: &mut Mesh, p: [Pos2; 4], uv: [Pos2; 4]) {
    let i = m.vertices.len() as u32;
    for k in 0..4 {
        m.vertices.push(Vertex {
            pos: p[k],
            uv: uv[k],
            color: Color32::WHITE,
        });
    }
    m.add_triangle(i, i + 1, i + 2);
    m.add_triangle(i, i + 2, i + 3);
}

/// Kenar birimi kaç kez tekrar eder: tam sayı ("round"), motif yarıda kesilmez.
fn tekrar(uzunluk: f32, birim: f32) -> f32 {
    (uzunluk / birim).round().max(1.0)
}

/// Kilim kenarlığı `rect`'in içine, `t` kalınlığında çizer. Dış kenar dışa bakar.
pub fn kenarlik_ciz(painter: &Painter, rect: Rect, varyant: Varyant, t: f32) {
    let d = dokular(painter.ctx());
    let k = d.kenarlik(varyant);
    let birim = k.kenar.aspect_ratio() * t;
    let (x0, y0, x1, y1) = (rect.left(), rect.top(), rect.right(), rect.bottom());
    let (yatay, dikey) = (
        tekrar(x1 - x0 - 2.0 * t, birim),
        tekrar(y1 - y0 - 2.0 * t, birim),
    );
    let (xa, xb, ya, yb) = (x0 + t, x1 - t, y0 + t, y1 - t);
    let mut m = Mesh::with_texture(k.kenar.id());
    // v=0 her zaman dış kenarda; u kenar boyunca 0..n.
    dortgen(
        &mut m,
        [pos2(xa, y0), pos2(xb, y0), pos2(xb, ya), pos2(xa, ya)],
        [
            pos2(0.0, 0.0),
            pos2(yatay, 0.0),
            pos2(yatay, 1.0),
            pos2(0.0, 1.0),
        ],
    );
    dortgen(
        &mut m,
        [pos2(xa, yb), pos2(xb, yb), pos2(xb, y1), pos2(xa, y1)],
        [
            pos2(yatay, 1.0),
            pos2(0.0, 1.0),
            pos2(0.0, 0.0),
            pos2(yatay, 0.0),
        ],
    );
    dortgen(
        &mut m,
        [pos2(x0, ya), pos2(xa, ya), pos2(xa, yb), pos2(x0, yb)],
        [
            pos2(dikey, 0.0),
            pos2(dikey, 1.0),
            pos2(0.0, 1.0),
            pos2(0.0, 0.0),
        ],
    );
    dortgen(
        &mut m,
        [pos2(xb, ya), pos2(x1, ya), pos2(x1, yb), pos2(xb, yb)],
        [
            pos2(0.0, 1.0),
            pos2(0.0, 0.0),
            pos2(dikey, 0.0),
            pos2(dikey, 1.0),
        ],
    );
    painter.add(Shape::mesh(m));
    let kose = vec2(t, t);
    for (doku, sol_ust) in
        k.koseler
            .iter()
            .zip([pos2(x0, y0), pos2(xb, y0), pos2(xb, yb), pos2(x0, yb)])
    {
        painter.image(
            doku.id(),
            Rect::from_min_size(sol_ust, kose),
            UV_TAM,
            Color32::WHITE,
        );
    }
}

/// Uygulamanın kök `Ui`'sini tercihe göre sarar. Kilim: kâğıt + kenarlık, içerik (paneller
/// dahil) kenarlığın içinde saydam panel zeminiyle. Sade: yalnız uygulamanın düz zemini.
pub fn cerceve<R>(ui: &mut Ui, tercih: &TemaTercihi, icerik: impl FnOnce(&mut Ui) -> R) -> R {
    let rect = ui.max_rect();
    if tercih.gorunum == Gorunum::Sade {
        ui.painter().rect_filled(rect, 0.0, ui.visuals().panel_fill);
        return icerik(ui);
    }
    let varyant = tercih.varyant;
    // Koyuda aynı kâğıt koyulaştırılır: açık kâğıt üstünde açık metin okunmaz.
    let ton = if ui.visuals().dark_mode {
        KOYU_KAGIT
    } else {
        Color32::WHITE
    };
    kagit_ciz(ui.painter(), rect, ton);
    kenarlik_ciz(ui.painter(), rect, varyant, KALINLIK);
    ui.scope_builder(UiBuilder::new().max_rect(rect.shrink(KALINLIK)), |ui| {
        ui.visuals_mut().panel_fill = Color32::TRANSPARENT;
        icerik(ui)
    })
    .inner
}

/// Boncuk düğme: `cap` çaplı, üstüne `simge` (ör. egui-phosphor glifi) çizilir.
/// Üzerinde: hafif büyür; basılı: 1 px çöker ve kararır; pasif: soluk.
pub fn boncuk_dugme(ui: &mut Ui, boncuk: Boncuk, simge: &str, cap: f32) -> Response {
    let (rect, cevap) = ui.allocate_exact_size(vec2(cap, cap), Sense::click());
    if ui.is_rect_visible(rect) {
        let d = dokular(ui.ctx());
        let ustunde = ui.ctx().animate_bool(cevap.id, cevap.hovered());
        let basili = cevap.is_pointer_button_down_on();
        let (ton, kayma) = if !ui.is_enabled() {
            (Color32::from_gray(170).gamma_multiply(0.55), 0.0)
        } else if basili {
            (Color32::from_gray(215), 1.0)
        } else {
            (Color32::WHITE, 0.0)
        };
        let r = Rect::from_center_size(
            rect.center() + vec2(0.0, kayma),
            rect.size() * (1.0 + 0.06 * ustunde),
        );
        ui.painter().image(d.boncuk(boncuk).id(), r, UV_TAM, ton);
        if !simge.is_empty() && boncuk != Boncuk::Nazar {
            ui.painter().text(
                r.center(),
                Align2::CENTER_CENTER,
                simge,
                FontId::proportional(cap * 0.45),
                boncuk.simge_rengi(),
            );
        }
    }
    cevap
}

#[cfg(test)]
mod sinama {
    use super::*;

    #[test]
    fn kenar_birimi_tam_sayi_tekrar_eder() {
        assert_eq!(tekrar(700.0, 70.5), 10.0);
        assert_eq!(tekrar(740.0, 70.5), 10.0); // 10.5 → en yakın tam sayı, birim hafif esner
        assert_eq!(tekrar(20.0, 70.5), 1.0); // çok dar pencerede bile en az bir birim
    }
}

#[cfg(test)]
mod vurgu_sinama {
    use super::*;

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

    fn karsitlik(a: Color32, b: Color32) -> f32 {
        let (x, y) = (parlaklik(a), parlaklik(b));
        (x.max(y) + 0.05) / (x.min(y) + 0.05)
    }

    #[test]
    fn vurgu_ustundeki_yazi_okunur() {
        for v in [Varyant::CamGobegiAltin, Varyant::KirmiziYesil] {
            assert!(karsitlik(v.vurgu(false), acik().yuzey) >= 4.5, "{v:?} açık");
            assert!(karsitlik(v.vurgu(true), koyu().zem) >= 4.5, "{v:?} koyu");
        }
    }
}
