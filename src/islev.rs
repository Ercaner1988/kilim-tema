//! İsteğe bağlı semantik işlev betiği. Komut çalıştırmaz; gerçek Response'u döndürür.
use crate::{Boncuk, Rol};
use egui::{Color32, Context, Id, Rect, Response, Shape, Stroke, Ui};
use serde::Deserialize;

#[derive(Clone, Copy, Debug)]
pub struct IslevTanimi {
    pub id: &'static str,
    pub rol: Rol,
    pub komut: Option<&'static str>,
}

#[derive(Clone, Debug)]
pub struct IslevKaydi {
    pub id: String,
    pub rol: Rol,
    pub komut: Option<String>,
    pub response_id: Id,
    pub rect: Rect,
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
    pub disabled: bool,
    pub clicked: bool,
}

#[derive(Clone, Default)]
struct Defter {
    kare: u64,
    kayitlar: Vec<IslevKaydi>,
}
const DEFTER: &str = "kilim/islev-defteri";
const BETIK: &str = "kilim/islev-betigi";

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct Renkler {
    dolgu: Option<String>,
    metin: Option<String>,
    cizgi: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Kural {
    rol: String,
    id: Option<String>,
    komut: Option<String>,
    #[serde(default)]
    normal: Renkler,
    #[serde(default)]
    hover: Renkler,
    #[serde(default)]
    pressed: Renkler,
    #[serde(default)]
    focus: Renkler,
    #[serde(default)]
    disabled: Renkler,
    #[serde(default)]
    selected: Renkler,
    doku: Option<String>,
}

/// Şema 1: bilinmeyen alan/ad reddedilir, son eşleşen kural öncekinin üstüne biner.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemaBetigi {
    sema: u32,
    kurallar: Vec<Kural>,
}

fn renk(s: &str) -> Result<Color32, String> {
    let h = s
        .strip_prefix('#')
        .ok_or_else(|| format!("Renk #RRGGBB veya #RRGGBBAA olmalı: {s}"))?;
    if !matches!(h.len(), 6 | 8) || !h.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("Geçersiz renk: {s}"));
    }
    let n = |i| u8::from_str_radix(&h[i..i + 2], 16).unwrap();
    Ok(Color32::from_rgba_unmultiplied(
        n(0),
        n(2),
        n(4),
        if h.len() == 8 { n(6) } else { 255 },
    ))
}

fn boncuk(s: &str) -> Option<Boncuk> {
    Some(match s.strip_prefix("boncuk:")? {
        "firuze" => Boncuk::Firuze,
        "akik" => Boncuk::Akik,
        "kehribar" => Boncuk::Kehribar,
        "lapis" => Boncuk::Lapis,
        "telkari" => Boncuk::Telkari,
        "sirma" => Boncuk::Sirma,
        "nazar" => Boncuk::Nazar,
        "tesbih" => Boncuk::Tesbih,
        "sedef" => Boncuk::Sedef,
        _ => return None,
    })
}

impl TemaBetigi {
    pub fn coz(metin: &str) -> Result<Self, String> {
        if metin.trim().is_empty() || metin.len() > 262_144 {
            return Err("Betik boş veya 256 KiB sınırını aşıyor".into());
        }
        let b: Self = serde_json::from_str(metin).map_err(|e| format!("Tema JSON: {e}"))?;
        b.dogrula()?;
        Ok(b)
    }
    fn dogrula(&self) -> Result<(), String> {
        if self.sema != 1 {
            return Err(format!("Desteklenmeyen tema şeması: {}", self.sema));
        }
        if self.kurallar.is_empty() || self.kurallar.len() > 512 {
            return Err("1–512 kural gerekli".into());
        }
        for k in &self.kurallar {
            if k.focus.dolgu.is_some() || k.focus.metin.is_some() {
                return Err("focus yalnız cizgi rengini destekler".into());
            }
            if !Rol::HEPSI.iter().any(|r| r.ad() == k.rol) {
                return Err(format!("Bilinmeyen rol: {}", k.rol));
            }
            for s in [&k.id, &k.komut].into_iter().flatten() {
                if s.trim().is_empty() || s.len() > 256 {
                    return Err("id/komut boş veya 256 bayttan uzun".into());
                }
            }
            if let Some(d) = &k.doku {
                if d != "kagit" && boncuk(d).is_none() {
                    return Err(format!("Bilinmeyen doku: {d}"));
                }
            }
            for r in [
                &k.normal,
                &k.hover,
                &k.pressed,
                &k.focus,
                &k.disabled,
                &k.selected,
            ] {
                for c in [&r.dolgu, &r.metin, &r.cizgi].into_iter().flatten() {
                    renk(c)?;
                }
            }
        }
        Ok(())
    }
    /// Yalnız doğrulanmış betik Context'e atomik olarak yazılır; global stil değişmez.
    pub fn kur(&self, ctx: &Context) -> Result<(), String> {
        self.dogrula()?;
        ctx.data_mut(|d| d.insert_temp(Id::new(BETIK), std::sync::Arc::new(self.clone())));
        ctx.request_repaint();
        Ok(())
    }
}

fn uygula(v: &mut egui::style::WidgetVisuals, r: &Renkler, k: &crate::Kilim) {
    if let Some(c) = &r.dolgu {
        v.bg_fill = renk(c).unwrap();
        v.weak_bg_fill = v.bg_fill;
        if r.metin.is_none() {
            v.fg_stroke.color = crate::okunur_metin(v.bg_fill, k);
        }
    }
    if let Some(c) = &r.metin {
        v.fg_stroke.color = renk(c).unwrap();
    }
    if let Some(c) = &r.cizgi {
        v.bg_stroke = Stroke::new(1.0, renk(c).unwrap());
    }
}

/// Closure kendi egui kimliğini korur; semantik id birden çok satırı seçebilir.
pub fn islev(ui: &mut Ui, tanim: IslevTanimi, ciz: impl FnOnce(&mut Ui) -> Response) -> Response {
    let initialized = ui.ctx().data_mut(|d| {
        let id = Id::new("kilim/islev-hook");
        let exists = d.get_temp::<bool>(id).unwrap_or(false);
        d.insert_temp(id, true);
        exists
    });
    if !initialized {
        ui.ctx().on_begin_pass(
            "kilim/islev-defteri",
            std::sync::Arc::new(|ui| {
                ui.ctx().data_mut(|d| d.remove::<Defter>(Id::new(DEFTER)));
            }),
        );
    }
    let betik = ui
        .ctx()
        .data(|d| d.get_temp::<std::sync::Arc<TemaBetigi>>(Id::new(BETIK)));
    let kurallar: Vec<_> = betik
        .as_ref()
        .map(|b| {
            b.kurallar
                .iter()
                .filter(|k| {
                    k.rol == tanim.rol.ad()
                        && k.id.as_deref().is_none_or(|id| id == tanim.id)
                        && k.komut.as_deref().is_none_or(|c| Some(c) == tanim.komut)
                })
                .collect()
        })
        .unwrap_or_default();
    let doku = kurallar
        .iter()
        .filter_map(|k| k.doku.as_deref())
        .next_back();
    let painter = ui.painter().clone();
    let yuzey = matches!(
        tanim.rol,
        Rol::Zemin
            | Rol::Yuzey
            | Rol::YanPanel
            | Rol::UstCubuk
            | Rol::DurumCubugu
            | Rol::Kart
            | Rol::Pencere
            | Rol::Acilir
    );
    let arka = (doku.is_some() || yuzey).then(|| painter.add(Shape::Noop));
    let enabled = ui.is_enabled();
    let k = if ui.visuals().dark_mode {
        crate::koyu()
    } else {
        crate::acik()
    };
    let rol_stili = tanim.rol.stili(ui);
    let (cevap, gorseller) = ui
        .scope(|ui| {
            {
                let v = ui.visuals_mut();
                for w in [
                    &mut v.widgets.noninteractive,
                    &mut v.widgets.inactive,
                    &mut v.widgets.hovered,
                    &mut v.widgets.active,
                ] {
                    w.bg_fill = rol_stili.dolgu;
                    w.weak_bg_fill = rol_stili.dolgu;
                    w.bg_stroke = rol_stili.cizgi;
                    w.corner_radius = egui::CornerRadius::same(rol_stili.kose);
                    if let Some(c) = rol_stili.metin {
                        w.fg_stroke.color = c;
                    }
                }
            }
            if !kurallar.is_empty() {
                let v = ui.visuals_mut();
                v.override_text_color = None;
                for kural in &kurallar {
                    for w in [
                        &mut v.widgets.noninteractive,
                        &mut v.widgets.inactive,
                        &mut v.widgets.hovered,
                        &mut v.widgets.active,
                    ] {
                        uygula(w, &kural.normal, &k);
                    }
                    uygula(&mut v.widgets.hovered, &kural.hover, &k);
                    uygula(&mut v.widgets.active, &kural.pressed, &k);
                    uygula(&mut v.widgets.noninteractive, &kural.disabled, &k);
                    if let Some(c) = &kural.selected.dolgu {
                        v.selection.bg_fill = renk(c).unwrap();
                    }
                    if let Some(c) = &kural.selected.metin {
                        v.selection.stroke.color = renk(c).unwrap();
                    }
                    if let Some(c) = &kural.selected.cizgi {
                        v.selection.stroke = Stroke::new(1.0, renk(c).unwrap());
                    }
                }
            }
            let g = ui.visuals().widgets.clone();
            if tanim.rol == Rol::Giris {
                let color = if enabled {
                    g.inactive.bg_fill
                } else {
                    g.noninteractive.bg_fill
                };
                ui.visuals_mut().text_edit_bg_color = Some(if doku.is_some() {
                    Color32::TRANSPARENT
                } else {
                    color
                });
            }
            if doku.is_some() {
                let v = ui.visuals_mut();
                for w in [
                    &mut v.widgets.noninteractive,
                    &mut v.widgets.inactive,
                    &mut v.widgets.hovered,
                    &mut v.widgets.active,
                ] {
                    w.bg_fill = Color32::TRANSPARENT;
                    w.weak_bg_fill = Color32::TRANSPARENT;
                }
            }
            (ciz(ui), g)
        })
        .inner;
    if let Some(index) = arka {
        let w = if !cevap.enabled() {
            gorseller.noninteractive
        } else if cevap.is_pointer_button_down_on() {
            gorseller.active
        } else if cevap.hovered() {
            gorseller.hovered
        } else {
            gorseller.inactive
        };
        if let Some(d) = doku {
            let ds = crate::dokular(ui.ctx());
            let (texture, uv) = if d == "kagit" {
                let karo = ds.kagit.size_vec2() * 0.5;
                (
                    ds.kagit.id(),
                    Rect::from_min_max(
                        (cevap.rect.min.to_vec2() / karo).to_pos2(),
                        (cevap.rect.max.to_vec2() / karo).to_pos2(),
                    ),
                )
            } else {
                (ds.boncuk(boncuk(d).unwrap()).id(), crate::UV_TAM)
            };
            painter.set(
                index,
                egui::epaint::RectShape::filled(cevap.rect, w.corner_radius, w.bg_fill)
                    .with_texture(texture, uv),
            );
        } else {
            painter.set(
                index,
                egui::epaint::RectShape::filled(cevap.rect, w.corner_radius, w.bg_fill),
            );
        }
    }
    if cevap.has_focus() {
        for kural in &kurallar {
            if let Some(c) = &kural.focus.cizgi {
                painter.rect_stroke(
                    cevap.rect,
                    0.0,
                    Stroke::new(2.0, renk(c).unwrap()),
                    egui::StrokeKind::Inside,
                );
            }
        }
    }
    let kayit = IslevKaydi {
        id: tanim.id.into(),
        rol: tanim.rol,
        komut: tanim.komut.map(str::to_owned),
        response_id: cevap.id,
        rect: cevap.rect,
        hovered: cevap.hovered(),
        pressed: cevap.is_pointer_button_down_on(),
        focused: cevap.has_focus(),
        disabled: !cevap.enabled(),
        clicked: cevap.clicked(),
    };
    let kare = ui.ctx().cumulative_frame_nr();
    ui.ctx().data_mut(|d| {
        let f = d.get_temp_mut_or_default::<Defter>(Id::new(DEFTER));
        if f.kare != kare {
            f.kare = kare;
            f.kayitlar.clear();
        }
        if let Some(eski) = f
            .kayitlar
            .iter_mut()
            .find(|k| k.id == kayit.id && k.response_id == kayit.response_id)
        {
            *eski = kayit;
        } else {
            f.kayitlar.push(kayit);
        }
    });
    cevap
}

/// Son çizim geçişinin kayıtları; sonraki boş geçişte defter temizlenir.
pub fn islev_kayitlari(ctx: &Context) -> Vec<IslevKaydi> {
    ctx.data(|d| d.get_temp::<Defter>(Id::new(DEFTER)))
        .map(|d| d.kayitlar)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tanim() -> IslevTanimi {
        IslevTanimi {
            id: "ara",
            rol: Rol::DugmeBirincil,
            komut: Some("ara"),
        }
    }
    fn betik() -> TemaBetigi {
        TemaBetigi::coz(r##"{"sema":1,"kurallar":[{"rol":"DugmeBirincil","normal":{"dolgu":"#123456"},"hover":{"dolgu":"#abcdef"},"disabled":{"dolgu":"#222222"}}]}"##).unwrap()
    }
    #[test]
    fn parser_hatalari_ve_kurulum_atomik() {
        let ctx = Context::default();
        betik().kur(&ctx).unwrap();
        for s in [
            "",
            "{}",
            r#"{"sema":2,"kurallar":[]}"#,
            r#"{"sema":1,"kurallar":[]}"#,
            r#"{"sema":1,"kurallar":[{"rol":"yok"}]}"#,
            r#"{"sema":1,"kurallar":[{"rol":"Zemin","doku":"yok"}]}"#,
            r#"{"sema":1,"kurallar":[{"rol":"Zemin","normal":{"dolgu":"red"}}]}"#,
            r##"{"sema":1,"kurallar":[{"rol":"Zemin","focus":{"metin":"#ffffff"}}]}"##,
            r#"{"sema":1,"kurallar":[{"rol":"Zemin","fazla":1}]}"#,
        ] {
            assert!(TemaBetigi::coz(s).is_err(), "{s}");
        }
        assert!(TemaBetigi::coz(&" ".repeat(262145)).is_err());
        assert!(ctx
            .data(|d| d.get_temp::<std::sync::Arc<TemaBetigi>>(Id::new(BETIK)))
            .is_some());
        assert!(Context::default()
            .data(|d| d.get_temp::<std::sync::Arc<TemaBetigi>>(Id::new(BETIK)))
            .is_none());
    }
    #[test]
    fn gercek_stil_ve_shape() {
        let ctx = Context::default();
        betik().kur(&ctx).unwrap();
        let mut out = ctx.run_ui(Default::default(), |ui| {
            islev(ui, tanim(), |ui| {
                assert_eq!(
                    ui.visuals().widgets.hovered.bg_fill,
                    Color32::from_rgb(171, 205, 239)
                );
                assert_eq!(
                    ui.visuals().widgets.noninteractive.bg_fill,
                    Color32::from_gray(34)
                );
                ui.button("Ara")
            });
        });
        out.textures_delta.clear();
        assert!(out
            .shapes
            .iter()
            .any(|s| matches!(&s.shape,Shape::Rect(r) if r.fill==Color32::from_rgb(18,52,86))));
    }
    #[test]
    fn defter_satirlar_nested_context_ve_bos_kare() {
        let ctx = Context::default();
        let mut out = ctx.run_ui(Default::default(), |ui| {
            let a = islev(ui, tanim(), |ui| ui.button("A"));
            let b = islev(ui, tanim(), |ui| ui.button("B"));
            assert_ne!(a.id, b.id);
            let c = islev(ui, tanim(), |ui| islev(ui, tanim(), |ui| ui.button("C")));
            assert_eq!(islev_kayitlari(ui.ctx()).len(), 3);
            assert_eq!(islev_kayitlari(ui.ctx())[2].response_id, c.id);
        });
        out.textures_delta.clear();
        assert_eq!(islev_kayitlari(&ctx).len(), 3);
        assert!(islev_kayitlari(&Context::default()).is_empty());
        let mut out = ctx.run_ui(Default::default(), |_| {});
        out.textures_delta.clear();
        assert!(islev_kayitlari(&ctx).is_empty());
    }
    #[test]
    fn doku_gercek_shape_ve_kimlik() {
        let ctx = Context::default();
        TemaBetigi::coz(r#"{"sema":1,"kurallar":[{"rol":"DugmeBirincil","doku":"kagit"}]}"#)
            .unwrap()
            .kur(&ctx)
            .unwrap();
        let mut id = None;
        let mut out = ctx.run_ui(Default::default(), |ui| {
            let r = islev(ui, tanim(), |ui| ui.button("Ara"));
            id = Some(r.id);
        });
        out.textures_delta.clear();
        let texture = crate::dokular(&ctx).kagit.id();
        assert!(out.shapes.iter().any(|s|matches!(&s.shape,Shape::Rect(r) if r.brush.as_ref().is_some_and(|b|b.fill_texture_id==texture))));
        let kayit = islev_kayitlari(&ctx).pop().unwrap();
        assert_eq!(Some(kayit.response_id), id);
        assert_eq!(kayit.komut.as_deref(), Some("ara"));
    }
    #[test]
    fn secili_widget_gercek_shape() {
        let ctx = Context::default();
        TemaBetigi::coz(
            r##"{"sema":1,"kurallar":[{"rol":"DugmeBirincil","selected":{"dolgu":"#ed1234"}}]}"##,
        )
        .unwrap()
        .kur(&ctx)
        .unwrap();
        let mut out = ctx.run_ui(Default::default(), |ui| {
            islev(ui, tanim(), |ui| ui.selectable_label(true, "Seçili"));
        });
        out.textures_delta.clear();
        assert!(out
            .shapes
            .iter()
            .any(|s| matches!(&s.shape,Shape::Rect(r) if r.fill==Color32::from_rgb(237,18,52))));
    }
    #[test]
    fn giris_ve_yuzey_gercek_dolgu() {
        for rol in [Rol::Giris, Rol::Kart] {
            let ctx = Context::default();
            let source = format!(
                r##"{{"sema":1,"kurallar":[{{"rol":"{}","normal":{{"dolgu":"#c12345"}}}}]}}"##,
                rol.ad()
            );
            TemaBetigi::coz(&source).unwrap().kur(&ctx).unwrap();
            let mut text = String::from("Metin");
            let mut out = ctx.run_ui(Default::default(), |ui| {
                let t = IslevTanimi { rol, ..tanim() };
                islev(ui, t, |ui| {
                    if rol == Rol::Giris {
                        ui.text_edit_singleline(&mut text)
                    } else {
                        ui.scope(|ui| {
                            ui.label("Kart");
                        })
                        .response
                    }
                });
            });
            out.textures_delta.clear();
            assert!(
                out.shapes.iter().any(
                    |s| matches!(&s.shape,Shape::Rect(r) if r.fill==Color32::from_rgb(193,35,69))
                ),
                "{}",
                rol.ad()
            );
        }
    }
    #[test]
    fn iki_context_ayni_islev_farkli_tema() {
        for hex in ["#aa1122", "#22aa11"] {
            let ctx = Context::default();
            let source = format!(
                r#"{{"sema":1,"kurallar":[{{"rol":"DugmeBirincil","normal":{{"dolgu":"{hex}"}}}}]}}"#
            );
            TemaBetigi::coz(&source).unwrap().kur(&ctx).unwrap();
            let mut out = ctx.run_ui(Default::default(), |ui| {
                islev(ui, tanim(), |ui| ui.button("Ara"));
            });
            out.textures_delta.clear();
            assert!(out
                .shapes
                .iter()
                .any(|s| matches!(&s.shape,Shape::Rect(r) if r.fill==renk(hex).unwrap())));
        }
    }
    #[test]
    fn hover_basili_click_komut_ve_pasif() {
        let ctx = Context::default();
        betik().kur(&ctx).unwrap();
        let mut out = ctx.run_ui(Default::default(), |ui| {
            islev(ui, tanim(), |ui| ui.button("Ara"));
        });
        out.textures_delta.clear();
        let pos = islev_kayitlari(&ctx)[0].rect.center();
        for pressed in [true, false] {
            let input = egui::RawInput {
                events: vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Default::default(),
                    },
                ],
                ..Default::default()
            };
            let mut out = ctx.run_ui(input, |ui| {
                let r = islev(ui, tanim(), |ui| ui.button("Ara"));
                assert_eq!(r.clicked(), !pressed);
            });
            out.textures_delta.clear();
            let kayit = islev_kayitlari(&ctx).pop().unwrap();
            assert_eq!(kayit.clicked, !pressed);
            assert_eq!(kayit.komut.as_deref(), Some("ara"));
        }
        let mut out = ctx.run_ui(Default::default(), |ui| {
            ui.add_enabled_ui(false, |ui| islev(ui, tanim(), |ui| ui.button("Ara")));
        });
        out.textures_delta.clear();
        assert!(islev_kayitlari(&ctx)[0].disabled);
    }
}
