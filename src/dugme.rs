//! Boncuklu etkileşimli öğeler: metin düğmesi, seçim (radyo), kutu (onay), anahtar.
//! egui'nin düz yeşil/mavi işaretleri yerine boncuk yuvası: seçili = boncuk, değil = boş yuva.

use egui::{
    pos2, vec2, Button, Color32, CornerRadius, Image, Painter, Rect, Response, Sense, Stroke,
    TextStyle, TextWrapMode, Ui, WidgetInfo, WidgetText, WidgetType,
};

use crate::{dokular, Boncuk, UV_TAM};

/// Seçim işaretlerinin boncuğu.
const ISARET: Boncuk = Boncuk::Firuze;

/// Boncuğun `cap` çaplı resmi: `Button::image_and_text` vb. için.
pub fn boncuk_resmi(ctx: &egui::Context, boncuk: Boncuk, cap: f32) -> Image<'static> {
    let doku = dokular(ctx).boncuk(boncuk).clone();
    Image::new((doku.id(), vec2(cap, cap)))
}

/// Solunda boncuk olan metin düğmesi. Dolgu/çizgi/köşe çağıranın (`.fill()` …).
pub fn boncuklu(ui: &Ui, boncuk: Boncuk, metin: impl Into<WidgetText>) -> Button<'static> {
    let cap = ui.text_style_height(&TextStyle::Button) + 4.0;
    Button::image_and_text(boncuk_resmi(ui.ctx(), boncuk, cap), metin)
}

/// Yuva: seçiliyse boncuk, değilse `cizgi` renkli boş halka (`kare`: köşeli onay yuvası).
pub fn isaret_ciz(
    painter: &Painter,
    yuva: Rect,
    secili: bool,
    kare: bool,
    cizgi: Color32,
    ton: Color32,
) {
    let r = yuva.width() * 0.5;
    let halka = Stroke::new(1.2, cizgi);
    if kare {
        painter.rect_stroke(
            yuva.shrink(1.0),
            CornerRadius::same(3),
            halka,
            egui::StrokeKind::Inside,
        );
    } else {
        painter.circle_stroke(yuva.center(), r - 1.0, halka);
    }
    if secili {
        let d = dokular(painter.ctx());
        painter.image(
            d.boncuk(ISARET).id(),
            yuva.shrink(if kare { 2.0 } else { 0.0 }),
            UV_TAM,
            ton,
        );
    }
}

fn isaretli(ui: &mut Ui, secili: bool, metin: impl Into<WidgetText>, kare: bool) -> Response {
    let cap = ui.spacing().interact_size.y;
    let bosluk = ui.spacing().icon_spacing;
    let galley = metin.into().into_galley(
        ui,
        Some(TextWrapMode::Extend),
        f32::INFINITY,
        TextStyle::Button,
    );
    let boy = vec2(cap + bosluk + galley.size().x, cap.max(galley.size().y));
    let (rect, cevap) = ui.allocate_exact_size(boy, Sense::click());
    let tur = if kare {
        WidgetType::Checkbox
    } else {
        WidgetType::RadioButton
    };
    cevap.widget_info(|| WidgetInfo::selected(tur, ui.is_enabled(), secili, galley.text()));
    if ui.is_rect_visible(rect) {
        let g = ui.style().interact(&cevap);
        let ustunde = ui.ctx().animate_bool(cevap.id, cevap.hovered());
        let yuva = Rect::from_center_size(
            pos2(rect.left() + cap * 0.5, rect.center().y),
            vec2(cap, cap) * (0.9 + 0.08 * ustunde),
        );
        let ton = if ui.is_enabled() {
            Color32::WHITE
        } else {
            Color32::from_gray(160).gamma_multiply(0.5)
        };
        isaret_ciz(ui.painter(), yuva, secili, kare, g.fg_stroke.color, ton);
        let yazi = pos2(
            rect.left() + cap + bosluk,
            rect.center().y - galley.size().y * 0.5,
        );
        ui.painter().galley(yazi, galley, g.text_color());
    }
    cevap
}

/// Radyo düğmesi: tıklanınca `*deger = bu`.
pub fn secim<T: PartialEq>(
    ui: &mut Ui,
    deger: &mut T,
    bu: T,
    metin: impl Into<WidgetText>,
) -> Response {
    let mut cevap = isaretli(ui, *deger == bu, metin, false);
    if cevap.clicked() && *deger != bu {
        *deger = bu;
        cevap.mark_changed();
    }
    cevap
}

/// Onay kutusu.
pub fn kutu(ui: &mut Ui, acik: &mut bool, metin: impl Into<WidgetText>) -> Response {
    let mut cevap = isaretli(ui, *acik, metin, true);
    if cevap.clicked() {
        *acik = !*acik;
        cevap.mark_changed();
    }
    cevap
}

/// Anahtar (toggle) çizimi: kanal `iz` tonlu kâğıt (bkz. [`crate::kagit_ciz`]), düğmesi boncuk. `konum` 0 = kapalı, 1 = açık
/// (`ctx.animate_bool` ile kaydırılabilir). Kapalıyken boncuk solgun.
pub fn anahtar_ciz(painter: &Painter, rect: Rect, konum: f32, iz: Color32) {
    crate::kagit_ciz_yuvarlak(painter, rect, iz, CornerRadius::same(255));
    let cap = rect.height() - 2.0;
    let x = egui::lerp(
        rect.left() + 1.0 + cap * 0.5..=rect.right() - 1.0 - cap * 0.5,
        konum,
    );
    let ton = Color32::from_gray(egui::lerp(150.0..=255.0, konum) as u8);
    let d = dokular(painter.ctx());
    let boncuk = Rect::from_center_size(pos2(x, rect.center().y), vec2(cap, cap));
    painter.image(d.boncuk(ISARET).id(), boncuk, UV_TAM, ton);
}
