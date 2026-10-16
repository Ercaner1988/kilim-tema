//! Çerçevesiz uygulamalar için ortak, erişilebilir pencere düğmeleri.
//! Gösterme/gizleme ve yerel pencere komutlarını çağıran uygulama yönetir.

use crate::Boncuk;
use egui::{pos2, vec2, CursorIcon, Response, Sense, Stroke, Ui, WidgetInfo, WidgetType};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PencereDugmesi {
    Gizle,
    Buyut,
    GeriAl,
    Tasi,
}

impl PencereDugmesi {
    pub fn etiket(self) -> &'static str {
        match self {
            Self::Gizle => "Pencereyi gizle",
            Self::Buyut => "Pencereyi büyüt",
            Self::GeriAl => "Önceki boyuta dön",
            Self::Tasi => "Pencereyi taşı",
        }
    }
}

/// Boncuk arka planı ve yazı tipinden bağımsız çizilen simge; aydınlık/koyu ve sade temaya uyar.
/// `Tasi` hem sürükleme hem tıklama yanıtı verir. Eylemin anlamını uygulama belirler.
pub fn pencere_dugmesi(ui: &mut Ui, tur: PencereDugmesi, cap: f32) -> Response {
    let boncuk = match tur {
        PencereDugmesi::Gizle => Boncuk::Akik,
        PencereDugmesi::Buyut | PencereDugmesi::GeriAl => Boncuk::Kehribar,
        PencereDugmesi::Tasi => Boncuk::Firuze,
    };
    let duyu = if tur == PencereDugmesi::Tasi {
        Sense::click_and_drag()
    } else {
        Sense::click()
    };
    let r = crate::boncuk_dugme_duyulu(ui, boncuk, "", cap, duyu);
    r.widget_info(|| WidgetInfo::labeled(WidgetType::Button, ui.is_enabled(), tur.etiket()));
    if ui.is_rect_visible(r.rect) {
        let c = r.rect.center()
            + vec2(
                0.0,
                if r.is_pointer_button_down_on() {
                    1.0
                } else {
                    0.0
                },
            );
        let a = cap * 0.18;
        let s = Stroke::new(
            1.5,
            boncuk
                .simge_rengi()
                .gamma_multiply(if ui.is_enabled() { 1.0 } else { 0.5 }),
        );
        let p = ui.painter();
        match tur {
            PencereDugmesi::Gizle => {
                p.line_segment([c - vec2(a, a), c + vec2(a, a)], s);
                p.line_segment([c + vec2(-a, a), c + vec2(a, -a)], s);
            }
            PencereDugmesi::Buyut | PencereDugmesi::GeriAl => {
                let rect = egui::Rect::from_center_size(c, vec2(a * 2.0, a * 2.0));
                if tur == PencereDugmesi::GeriAl {
                    p.rect_stroke(
                        rect.translate(vec2(2.0, -2.0)),
                        0,
                        s,
                        egui::StrokeKind::Inside,
                    );
                }
                p.rect_stroke(rect, 0, s, egui::StrokeKind::Inside);
            }
            PencereDugmesi::Tasi => {
                for (x, y) in [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)] {
                    let uc = c + vec2(x, y) * (a + 1.0);
                    p.line_segment([c, uc], s);
                    for yon in [-1.0, 1.0] {
                        p.line_segment(
                            [
                                uc,
                                pos2(
                                    uc.x - x * 2.5 + y * yon * 2.5,
                                    uc.y - y * 2.5 + x * yon * 2.5,
                                ),
                            ],
                            s,
                        );
                    }
                }
            }
        }
    }
    r.on_hover_cursor(if tur == PencereDugmesi::Tasi {
        CursorIcon::Grab
    } else {
        CursorIcon::PointingHand
    })
}
