//! # kilim-tema
//!
//! Paslı Beyin, bge-embed-rs, kervan ve el-fihrist'in ortak egui teması.
//! Katman kuralları (bkz. `Desktop\arayuz-tema-istemi.md` §5):
//! - kâğıt lifi **döşenir** (pencerenin sol üstüne sabit, gerilmez);
//! - leke/sararma **gerilir** (bulanık, gerilmesi görünmez);
//! - kilim kenarlık **9 dilim**: köşe sabit, kenar birimi tam sayıda tekrar eder;
//! - boncuk düğme **sabit boyutlu**, durumlar ton/ölçekle kodda türetilir.

mod doku;
mod renk;

pub use doku::{dokular, Dokular, Kenarlik};
pub use renk::{acik, gecis_uygula, koyu, Kilim};

use egui::epaint::{Mesh, Vertex};
use egui::{
    pos2, vec2, Align2, Color32, FontId, Painter, Pos2, Rect, Response, Sense, Shape, Ui, UiBuilder,
};

/// Kenarlık kalınlığı (mantıksal px). Dokular 64 px: 2x ekranda birebir.
pub const KALINLIK: f32 = 32.0;
/// Kâğıt karosu 1 doku pikseli = 0.5 mantıksal px (2x ekranda birebir).
const KAGIT_OLCEK: f32 = 0.5;
const UV_TAM: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Varyant {
    CamGobegiAltin,
    KirmiziYesil,
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

/// Kâğıt zemini (döşenmiş lif + gerilmiş leke) `rect`'e çizer.
pub fn kagit_ciz(painter: &Painter, rect: Rect) {
    let d = dokular(painter.ctx());
    let karo = d.kagit.size_vec2() * KAGIT_OLCEK;
    // UV ekran konumundan: karo pencerenin sol üstüne sabit, panel/rect değişse de kaymaz.
    let uv = Rect::from_min_max(
        (rect.min.to_vec2() / karo).to_pos2(),
        (rect.max.to_vec2() / karo).to_pos2(),
    );
    painter.image(d.kagit.id(), rect, uv, Color32::WHITE);
    painter.image(d.leke.id(), rect, UV_TAM, Color32::WHITE);
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

/// Uygulamanın kök `Ui`'sini kâğıt + kilim çerçeveyle sarar; içerik (paneller dahil)
/// kenarlığın içinde, saydam panel zeminiyle çizilir.
pub fn cerceve<R>(ui: &mut Ui, varyant: Varyant, icerik: impl FnOnce(&mut Ui) -> R) -> R {
    let rect = ui.max_rect();
    kagit_ciz(ui.painter(), rect);
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
