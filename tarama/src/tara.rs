//! Sözcük dizisinden egui bölgelerini ve öğelerini bulur: kurucu çağrı + zincir + `.show(..)` + gövde.
//! Ağaç ilişkisi (hangi öğe hangi bölgenin gövdesinde) sözcük aralıklarından çıkar.

use crate::sozcuk::{ayir, Sozcuk, Tur as SozTur};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tur {
    PanelSol,
    PanelSag,
    PanelUst,
    PanelAlt,
    Merkez,
    Pencere,
    Cerceve,
    Izgara,
    Kaydirma,
    /// `ui.horizontal(..)` (sarmasız).
    Yatay,
    /// `ui.horizontal_wrapped(..)`.
    YatayDolgulu,
    /// kilim-tema yardımcıları: sabit_genislik, alan_satiri, birim, tablo.
    KilimYardimci,
    ComboBox,
    MetinKutusu,
    Dugme,
    SecimEtiketi,
    /// `Color32::RED`, `Color32::from_rgb(..)` ...
    Renk,
    /// `ui.visuals()` (tema renginden okuma).
    TemaRengi,
    /// kilim-tema'nın kurulum çağrıları (sarma_kur, giydir, iki_temayi_kur ...).
    Kurulum,
    /// `ui`'yi başka bir işleve geçiren çağrı (`panel::ciz(ui, ..)`, `self.liste(ui, ..)`): içerik orada çizilir.
    DisCagri,
}

impl Tur {
    pub fn govdeli(self) -> bool {
        use Tur::*;
        matches!(
            self,
            PanelSol
                | PanelSag
                | PanelUst
                | PanelAlt
                | Merkez
                | Pencere
                | Cerceve
                | Izgara
                | Kaydirma
                | Yatay
                | YatayDolgulu
                | KilimYardimci
                | ComboBox
        )
    }

    /// Rol taşıyabilen bölge.
    pub fn bolge_mi(self) -> bool {
        use Tur::*;
        matches!(
            self,
            PanelSol | PanelSag | PanelUst | PanelAlt | Merkez | Pencere | Cerceve
        )
    }

    pub fn ad(self) -> &'static str {
        use Tur::*;
        match self {
            PanelSol => "Panel::left",
            PanelSag => "Panel::right",
            PanelUst => "Panel::top",
            PanelAlt => "Panel::bottom",
            Merkez => "CentralPanel",
            Pencere => "Window",
            Cerceve => "Frame",
            Izgara => "Grid",
            Kaydirma => "ScrollArea",
            Yatay => "horizontal",
            YatayDolgulu => "horizontal_wrapped",
            KilimYardimci => "kilim_tema yardımcısı",
            ComboBox => "ComboBox",
            MetinKutusu => "TextEdit",
            Dugme => "düğme",
            SecimEtiketi => "seçim etiketi",
            Renk => "renk sabiti",
            TemaRengi => "ui.visuals()",
            Kurulum => "kurulum çağrısı",
            DisCagri => "dış çağrı",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Bulgu {
    pub tur: Tur,
    /// Kurucunun (ya da çağrının) konumu.
    pub satir: u32,
    pub sutun: u32,
    /// `Panel::left("cekmece")` → `cekmece`; ilk dize bağımsız değişkeni.
    pub ad: Option<String>,
    /// Kurucudan sonra zincirdeki yöntem adları (`resizable`, `rol`, `frame` ...).
    pub zincir: Vec<String>,
    /// `zincir` yöntemlerinin noktalarının (satır, sütun) konumu.
    pub zincir_konum: Vec<(u32, u32)>,
    /// Kurucunun ikinci yol parçası (`Frame::popup` → `popup`, `Helper::birim` → `birim`).
    pub kurucu: String,
    /// `.show(`'nun noktasının konumu (satır, sütun): işaretçi buraya eklenir.
    pub show: Option<(u32, u32)>,
    /// Gövdenin sözcük aralığı.
    pub govde: Option<(usize, usize)>,
    /// Serbest ek bilgi (renk sabitinin metni gibi).
    pub detay: String,
    /// Kurucunun sözcük indeksi.
    pub indeks: usize,
    /// İçinde bulunduğu en yakın gövdeli bölge (bulgu indeksi).
    pub ust: Option<usize>,
    /// Zincir `.show` ile bitmedi (kurucu bir değişkene bağlanıyor: `let z = Frame::new()...;`).
    pub tanim_mi: bool,
}

const SON_YONTEMLER: [&str; 6] = [
    "show",
    "show_inside",
    "show_collapsible",
    "show_animated",
    "show_animated_inside",
    "show_switched",
];

const RENK_ADLARI: [&str; 10] = [
    "RED",
    "WHITE",
    "BLACK",
    "GREEN",
    "BLUE",
    "YELLOW",
    "GRAY",
    "LIGHT_GRAY",
    "DARK_GRAY",
    "LIGHT_RED",
];

/// Zincirdeki bir yöntem: ad ve noktasının (satır, sütun) konumu.
type Yontem = (String, (u32, u32));

struct Tarayici {
    t: Vec<Sozcuk>,
    es: Vec<Option<usize>>,
}

/// Bir `fn`'in adı ve gövdesinin sözcük aralığı.
#[derive(Clone, Debug)]
pub struct Islev {
    pub ad: String,
    pub govde: (usize, usize),
}

pub struct Sonuc {
    pub bulgular: Vec<Bulgu>,
    pub islevler: Vec<Islev>,
}

pub fn tara(kaynak: &str) -> Vec<Bulgu> {
    tara_tam(kaynak).bulgular
}

pub fn tara_tam(kaynak: &str) -> Sonuc {
    let t = ayir(kaynak);
    let es = eslesmeler(&t);
    let z = Tarayici { t, es };
    let mut bulgular = z.bul();
    z.baglari_kur(&mut bulgular);
    Sonuc {
        islevler: z.islevler(),
        bulgular,
    }
}

fn eslesmeler(t: &[Sozcuk]) -> Vec<Option<usize>> {
    let mut es = vec![None; t.len()];
    let mut yigin: Vec<usize> = Vec::new();
    for (i, s) in t.iter().enumerate() {
        if s.tur != SozTur::Simge {
            continue;
        }
        match s.metin.as_str() {
            "(" | "{" | "[" => yigin.push(i),
            ")" | "}" | "]" => {
                if let Some(a) = yigin.pop() {
                    es[a] = Some(i);
                    es[i] = Some(a);
                }
            }
            _ => {}
        }
    }
    es
}

impl Tarayici {
    fn yol(&self, i: usize) -> Option<(Vec<&str>, usize)> {
        if self.t[i].tur != SozTur::Ad
            || (i > 0 && (self.t[i - 1].simge("::") || self.t[i - 1].simge(".")))
        {
            return None;
        }
        let (mut parcalar, mut j) = (vec![self.t[i].metin.as_str()], i + 1);
        while self.t.get(j).is_some_and(|x| x.simge("::"))
            && self.t.get(j + 1).is_some_and(|x| x.tur == SozTur::Ad)
        {
            parcalar.push(self.t[j + 1].metin.as_str());
            j += 2;
        }
        Some((parcalar, j))
    }

    fn bul(&self) -> Vec<Bulgu> {
        let t = &self.t;
        let mut cikti = Vec::new();
        let pasif = self.pasif_araliklar();
        let mut i = 0;
        while i < t.len() {
            if let Some(&(_, b)) = pasif.iter().find(|a| a.0 == i) {
                i = b + 1;
                continue;
            }
            if t[i].simge("(") && i > 0 && t[i - 1].tur == SozTur::Ad {
                if let Some(b) = self.dis_cagri(i) {
                    cikti.push(b);
                }
            }
            // Yöntem çağrısı: `.horizontal(`, `.button(` ...
            if t[i].simge(".")
                && t.get(i + 2).is_some_and(|x| x.simge("("))
                && t[i + 1].tur == SozTur::Ad
            {
                if let Some(b) = self.yontem(i) {
                    cikti.push(b);
                }
            } else if let Some((parca, son)) = self.yol(i) {
                if let Some(b) = self.kurucu(i, &parca, son) {
                    cikti.push(b);
                }
            }
            i += 1;
        }
        cikti
    }

    /// `#[cfg(test)] mod x { .. }` ve `#[test] fn f() { .. }` gövdeleri taranmaz.
    fn pasif_araliklar(&self) -> Vec<(usize, usize)> {
        let t = &self.t;
        let mut a = Vec::new();
        for i in 0..t.len().saturating_sub(4) {
            if !(t[i].simge("#") && t[i + 1].simge("[")) {
                continue;
            }
            let Some(kapa) = self.es[i + 1] else { continue };
            let oz: String = t[i + 2..kapa].iter().map(|s| s.metin.as_str()).collect();
            if oz == "test" || oz == "cfg(test)" {
                if let Some(ac) = (kapa + 1..t.len().min(kapa + 12)).find(|&k| t[k].simge("{")) {
                    if let Some(k) = self.es[ac] {
                        a.push((i, k));
                    }
                }
            }
        }
        a
    }

    /// `f(ui, ..)` biçimi: `ui` doğrudan bağımsız değişken. egui'nin ve kilim-tema'nın kendi çağrıları sayılmaz.
    fn dis_cagri(&self, i: usize) -> Option<Bulgu> {
        const KENDI: [&str; 24] = [
            "show",
            "show_inside",
            "show_ui",
            "show_collapsible",
            "show_animated",
            "ui",
            "scope",
            "add",
            "new",
            "sabit_genislik",
            "alan_satiri",
            "birim",
            "tablo",
            "secim",
            "kutu",
            "boncuklu",
            "boncuk_dugme",
            "tema_secici",
            "tema_secici_kaynakli",
            "cerceve",
            "allocate_ui",
            "interact",
            "paint_at",
            "fonts",
        ];
        let t = &self.t;
        let ad = t[i - 1].metin.as_str();
        let kapa = self.es[i]?;
        if KENDI.contains(&ad)
            || (i >= 3
                && t[i - 2].simge("::")
                && ["egui", "kilim_tema"].contains(&t[i - 3].metin.as_str()))
        {
            return None;
        }
        // Yalnız doğrudan (iç içe parantezde değil) bağımsız değişken: `(ui,` `, ui,` `, ui)`.
        let mut k = i + 1;
        let mut ui_arguman = false;
        while k < kapa {
            if t[k].simge("(") || t[k].simge("{") || t[k].simge("[") {
                k = self.es[k].unwrap_or(k) + 1;
                continue;
            }
            let onceki = t[k - 1].simge("(") || t[k - 1].simge(",");
            let sonraki = t.get(k + 1).is_some_and(|x| x.simge(",") || x.simge(")"));
            ui_arguman |= t[k].ad("ui") && onceki && sonraki;
            k += 1;
        }
        ui_arguman.then(|| {
            let mut b = self.yeni(Tur::DisCagri, i - 1, ad);
            b.ad = Some(ad.to_string());
            // `modul::ad(ui)` ise niteleyici işlev tanımını daraltır.
            if i >= 3 && t[i - 2].simge("::") {
                b.detay = t[i - 3].metin.clone();
            }
            b
        })
    }

    /// Her `fn ad(..) { .. }` için gövde aralığı.
    fn islevler(&self) -> Vec<Islev> {
        let t = &self.t;
        let mut v = Vec::new();
        for i in 0..t.len().saturating_sub(2) {
            if !(t[i].ad("fn")
                && t[i + 1].tur == SozTur::Ad
                && (t[i + 2].simge("(") || t[i + 2].simge("<")))
            {
                continue;
            }
            let mut k = i + 2;
            while k < t.len() && !t[k].simge("{") && !t[k].simge(";") {
                k = if t[k].simge("(") {
                    self.es[k].unwrap_or(k)
                } else {
                    k
                } + 1;
            }
            if let Some(e) = t.get(k).filter(|x| x.simge("{")).and(self.es[k]) {
                v.push(Islev {
                    ad: t[i + 1].metin.clone(),
                    govde: (k + 1, e),
                });
            }
        }
        v
    }

    fn ilk_dize(&self, ac: usize) -> Option<String> {
        let kapa = self.es[ac]?;
        let ilk = self.t.get(ac + 1)?;
        (ac + 1 < kapa && ilk.tur == SozTur::Dize).then(|| ilk.metin.clone())
    }

    /// Kurucudan sonraki `.yontem(..)` zinciri. Döner: (yöntem adları, `.show` noktası, `.show` bağımsız
    /// değişken aralığı).
    fn zincir(&self, mut j: usize) -> (Vec<Yontem>, Option<(usize, usize, usize)>) {
        let t = &self.t;
        let mut adlar = Vec::new();
        while t.get(j).is_some_and(|x| x.simge("."))
            && t.get(j + 1).is_some_and(|x| x.tur == SozTur::Ad)
        {
            let ad = t[j + 1].metin.as_str();
            let mut k = j + 2;
            if t.get(k).is_some_and(|x| x.simge("::")) && t.get(k + 1).is_some_and(|x| x.simge("<"))
            {
                let mut derin = 0;
                while k < t.len() {
                    if t[k].simge("<") {
                        derin += 1;
                    } else if t[k].simge(">") {
                        derin -= 1;
                        if derin == 0 {
                            k += 1;
                            break;
                        }
                    }
                    k += 1;
                }
            }
            if !t.get(k).is_some_and(|x| x.simge("(")) {
                break;
            }
            let Some(kapa) = self.es[k] else { break };
            if SON_YONTEMLER.contains(&ad) {
                return (adlar, Some((j, k + 1, kapa)));
            }
            adlar.push((ad.to_string(), (t[j].satir, t[j].sutun)));
            j = kapa + 1;
        }
        (adlar, None)
    }

    /// Bağımsız değişken aralığındaki `|..|` kapanışının gövdesi.
    fn govde(&self, bas: usize, son: usize) -> Option<(usize, usize)> {
        let t = &self.t;
        let mut k = bas;
        while k < son {
            if t[k].simge("(") || t[k].simge("{") || t[k].simge("[") {
                k = self.es[k].unwrap_or(k) + 1;
                continue;
            }
            if t[k].simge("|") {
                let mut p = k + 1;
                while p < son && !t[p].simge("|") {
                    p += 1;
                }
                let mut g = p + 1;
                if t.get(g).is_some_and(|x| x.simge("-")) {
                    while g < son && !t[g].simge("{") {
                        g += 1;
                    }
                }
                return match t.get(g) {
                    Some(x) if g < son && x.simge("{") => self.es[g].map(|e| (g + 1, e)),
                    _ => Some((g, son)),
                };
            }
            k += 1;
        }
        None
    }

    fn yeni(&self, tur: Tur, i: usize, kurucu: &str) -> Bulgu {
        Bulgu {
            tur,
            satir: self.t[i].satir,
            sutun: self.t[i].sutun,
            ad: None,
            zincir: Vec::new(),
            zincir_konum: Vec::new(),
            kurucu: kurucu.to_string(),
            show: None,
            govde: None,
            detay: String::new(),
            indeks: i,
            ust: None,
            tanim_mi: false,
        }
    }

    fn yontem(&self, i: usize) -> Option<Bulgu> {
        let ad = self.t[i + 1].metin.as_str();
        let tur = match ad {
            "horizontal" | "horizontal_top" | "horizontal_centered" => Tur::Yatay,
            "horizontal_wrapped" => Tur::YatayDolgulu,
            "button" | "small_button" => Tur::Dugme,
            "selectable_label" | "selectable_value" => Tur::SecimEtiketi,
            "text_edit_singleline" | "text_edit_multiline" => Tur::MetinKutusu,
            "visuals" => Tur::TemaRengi,
            "uygula" | "gorsel" => Tur::Kurulum,
            _ => return None,
        };
        let mut b = self.yeni(tur, i + 1, ad);
        if tur.govdeli() {
            let kapa = self.es[i + 2]?;
            b.govde = self.govde(i + 3, kapa);
        }
        Some(b)
    }

    fn kurucu(&self, i: usize, parca: &[&str], son: usize) -> Option<Bulgu> {
        let t = &self.t;
        let (ilk, sonu) = match parca {
            [.., a, b] => (*a, *b),
            [b] => ("", *b),
            [] => return None,
        };
        let cagri = t.get(son).is_some_and(|x| x.simge("("));
        // kilim_tema::... yolu
        if parca[0] == "kilim_tema"
            || (parca.len() == 1
                && ["sabit_genislik", "alan_satiri", "birim", "tablo"].contains(&sonu))
        {
            return self.kilim(i, sonu, son, cagri);
        }
        if ilk == "Color32" {
            let mut b = self.yeni(Tur::Renk, i, sonu);
            let sabit =
                RENK_ADLARI.contains(&sonu) || sonu.chars().all(|c| c.is_uppercase() || c == '_');
            if cagri && sonu.starts_with("from_") {
                let kapa = self.es[son]?;
                b.detay = t[son + 1..kapa]
                    .iter()
                    .map(|s| s.metin.as_str())
                    .collect::<String>();
            } else if !sabit || cagri {
                return None;
            }
            return Some(b);
        }
        let tur = match (ilk, sonu) {
            ("Panel" | "SidePanel", "left") => Tur::PanelSol,
            ("Panel" | "SidePanel", "right") => Tur::PanelSag,
            ("Panel" | "TopBottomPanel", "top") => Tur::PanelUst,
            ("Panel" | "TopBottomPanel", "bottom") => Tur::PanelAlt,
            ("CentralPanel", "default" | "no_frame") => Tur::Merkez,
            ("Window" | "Modal", "new") => Tur::Pencere,
            ("Frame", _) => Tur::Cerceve,
            ("Grid", "new") => Tur::Izgara,
            ("ScrollArea", "vertical" | "horizontal" | "both" | "neither") => Tur::Kaydirma,
            ("ComboBox", "from_id_salt" | "from_label" | "from_id_source" | "new") => Tur::ComboBox,
            ("TextEdit", "singleline" | "multiline" | "new") => Tur::MetinKutusu,
            ("Button", "selectable") => Tur::SecimEtiketi,
            ("Button", "new" | "image_and_text" | "image") => Tur::Dugme,
            _ => return None,
        };
        // `Frame::NONE` çağrı değil sabittir; öteki kurucular çağrıdır.
        if tur != Tur::Cerceve && !cagri {
            return None;
        }
        let mut b = self.yeni(tur, i, sonu);
        let (ad_i, zincir_basi) = if cagri {
            (Some(son), self.es[son]? + 1)
        } else {
            (None, son)
        };
        b.ad = ad_i.and_then(|a| self.ilk_dize(a));
        let (zincir, show) = self.zincir(zincir_basi);
        b.zincir = zincir.iter().map(|z| z.0.clone()).collect();
        b.zincir_konum = zincir.iter().map(|z| z.1).collect();
        if tur.govdeli() && tur != Tur::ComboBox {
            match show {
                Some((nokta, bas, kapa)) => {
                    b.show = Some((t[nokta].satir, t[nokta].sutun));
                    b.govde = self.govde(bas, kapa);
                }
                None => b.tanim_mi = true,
            }
        } else if tur == Tur::ComboBox {
            // `.show_ui(ui, |ui| ..)` ya da `.show_index(..)`: gövdedeki seçenekler üst bölgeye sayılmaz.
            if let Some(g) = self.combo_govdesi(zincir_basi) {
                b.govde = Some(g);
            }
        }
        Some(b)
    }

    fn combo_govdesi(&self, j: usize) -> Option<(usize, usize)> {
        let t = &self.t;
        let mut j = j;
        while t.get(j).is_some_and(|x| x.simge("."))
            && t.get(j + 1).is_some_and(|x| x.tur == SozTur::Ad)
        {
            let k = j + 2;
            let kapa = if t.get(k).is_some_and(|x| x.simge("(")) {
                self.es[k]?
            } else {
                return None;
            };
            if t[j + 1].metin == "show_ui" {
                return self.govde(k + 1, kapa);
            }
            j = kapa + 1;
        }
        None
    }

    fn kilim(&self, i: usize, ad: &str, son: usize, cagri: bool) -> Option<Bulgu> {
        let kurulum = [
            "sarma_kur",
            "giydir",
            "iki_temayi_kur",
            "cerceve",
            "tema_secici",
            "tema_secici_kaynakli",
        ];
        if ["sabit_genislik", "alan_satiri", "birim", "tablo"].contains(&ad) && cagri {
            let mut b = self.yeni(Tur::KilimYardimci, i, ad);
            let kapa = self.es[son]?;
            b.govde = self.govde(son + 1, kapa);
            return Some(b);
        }
        kurulum
            .contains(&ad)
            .then(|| self.yeni(Tur::Kurulum, i, ad))
    }

    /// Her bulgunun içinde bulunduğu en yakın gövdeli bulguyu bulur.
    fn baglari_kur(&self, b: &mut [Bulgu]) {
        let govdeler: Vec<(usize, usize, usize)> = b
            .iter()
            .enumerate()
            .filter_map(|(k, x)| x.govde.map(|(a, e)| (k, a, e)))
            .collect();
        for (k, bulgu) in b.iter_mut().enumerate() {
            let ix = bulgu.indeks;
            bulgu.ust = govdeler
                .iter()
                .filter(|&&(g, a, e)| g != k && a <= ix && ix < e)
                .min_by_key(|&&(_, a, e)| e - a)
                .map(|&(g, _, _)| g);
        }
    }
}

#[cfg(test)]
mod sinama {
    use super::*;

    fn turler(k: &str) -> Vec<(Tur, Option<String>)> {
        tara(k).into_iter().map(|b| (b.tur, b.ad)).collect()
    }

    #[test]
    fn panel_zinciri_ve_govde_icindeki_ogeler() {
        let b = tara(
            "fn f(ui: &mut Ui) {\n egui::Panel::left(\"cekmece\")\n  .resizable(true)\n  .show(ui, |ui| {\n   ui.horizontal(|ui| { ui.button(\"x\"); });\n  });\n}",
        );
        assert_eq!(b[0].tur, Tur::PanelSol);
        assert_eq!(b[0].ad.as_deref(), Some("cekmece"));
        assert_eq!(b[0].zincir, ["resizable"]);
        assert_eq!(b[0].show, Some((4, 3)));
        assert_eq!(b[1].tur, Tur::Yatay);
        assert_eq!(b[1].ust, Some(0));
        assert_eq!(b[2].tur, Tur::Dugme);
        assert_eq!(b[2].ust, Some(1), "düğme en yakın gövdeye (yatay) bağlanır");
    }

    #[test]
    fn dize_ve_yorumdaki_cagrilar_sayilmaz() {
        let b =
            turler("// Panel::left(\"a\").show(ui, |ui| {})\nlet s = \"Panel::top(\\\"b\\\")\";");
        assert!(b.is_empty(), "{b:?}");
    }

    #[test]
    fn test_modulu_taranmaz() {
        let b = turler("#[cfg(test)]\nmod s { fn f() { egui::Panel::top(\"x\").show(ui, |ui| {}); } }\nfn g() { egui::Panel::bottom(\"y\").show(ui, |ui| {}); }");
        assert_eq!(b, [(Tur::PanelAlt, Some("y".to_string()))]);
    }

    #[test]
    fn cerceve_tanimi_ile_gosterimi_ayrilir() {
        let b = tara("let z = Frame::new().fill(c).inner_margin(2.0);\nPanel::bottom(\"d\").frame(z).show(ui, |ui| {});\nFrame::NONE.show(ui, |ui| {});");
        assert!(b[0].tanim_mi && b[0].tur == Tur::Cerceve);
        assert_eq!(b[1].zincir, ["frame"]);
        assert!(b[2].show.is_some() && !b[2].tanim_mi);
    }

    #[test]
    fn combo_secenekleri_ust_bolgeye_baglanmaz() {
        let b = tara("Panel::left(\"s\").show(ui, |ui| { ComboBox::from_id_salt(1).show_ui(ui, |ui| { ui.selectable_value(a, b, \"x\"); }); });");
        let sec = b.iter().position(|x| x.tur == Tur::SecimEtiketi).unwrap();
        let combo = b.iter().position(|x| x.tur == Tur::ComboBox).unwrap();
        assert_eq!(b[sec].ust, Some(combo));
    }

    #[test]
    fn renk_sabiti_ve_literal() {
        let b = tara("ui.colored_label(egui::Color32::RED, h); let c = Color32::from_rgb(230, 180, 60); let d = Color32::from_rgb;");
        let r: Vec<_> = b
            .iter()
            .filter(|x| x.tur == Tur::Renk)
            .map(|x| x.detay.clone())
            .collect();
        assert_eq!(r, ["", "230,180,60"]);
    }
}
