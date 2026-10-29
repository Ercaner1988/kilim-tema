//! Anlamsal roller ve çizim kuralları (ADR 0001). TEK KAYNAK: Kilim paleti (`acik()`/`koyu()`), etkin
//! kenarlık varyantı ve görünüm. Hiçbir rol rengi burada sabit yazılmaz; hepsi paletten türer, o yüzden
//! tema paketi (`TemaPaketi`) roller için de geçerlidir. Her rolün metni, kendi dolgusu üstünde WCAG AA
//! (4.5:1) okunur olacak biçimde türetilir (sınama `tests/rol.rs`).

use std::sync::Arc;

use egui::{
    Color32, Context, CornerRadius, Frame, Id, Margin, Shadow, Stroke, TextStyle, TextWrapMode, Ui,
};

use crate::{karsitlik_ayarla, okunur_metin, yazi_rengi_ayarla, Gorunum, Kilim, Varyant};

/// Bir arayüz bölgesinin ya da öğesinin anlamı. Ad, işaretçide (`.rol(Rol::YanPanel)`) ve durağan
/// tarayıcının (`kilim-tarama`) önerisinde aynıdır.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Rol {
    /// Pencerenin ana zemini ("arka plan"): `CentralPanel`.
    Zemin,
    /// Zeminden ayrışan yükseltilmiş yüzey ("fon"): yazma alanı, okuma yüzeyi.
    Yuzey,
    YanPanel,
    /// Menü, araç ya da şerit çubuğu.
    UstCubuk,
    DurumCubugu,
    /// Çerçeveli grup (`Frame`).
    Kart,
    /// `Window` / `Modal`.
    Pencere,
    /// Açılır menü, ipucu, pop-up.
    Acilir,
    /// Metin kutusu çevresi.
    Giris,
    /// Seçilebilir liste satırı (seçili durum dolgusu).
    ListeSatiri,
    /// Tablo (şeritli satır dolgusu).
    Tablo,
    Sekme,
    /// Küçük etiket/çip.
    Rozet,
    DugmeBirincil,
    DugmeIkincil,
    DugmeTehlikeli,
    /// Vurgu rengiyle yazı.
    Vurgu,
    Hata,
    Uyari,
    Basari,
}

impl Rol {
    pub const HEPSI: [Rol; 20] = [
        Rol::Zemin,
        Rol::Yuzey,
        Rol::YanPanel,
        Rol::UstCubuk,
        Rol::DurumCubugu,
        Rol::Kart,
        Rol::Pencere,
        Rol::Acilir,
        Rol::Giris,
        Rol::ListeSatiri,
        Rol::Tablo,
        Rol::Sekme,
        Rol::Rozet,
        Rol::DugmeBirincil,
        Rol::DugmeIkincil,
        Rol::DugmeTehlikeli,
        Rol::Vurgu,
        Rol::Hata,
        Rol::Uyari,
        Rol::Basari,
    ];

    /// Rust adı (işaretçi ve tarayıcıyla aynı).
    pub fn ad(self) -> &'static str {
        match self {
            Rol::Zemin => "Zemin",
            Rol::Yuzey => "Yuzey",
            Rol::YanPanel => "YanPanel",
            Rol::UstCubuk => "UstCubuk",
            Rol::DurumCubugu => "DurumCubugu",
            Rol::Kart => "Kart",
            Rol::Pencere => "Pencere",
            Rol::Acilir => "Acilir",
            Rol::Giris => "Giris",
            Rol::ListeSatiri => "ListeSatiri",
            Rol::Tablo => "Tablo",
            Rol::Sekme => "Sekme",
            Rol::Rozet => "Rozet",
            Rol::DugmeBirincil => "DugmeBirincil",
            Rol::DugmeIkincil => "DugmeIkincil",
            Rol::DugmeTehlikeli => "DugmeTehlikeli",
            Rol::Vurgu => "Vurgu",
            Rol::Hata => "Hata",
            Rol::Uyari => "Uyari",
            Rol::Basari => "Basari",
        }
    }

    /// Bu rol yazı rengi mi (dolgusu yok, metin rengi taşır)?
    pub fn yazi_rengi_mi(self) -> bool {
        matches!(self, Rol::Vurgu | Rol::Hata | Rol::Uyari | Rol::Basari)
    }
}

/// Bir rolün çizim kuralı. Alanlar herkese açık: araç kendi rengini/yoğunluğunu ezebilir
/// (`Rolu::duzenle`, [`rol_ezme`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RolStili {
    pub dolgu: Color32,
    pub cizgi: Stroke,
    pub kose: u8,
    pub ic_bosluk: Margin,
    pub golge: bool,
    /// `None`: temanın metin rengi.
    pub metin: Option<Color32>,
    /// `TextStyle::Small` (durum çubuğu, rozet).
    pub kucuk_yazi: bool,
    /// Metin sarma kipi (`None`: üst `Ui`'nin kipi, yani `sarma_kur`).
    pub sarma: Option<TextWrapMode>,
    /// Panel içeriği [`crate::sabit_genislik`] içinde çizilir.
    pub sabit_genislik: bool,
    /// Panelin ayırıcı çizgisi (egui `show_separator_line`).
    pub ayirici: bool,
}

// ------------------------------------------------------------------ kural tablosu

fn yari(c: Color32, kilim_gorunumu: bool, a: f32) -> Color32 {
    if kilim_gorunumu {
        c.gamma_multiply(a)
    } else {
        c
    }
}

impl Rol {
    /// Saf kural tablosu (bağlam yok): paletten, açık/koyu bayrağından, varyanttan ve görünümden.
    /// Kilim görünümünde panel dolguları yarı saydamdır (kâğıt lifi görünür); sadede opaktır.
    pub fn stili_hesapla(
        self,
        k: &Kilim,
        koyu: bool,
        v: Varyant,
        kilim_gorunumu: bool,
    ) -> RolStili {
        let sade = RolStili {
            dolgu: Color32::TRANSPARENT,
            cizgi: Stroke::NONE,
            kose: 0,
            ic_bosluk: Margin::symmetric(8, 6),
            golge: false,
            metin: None,
            kucuk_yazi: false,
            sarma: None,
            sabit_genislik: false,
            ayirici: false,
        };
        let ince = Stroke::new(1.0, k.cizgi);
        let belirgin = Stroke::new(1.0, k.cizgi_koyu);
        let vurgu = v.vurgu(koyu);
        // Yarı saydam dolgunun yazısı, dolgunun yüzey üstündeki görünür rengine göre seçilir.
        let ton = |c: Color32| RolStili {
            metin: Some(okunur_metin(k.yuzey.blend(c), k)),
            dolgu: c,
            ..sade
        };
        match self {
            Rol::Zemin => RolStili {
                dolgu: if kilim_gorunumu {
                    Color32::TRANSPARENT
                } else {
                    k.zem
                },
                ic_bosluk: Margin::same(8),
                ..sade
            },
            Rol::Yuzey => RolStili {
                dolgu: yari(k.yuzey, kilim_gorunumu, 0.85),
                cizgi: ince,
                kose: 4,
                ..sade
            },
            Rol::YanPanel => RolStili {
                dolgu: yari(k.yuzey, kilim_gorunumu, 0.78),
                sarma: Some(TextWrapMode::Wrap),
                sabit_genislik: true,
                ayirici: true,
                ..sade
            },
            Rol::UstCubuk => RolStili {
                dolgu: yari(k.yuzey, kilim_gorunumu, 0.85),
                ic_bosluk: Margin::symmetric(8, 2),
                sarma: Some(TextWrapMode::Wrap),
                ayirici: true,
                ..sade
            },
            Rol::DurumCubugu => RolStili {
                dolgu: yari(k.zem, kilim_gorunumu, 0.9),
                ic_bosluk: Margin::symmetric(8, 2),
                metin: Some(karsitlik_ayarla(k.soluk, k.zem, 4.5, k)),
                kucuk_yazi: true,
                sarma: Some(TextWrapMode::Truncate),
                ayirici: true,
                ..sade
            },
            Rol::Kart => RolStili {
                dolgu: yari(k.yuzey, kilim_gorunumu, 0.9),
                cizgi: ince,
                kose: 6,
                ic_bosluk: Margin::same(8),
                ..sade
            },
            Rol::Pencere => RolStili {
                dolgu: k.yuzey,
                cizgi: belirgin,
                kose: 8,
                ic_bosluk: Margin::same(8),
                golge: true,
                ..sade
            },
            Rol::Acilir => RolStili {
                dolgu: k.yuzey,
                cizgi: belirgin,
                kose: 6,
                ic_bosluk: Margin::same(6),
                golge: true,
                ..sade
            },
            Rol::Giris => RolStili {
                dolgu: k.giris_zemini(koyu),
                cizgi: belirgin,
                kose: 4,
                ic_bosluk: Margin::symmetric(6, 4),
                ..sade
            },
            Rol::ListeSatiri => RolStili {
                kose: 3,
                ic_bosluk: Margin::symmetric(6, 2),
                ..ton(vurgu.gamma_multiply(0.35))
            },
            Rol::Tablo => ton(k.cizgi.gamma_multiply(0.35)),
            Rol::Sekme => RolStili {
                kose: 4,
                ic_bosluk: Margin::symmetric(10, 4),
                ..ton(vurgu.gamma_multiply(0.35))
            },
            Rol::Rozet => RolStili {
                kose: 255,
                ic_bosluk: Margin::symmetric(8, 1),
                kucuk_yazi: true,
                ..ton(k.cizgi)
            },
            Rol::DugmeBirincil => RolStili {
                kose: 4,
                ic_bosluk: Margin::symmetric(10, 4),
                ..ton(vurgu)
            },
            Rol::DugmeIkincil => RolStili {
                cizgi: belirgin,
                kose: 4,
                ic_bosluk: Margin::symmetric(10, 4),
                ..ton(k.yuzey)
            },
            Rol::DugmeTehlikeli => RolStili {
                kose: 4,
                ic_bosluk: Margin::symmetric(10, 4),
                ..ton(k.kirmizi)
            },
            Rol::Vurgu => RolStili {
                metin: Some(yazi_rengi_ayarla(vurgu, k)),
                ..sade
            },
            Rol::Hata => RolStili {
                metin: Some(yazi_rengi_ayarla(k.kirmizi, k)),
                ..sade
            },
            Rol::Uyari => RolStili {
                metin: Some(yazi_rengi_ayarla(k.sari, k)),
                ..sade
            },
            Rol::Basari => RolStili {
                metin: Some(yazi_rengi_ayarla(k.yesil, k)),
                ..sade
            },
        }
    }

    /// Bağlamdan (etkin tema paketi, varyant, görünüm, açık/koyu) türetilmiş kural; araç ezmeleri uygulanır.
    pub fn stili(self, ui: &Ui) -> RolStili {
        self.stili_ctx(ui.ctx(), ui.visuals().dark_mode)
    }

    pub fn stili_ctx(self, ctx: &Context, koyu: bool) -> RolStili {
        let k = if koyu { crate::koyu() } else { crate::acik() };
        let kilim = etkin_gorunum(ctx) == Gorunum::Kilim;
        let mut s = self.stili_hesapla(&k, koyu, crate::etkin_varyant(ctx), kilim);
        if let Some(ezme) = ctx.data(|d| d.get_temp::<Ezmeler>(Id::unique(EZME))) {
            for (rol, f) in ezme.0.iter() {
                if *rol == self {
                    f(&mut s);
                }
            }
        }
        s
    }
}

impl RolStili {
    /// Bölge çerçevesi (`Panel::frame`, `Window::frame`, `Frame`).
    pub fn cerceve(&self) -> Frame {
        Frame::new()
            .fill(self.dolgu)
            .stroke(self.cizgi)
            .corner_radius(CornerRadius::same(self.kose))
            .inner_margin(self.ic_bosluk)
            .shadow(if self.golge {
                Shadow {
                    offset: [0, 4],
                    blur: 12,
                    spread: 0,
                    color: Color32::from_black_alpha(60),
                }
            } else {
                Shadow::NONE
            })
    }

    /// Bölgenin içindeki `Ui`'ye yazı/sarma kurallarını uygular (yalnız bu `Ui` ve çocukları).
    pub fn icerigi_uygula(&self, ui: &mut Ui) {
        if let Some(m) = self.metin {
            ui.visuals_mut().override_text_color = Some(m);
        }
        if self.kucuk_yazi {
            ui.style_mut().override_text_style = Some(TextStyle::Small);
        }
        if self.sarma.is_some() {
            ui.style_mut().wrap_mode = self.sarma;
        }
    }
}

// ------------------------------------------------------------------ araç ezmeleri ve bağlam

type EzmeFn = Arc<dyn Fn(&mut RolStili) + Send + Sync>;

#[derive(Clone, Default)]
struct Ezmeler(Arc<Vec<(Rol, EzmeFn)>>);

const EZME: &str = "kilim-tema-rol-ezme";
const GORUNUM: &str = "kilim-tema-gorunum";

/// Aracın bir rolün kuralını bu `Context`'te kalıcı ezmesi (örn. durum çubuğu kendi rengini ister).
/// Aynı rol için sonradan eklenen önceki ezmelerin üstüne biner.
pub fn rol_ezme(ctx: &Context, rol: Rol, f: impl Fn(&mut RolStili) + Send + Sync + 'static) {
    ctx.data_mut(|d| {
        let id = Id::unique(EZME);
        let mut e: Vec<(Rol, EzmeFn)> = d
            .get_temp::<Ezmeler>(id)
            .map_or_else(Vec::new, |e| e.0.to_vec());
        e.push((rol, Arc::new(f)));
        d.insert_temp(id, Ezmeler(Arc::new(e)));
    });
}

/// `TemaTercihi::uygula`nın son kurduğu görünüm. Hiç kurulmadıysa [`Gorunum::Sade`] (palet düz yüzey;
/// kâğıt yalnız `cerceve` ile çizildiğinde vardır).
pub fn etkin_gorunum(ctx: &Context) -> Gorunum {
    ctx.data(|d| d.get_temp(Id::unique(GORUNUM)))
        .unwrap_or(Gorunum::Sade)
}

pub(crate) fn gorunumu_kaydet(ctx: &Context, g: Gorunum) {
    ctx.data_mut(|d| d.insert_temp(Id::unique(GORUNUM), g));
}

/// Yazı rolünün (Vurgu/Hata/Uyari/Basari) rengi, `ui`'nin GERÇEK arka planına göre okunur ayarlanmış:
/// `ui.stack().bg_color()` üst çerçevelerin dolgusunu harmanlar (saydam panel kâğıt tonuna oturtulur).
pub fn rol_rengi(ui: &Ui, rol: Rol) -> Color32 {
    let koyu = ui.visuals().dark_mode;
    let k = if koyu { crate::koyu() } else { crate::acik() };
    let s = rol.stili(ui);
    let taban = s.metin.unwrap_or(k.murekkep);
    if !rol.yazi_rengi_mi() {
        return taban;
    }
    let zemin = k.zem.blend(ui.stack().bg_color());
    let zemin = Color32::from_rgb(zemin.r(), zemin.g(), zemin.b());
    karsitlik_ayarla(taban, zemin, 4.5, &k)
}

/// Rollü düğme: dolgu, çizgi, köşe ve yazı rengi rolden gelir (`DugmeBirincil/Ikincil/Tehlikeli`).
pub fn dugme(ui: &Ui, rol: Rol, metin: impl Into<egui::WidgetText>) -> egui::Button<'static> {
    let s = rol.stili(ui);
    let yazi: egui::WidgetText = metin.into();
    let yazi = match s.metin {
        Some(m) => yazi.color(m),
        None => yazi,
    };
    egui::Button::new(yazi)
        .fill(s.dolgu)
        .stroke(s.cizgi)
        .corner_radius(CornerRadius::same(s.kose))
}
