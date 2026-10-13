//! Bildirimsel işaretçiler (ADR 0001 §Saptama-c): araç bölgesini `.rol(Rol::..)` ile işaretler; çerçeve,
//! yazı/sarma kuralı ve `sabit_genislik` roldan gelir. Her işaretli bölge o karede deftere yazılır
//! (plan, `KILIM_ROL_IZ` bindirmesi, uyumsuzluk denetimi). Boşta yük sıfırdır: işaretçi `request_repaint`
//! çağırmaz, bindirme yalnız çizim yapar.

use egui::{
    Color32, Context, Frame, Id, InnerResponse, Panel, Rect, Stroke, Ui, UiBuilder, UiKind,
    UiStackInfo, Window,
};

use crate::rol::{Rol, RolStili};
use crate::yerlesim::sabit_genislik_bilgili;

const ETIKET: &str = "kilim-rol";
const DEFTER: &str = "kilim-tema-rol-defter";

type Duzen = Box<dyn FnOnce(&mut RolStili)>;

/// `.rol(..)` ile işaretlenebilen egui kapları: `Panel`, `CentralPanel`, `Window`, `Frame`.
pub trait RolVer: Sized {
    fn rol(self, rol: Rol) -> Rolu<Self> {
        Rolu {
            kap: self,
            rol,
            duzen: Vec::new(),
            cerceve: None,
        }
    }
}
impl RolVer for Panel {}
impl RolVer for egui::CentralPanel {}
impl RolVer for Window<'_> {}
impl RolVer for Frame {}

/// Rolü bağlanmış kap. `show*` çağrısıyla çizilir.
pub struct Rolu<K> {
    kap: K,
    rol: Rol,
    duzen: Vec<Duzen>,
    cerceve: Option<Frame>,
}

impl<K> Rolu<K> {
    /// Rolün kuralını bu bölge için ezer (araca özgü renk/yoğunluk).
    pub fn duzenle(mut self, f: impl FnOnce(&mut RolStili) + 'static) -> Self {
        self.duzen.push(Box::new(f));
        self
    }

    /// Aracın kendi çerçevesi: rol çerçevesinin yerine geçer (yazı/sarma/`sabit_genislik` kuralları sürer).
    pub fn frame(mut self, c: Frame) -> Self {
        self.cerceve = Some(c);
        self
    }

    fn stil(&mut self, ctx: &Context, koyu: bool) -> RolStili {
        let mut s = self.rol.stili_ctx(ctx, koyu);
        for f in self.duzen.drain(..) {
            f(&mut s);
        }
        s
    }
}

impl Rolu<Panel> {
    /// egui `Panel::show_separator_line` karşılığı (rolün ayırıcı kuralını ezer).
    pub fn show_separator_line(self, goster: bool) -> Self {
        self.duzenle(move |s| s.ayirici = goster)
    }

    pub fn show<R>(mut self, ui: &mut Ui, f: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let s = self.stil(ui.ctx(), ui.visuals().dark_mode);
        let rol = self.rol;
        let c = self.cerceve.unwrap_or_else(|| s.cerceve());
        self.kap
            .frame(c)
            .show_separator_line(s.ayirici)
            .show(ui, |ui| icerik(ui, rol, s, f))
    }

    /// `Panel::show_collapsible` karşılığı (kayma animasyonlu).
    pub fn show_collapsible<R>(
        mut self,
        ui: &mut Ui,
        acik: &mut bool,
        f: impl FnOnce(&mut Ui) -> R,
    ) -> Option<InnerResponse<R>> {
        let s = self.stil(ui.ctx(), ui.visuals().dark_mode);
        let rol = self.rol;
        let c = self.cerceve.unwrap_or_else(|| s.cerceve());
        self.kap
            .frame(c)
            .show_separator_line(s.ayirici)
            .show_collapsible(ui, acik, |ui| icerik(ui, rol, s, f))
    }
}

impl Rolu<egui::CentralPanel> {
    pub fn show<R>(mut self, ui: &mut Ui, f: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let s = self.stil(ui.ctx(), ui.visuals().dark_mode);
        let rol = self.rol;
        let c = self.cerceve.unwrap_or_else(|| s.cerceve());
        self.kap.frame(c).show(ui, |ui| icerik(ui, rol, s, f))
    }
}

impl Rolu<Window<'_>> {
    pub fn show<R>(
        mut self,
        ctx: &Context,
        f: impl FnOnce(&mut Ui) -> R,
    ) -> Option<InnerResponse<Option<R>>> {
        // Pencere kendi `Ui`'sini açmadan önce temayı bilmek için bağlamın temasına bakılır.
        let s = self.stil(ctx, ctx.global_style().visuals.dark_mode);
        let rol = self.rol;
        let c = self.cerceve.unwrap_or_else(|| s.cerceve());
        self.kap.frame(c).show(ctx, |ui| icerik(ui, rol, s, f))
    }
}

impl Rolu<Frame> {
    pub fn show<R>(mut self, ui: &mut Ui, f: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let s = self.stil(ui.ctx(), ui.visuals().dark_mode);
        let rol = self.rol;
        let c = self.cerceve.unwrap_or_else(|| s.cerceve());
        c.show(ui, |ui| icerik(ui, rol, s, f))
    }
}

/// Bölge içeriği: yazı/sarma kuralı, rol etiketi (alt `Ui`'ler `bagil_rol` ile sorar), gerekirse
/// `sabit_genislik`, ve deftere kayıt.
fn icerik<R>(ui: &mut Ui, rol: Rol, s: RolStili, f: impl FnOnce(&mut Ui) -> R) -> R {
    s.icerigi_uygula(ui);
    let bilgi = UiStackInfo::default().with_tag_value(ETIKET, rol);
    let r = if s.sabit_genislik {
        sabit_genislik_bilgili(ui, bilgi, f)
    } else {
        ui.scope_builder(UiBuilder::new().ui_stack_info(bilgi), f)
            .inner
    };
    kaydet(ui, rol, &s);
    r
}

/// `ui`'nin içinde bulunduğu en yakın işaretli bölgenin rolü (üst `Ui`'ler zinciri: `UiStack` etiketi).
/// İşaretli bölge yoksa `None`: bağlam duyarlı bileşenler (kunye-egui gibi) bununla davranışını seçer.
pub fn bagil_rol(ui: &Ui) -> Option<Rol> {
    ui.stack()
        .iter()
        .find_map(|n| n.tags().get_downcast::<Rol>(ETIKET).copied())
}

// ------------------------------------------------------------------ defter (plan)

/// Bir karede işaretlenmiş bölgenin kaydı.
#[derive(Clone, Debug, PartialEq)]
pub struct Kayit {
    pub rol: Rol,
    pub rect: Rect,
    /// Çevreleyen egui kabının türü (panel/pencere/açılır).
    pub kap: Option<UiKind>,
    /// Rol beklenen kap türünde mi (`YanPanel` bir `TopPanel`'de değil)?
    pub uyumlu: bool,
    pub sabit_genislik: bool,
    pub dolgu: Color32,
}

#[derive(Clone, Default)]
struct Defter {
    pass: u64,
    kayitlar: Vec<Kayit>,
}

/// Rolün hangi egui kaplarında anlamlı olduğu (boş: her yerde).
fn beklenen(rol: Rol) -> &'static [UiKind] {
    match rol {
        Rol::YanPanel => &[UiKind::LeftPanel, UiKind::RightPanel],
        Rol::UstCubuk => &[UiKind::TopPanel],
        Rol::DurumCubugu => &[UiKind::BottomPanel],
        Rol::Zemin => &[UiKind::CentralPanel],
        Rol::Pencere => &[UiKind::Window, UiKind::Modal],
        _ => &[],
    }
}

fn kap_turu(ui: &Ui) -> Option<UiKind> {
    ui.stack().iter().find_map(|n| {
        n.kind().filter(|k| {
            !matches!(
                k,
                UiKind::Frame | UiKind::ScrollArea | UiKind::Resize | UiKind::Collapsible
            )
        })
    })
}

fn kaydet(ui: &Ui, rol: Rol, s: &RolStili) {
    let kap = kap_turu(ui);
    let bekle = beklenen(rol);
    let kayit = Kayit {
        rol,
        rect: ui.max_rect(),
        kap,
        uyumlu: bekle.is_empty() || kap.is_some_and(|k| bekle.contains(&k)),
        sabit_genislik: s.sabit_genislik,
        dolgu: s.dolgu,
    };
    let ctx = ui.ctx();
    let pass = ctx.cumulative_pass_nr(); // `data_mut` içinde bağlam kilitli: dışarıda okunur
    ctx.data_mut(|d| {
        let df = d.get_temp_mut_or_default::<Defter>(Id::new(DEFTER));
        if df.pass != pass {
            df.pass = pass;
            df.kayitlar.clear();
        }
        df.kayitlar.push(kayit);
    });
}

/// Son çizilen karede işaretlenmiş bölgeler (çizim sırasıyla).
pub fn rol_kayitlari(ctx: &Context) -> Vec<Kayit> {
    ctx.data(|d| d.get_temp::<Defter>(Id::new(DEFTER)))
        .map(|d| d.kayitlar)
        .unwrap_or_default()
}

/// Kayıtlardan düz metin yerleşim planı (çalışma zamanı): rol, rect, kap türü, uyum.
pub fn plan_metni(ctx: &Context) -> String {
    let mut s = String::new();
    for k in rol_kayitlari(ctx) {
        let r = k.rect;
        s.push_str(&format!(
            "{:<14} {:>5.0},{:<5.0} {:>5.0}x{:<5.0} kap={:<11} {}{}\n",
            k.rol.ad(),
            r.min.x,
            r.min.y,
            r.width(),
            r.height(),
            k.kap.map_or("-".to_string(), |t| format!("{t:?}")),
            if k.sabit_genislik {
                "sabit_genislik "
            } else {
                ""
            },
            if k.uyumlu {
                ""
            } else {
                "UYUMSUZ: rol bu kap için değil"
            },
        ));
    }
    s
}

// ------------------------------------------------------------------ KILIM_ROL_IZ bindirmesi

/// Her işaretli bölgeyi rol rengiyle çerçeveler ve adını yazar (`KILIM_ROL_IZ=1`, yalnız hata ayıklama).
/// Uyumsuz roller kırmızı ve ünlemlidir. Yalnız çizer; yeniden çizim istemez.
pub struct RolIzi;

impl egui::Plugin for RolIzi {
    fn debug_name(&self) -> &'static str {
        "kilim-rol-izi"
    }

    fn on_end_pass(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        let boyaci = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Debug,
            Id::new("kilim-rol-izi"),
        ));
        for k in rol_kayitlari(&ctx) {
            let renk = if k.uyumlu {
                let i = Rol::HEPSI.iter().position(|r| *r == k.rol).unwrap_or(0);
                Color32::from(egui::ecolor::Hsva::new(i as f32 / 20.0, 0.8, 0.9, 1.0))
            } else {
                Color32::RED
            };
            boyaci.rect_stroke(
                k.rect,
                0.0,
                Stroke::new(2.0, renk),
                egui::StrokeKind::Inside,
            );
            let ad = if k.uyumlu {
                k.rol.ad().to_string()
            } else {
                format!("{} !", k.rol.ad())
            };
            boyaci.text(
                k.rect.min + egui::vec2(3.0, 1.0),
                egui::Align2::LEFT_TOP,
                ad,
                egui::FontId::monospace(10.0),
                renk,
            );
        }
    }
}

/// Bindirmeyi bu bağlama ekler (bir kez; tekrar çağrı zararsızdır).
pub fn rol_izi_ekle(ctx: &Context) {
    ctx.add_plugin(RolIzi);
}

/// Tek çağrıyla kurulum: iki tema + sarma + tercih (`uygula`) + (hata ayıklamada `KILIM_ROL_IZ=1` ise)
/// rol bindirmesi. Varsayılan hiçbir şey yapmayan araç bunu açılışta bir kez çağırır; sonra yalnız
/// bölgelerini işaretler (`.rol(..)`). `KILIM_TASMA_IZ` için bkz. [`crate::sarma_kur`].
pub fn giydir(ctx: &Context, tercih: &crate::TemaTercihi) {
    crate::iki_temayi_kur(ctx);
    crate::sarma_kur(ctx);
    tercih.uygula(ctx);
    #[cfg(debug_assertions)]
    if std::env::var_os("KILIM_ROL_IZ").is_some() {
        rol_izi_ekle(ctx);
    }
}
