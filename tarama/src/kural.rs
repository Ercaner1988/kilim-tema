//! Bulgulardan rol önerisi ve yerleşim kuralı önerileri çıkarır. Her öneri araca konacak somut bir
//! değişikliktir (dosya:satır, önce/sonra); araca yazan kişi tarayıcı değil araç sahibidir.

use std::collections::BTreeMap;

use crate::tara::{Bulgu, Islev, Tur};

/// kilim-tema `Rol` adları; sınama `kilim_tema::Rol::HEPSI` ile eşitliği doğrular.
pub const ROLLER: [&str; 20] = [
    "Zemin",
    "Yuzey",
    "YanPanel",
    "UstCubuk",
    "DurumCubugu",
    "Kart",
    "Pencere",
    "Acilir",
    "Giris",
    "ListeSatiri",
    "Tablo",
    "Sekme",
    "Rozet",
    "DugmeBirincil",
    "DugmeIkincil",
    "DugmeTehlikeli",
    "Vurgu",
    "Hata",
    "Uyari",
    "Basari",
];

pub struct Dosya {
    pub yol: String,
    pub satirlar: Vec<String>,
    pub bulgular: Vec<Bulgu>,
    pub islevler: Vec<Islev>,
    /// Dosyanın sandığı (en yakın `Cargo.toml`) `kilim-tema`ya bağlı mı? (`Dosya::oku` varsayılanı: evet)
    pub kilim_bagimli: bool,
}

impl Dosya {
    pub fn oku(yol: &str, kaynak: &str) -> Self {
        let s = crate::tara::tara_tam(kaynak);
        Self {
            yol: yol.replace('\\', "/"),
            satirlar: kaynak.lines().map(str::to_string).collect(),
            bulgular: s.bulgular,
            islevler: s.islevler,
            kilim_bagimli: true,
        }
    }

    fn satir(&self, n: u32) -> &str {
        self.satirlar.get(n as usize - 1).map_or("", |s| s.as_str())
    }

    /// `k` bölgesinin içeriği (öğe türü → sayı). ComboBox gövdesindeki seçenekler sayılmaz.
    pub fn icerik(&self, k: usize) -> BTreeMap<Tur, usize> {
        let mut say = BTreeMap::new();
        for (j, c) in self.bulgular.iter().enumerate() {
            let (mut x, mut icinde, mut combo_icinde) = (j, false, false);
            while let Some(u) = self.bulgular[x].ust {
                if u == k {
                    icinde = true;
                    break;
                }
                combo_icinde |= self.bulgular[u].tur == Tur::ComboBox;
                x = u;
            }
            if icinde
                && !combo_icinde
                && !matches!(c.tur, Tur::Renk | Tur::TemaRengi | Tur::Kurulum)
            {
                *say.entry(c.tur).or_insert(0) += 1;
            }
        }
        say
    }
}

#[derive(Clone, Debug)]
pub struct Oneri {
    pub kod: &'static str,
    pub yol: String,
    pub satir: u32,
    pub baslik: String,
    pub once: String,
    pub sonra: String,
    pub gerekce: &'static str,
}

/// Bölge türü + zincirden önerilen rol; yoksa bölge yalnız yerleşim çerçevesidir (rol gerekmez).
pub fn rol_oner(d: &Dosya, k: usize) -> Option<&'static str> {
    let b = &d.bulgular[k];
    Some(match b.tur {
        Tur::PanelSol | Tur::PanelSag => "YanPanel",
        Tur::PanelUst => "UstCubuk",
        // Alt çubukta metin kutusu varsa durum çubuğu değil, yazma alanıdır (sohbet girişi): fon.
        Tur::PanelAlt if d.icerik(k).contains_key(&Tur::MetinKutusu) => "Yuzey",
        Tur::PanelAlt => "DurumCubugu",
        Tur::Merkez => "Zemin",
        Tur::Pencere => "Pencere",
        Tur::Cerceve if b.kurucu == "popup" => "Acilir",
        Tur::Cerceve if matches!(b.kurucu.as_str(), "group" | "canvas") => "Kart",
        _ => return None,
    })
}

pub fn isaretli(b: &Bulgu) -> bool {
    b.zincir.iter().any(|z| z == "rol")
}

/// Rolün plan metnindeki kısa kararları (ayrıntı: kilim-tema `Rol::stili`).
pub fn rol_kararlari(rol: &str) -> &'static str {
    match rol {
        "YanPanel" => {
            "dolgu yuzey, kenar çizgisi, içerik sabit_genislik'te (şişme/daralma yok), sarma açık"
        }
        "UstCubuk" => "dolgu yuzey, alt çizgi, dar iç boşluk, düğme/giriş sıkı aralık",
        "DurumCubugu" => "dolgu çizgi tonu, küçük soluk yazı, tek satır",
        "Zemin" => "kâğıt görünümünde saydam (kâğıt görünür), sade görünümde zem",
        "Pencere" => "dolgu yuzey, belirgin çizgi, gölge, yuvarlak köşe",
        "Acilir" => "dolgu yuzey, belirgin çizgi, gölge",
        "Kart" => "dolgu yuzey, ince çizgi, iç boşluk",
        _ => "",
    }
}

fn renk_anlami(b: &Bulgu) -> Option<&'static str> {
    if !b.detay.is_empty() {
        let v: Vec<i32> = b
            .detay
            .split(',')
            .filter_map(|p| p.trim().trim_start_matches("0x").parse().ok())
            .collect();
        if let [r, g, bl, ..] = v[..] {
            return Some(if r >= 170 && g <= 110 && bl <= 110 {
                "Hata"
            } else if r >= 170 && g >= 110 && bl <= 100 {
                "Uyari"
            } else if g >= r + 30 && g >= bl + 20 {
                "Basari"
            } else {
                return None;
            });
        }
        return None;
    }
    match b.kurucu.as_str() {
        "RED" => Some("Hata"),
        "YELLOW" => Some("Uyari"),
        "GREEN" => Some("Basari"),
        _ => None,
    }
}

/// Anlamı olan renk için hangi kod kullanılmalı.
fn renk_onerisi(rol: &str) -> &'static str {
    match rol {
        "Hata" => "ui.visuals().error_fg_color",
        "Uyari" => "ui.visuals().warn_fg_color",
        _ => "kilim_tema::rol_rengi(ui, Rol::Basari)",
    }
}

/// Satırdaki `[egui::]Color32::<kurucu>[(..)]` ifadesini `yeni` ile değiştirir.
fn renk_degistir(satir: &str, kurucu: &str, yeni: &str) -> String {
    let anahtar = format!("Color32::{kurucu}");
    let Some(p) = satir.find(&anahtar) else {
        return satir.to_string();
    };
    let bas = if satir[..p].ends_with("egui::") {
        p - 6
    } else {
        p
    };
    let mut son = p + anahtar.len();
    if satir[son..].starts_with('(') {
        let mut derin = 0;
        for (i, c) in satir[son..].char_indices() {
            derin += i32::from(c == '(') - i32::from(c == ')');
            if derin == 0 {
                son += i + 1;
                break;
            }
        }
    }
    format!("{}{yeni}{}", &satir[..bas], &satir[son..])
}

fn araya_sok(satir: &str, sutun: u32, ek: &str) -> String {
    let mut s: Vec<char> = satir.chars().collect();
    let k = (sutun as usize - 1).min(s.len());
    s.splice(k..k, ek.chars());
    s.into_iter().collect()
}

pub fn oneriler(d: &Dosya) -> Vec<Oneri> {
    let mut o = Vec::new();
    for (k, b) in d.bulgular.iter().enumerate() {
        match b.tur {
            t if t.bolge_mi() => rol_onerisi(d, k, &mut o),
            Tur::Izgara => izgara(d, b, &mut o),
            Tur::Yatay => yatay(d, k, b, &mut o),
            Tur::ComboBox => combo(d, k, b, &mut o),
            Tur::Renk => renk(d, b, &mut o),
            _ => {}
        }
    }
    o
}

fn yeni(
    d: &Dosya,
    kod: &'static str,
    b: &Bulgu,
    baslik: String,
    sonra: String,
    gerekce: &'static str,
) -> Oneri {
    Oneri {
        kod,
        yol: d.yol.clone(),
        satir: b.satir,
        baslik,
        once: d.satir(b.satir).trim().to_string(),
        sonra,
        gerekce,
    }
}

fn rol_onerisi(d: &Dosya, k: usize, o: &mut Vec<Oneri>) {
    let b = &d.bulgular[k];
    let Some(rol) = rol_oner(d, k) else { return };
    let Some(show) = b.show else { return };
    if isaretli(b) {
        return;
    }
    // Aracın kendi `.frame(..)`'i varsa `.rol(..)` ondan ÖNCE girer (Rolu::frame onu sürdürür); aksi hâlde
    // `.frame` rolün çerçevesiyle ezilirdi. `.frame`den sonraki yöntemlerden yalnız `show_separator_line`
    // Rolu'da vardır; ötekileri `.rol(..)`dan önceye taşımak gerekir.
    let frame_i = b.zincir.iter().position(|z| z == "frame");
    let (sl, sc) = frame_i.map_or(show, |i| b.zincir_konum[i]);
    let tasinacak: Vec<&str> = frame_i.map_or(Vec::new(), |i| {
        b.zincir[i + 1..]
            .iter()
            .map(String::as_str)
            .filter(|z| *z != "show_separator_line")
            .collect()
    });
    let ad =
        b.ad.as_deref()
            .map_or(String::new(), |a| format!("(\"{a}\")"));
    let mut oz = yeni(
        d,
        "ROL",
        b,
        format!("{}{} -> Rol::{rol}", b.tur.ad(), ad),
        String::new(),
        "çerçeve/yerleşim kararı rolden gelir; işaretsiz bölgeyi çalışma zamanı da bulamaz",
    );
    oz.satir = sl;
    oz.once = d.satir(sl).trim().to_string();
    // `CentralPanel::no_frame()` aracın bilerek çerçevesiz isteğidir: rolün iç boşluğu onu bozmasın.
    let cercevesiz = if b.kurucu == "no_frame" && frame_i.is_none() {
        ".frame(egui::Frame::NONE)"
    } else {
        ""
    };
    oz.sonra = araya_sok(d.satir(sl), sc, &format!(".rol(Rol::{rol}){cercevesiz}"))
        .trim()
        .to_string();
    if frame_i.is_some() {
        oz.baslik.push_str("  [araç kendi .frame(..)'ini veriyor: .rol(..) ondan önce girer, çerçeve aracınki kalır]");
    }
    if !tasinacak.is_empty() {
        oz.baslik.push_str(&format!(
            "  [DİKKAT: .frame'den sonraki {tasinacak:?} yöntemlerini .rol(..)'dan ÖNCEYE taşıyın]"
        ));
    }
    o.push(oz);
}

/// Izgara `tablo` ile sarılı değilse (zincir `.show` ile bitiyor) öner.
fn izgara(d: &Dosya, b: &Bulgu, o: &mut Vec<Oneri>) {
    if b.show.is_none() {
        return; // `let grid = Grid::new(..)..;` + `kilim_tema::tablo(ui, grid, ..)` biçimi
    }
    o.push(yeni(
        d,
        "IZGARA",
        b,
        format!("Grid{} sarılı değil", b.ad.as_deref().map_or(String::new(), |a| format!("(\"{a}\")"))),
        "kısa hücreli tablo: let grid = egui::Grid::new(..); kilim_tema::tablo(ui, grid, |ui| ..)  |  uzun değerli form: kilim_tema::alan_satiri(ui, sol_genislik, ad, |ui| ..)".into(),
        "Grid sütun genişliğini önceki kareden ölçer: sarmada sıçrar (yerlesim.rs kök neden 3)",
    ));
}

/// Üst/alt çubuğun doğrudan gövdesindeki sarmasız `horizontal`.
fn yatay(d: &Dosya, _k: usize, b: &Bulgu, o: &mut Vec<Oneri>) {
    let Some(u) = b.ust.map(|u| &d.bulgular[u]) else {
        return;
    };
    let sabit_yukseklik = u.zincir.iter().any(|z| z.starts_with("exact_"));
    if matches!(u.tur, Tur::PanelUst | Tur::PanelSol | Tur::PanelSag)
        && !sabit_yukseklik
        && b.kurucu == "horizontal"
    {
        o.push(yeni(
            d,
            "YATAY",
            b,
            format!("{} içinde sarmasız horizontal", u.tur.ad()),
            d.satir(b.satir).replacen(".horizontal(", ".horizontal_wrapped(", 1).trim().to_string(),
            "dar pencerede satır sağdan taşar ve kırpılır; sarmalı satır alt satıra iner (pasli K14)",
        ));
    }
}

/// `horizontal*` gövdesindeki ComboBox, `birim` içinde değilse.
fn combo(d: &Dosya, _k: usize, b: &Bulgu, o: &mut Vec<Oneri>) {
    let Some(u) = b.ust.map(|u| &d.bulgular[u]) else {
        return;
    };
    if u.tur == Tur::YatayDolgulu {
        o.push(yeni(
            d,
            "COMBO",
            b,
            "sarmalı satırda ComboBox birim içinde değil".into(),
            "kilim_tema::birim(ui, 170.0, |ui| egui::ComboBox::from_id_salt(..)....show_ui(ui, ..).response)".into(),
            "kendi içinde kap açan denetim kalan dar yerde taşar ya da metnini harf harf kırar (yerlesim.rs kök neden 4)",
        ));
    }
}

fn renk(d: &Dosya, b: &Bulgu, o: &mut Vec<Oneri>) {
    let s = d.satir(b.satir);
    let metin_ya_da_dolgu = s.contains("colored_label")
        || s.contains(".color(")
        || s.contains(".fill(")
        || (s.trim_start().starts_with("const ") && s.contains("Color32"));
    let Some(anlam) = renk_anlami(b).filter(|_| metin_ya_da_dolgu) else {
        return;
    };
    let ek = if b.detay.is_empty() {
        format!("Color32::{}", b.kurucu)
    } else {
        format!("Color32::{}({})", b.kurucu, b.detay)
    };
    o.push(Oneri {
        kod: "RENK",
        yol: d.yol.clone(),
        satir: b.satir,
        baslik: format!("sabit {anlam} rengi: {ek}"),
        once: s.trim().to_string(),
        sonra: renk_degistir(s, &b.kurucu, renk_onerisi(anlam))
            .trim()
            .to_string(),
        gerekce: "sabit renk açık/koyu temayı ve paketi izlemez; anlamsal renk palette bağlıdır",
    });
}

#[cfg(test)]
mod sinama {
    use super::*;

    fn dosya(k: &str) -> Dosya {
        Dosya::oku("a.rs", k)
    }

    #[test]
    fn rol_isaretcisi_show_oncesine_girer() {
        let d = dosya("fn f() {\n    egui::Panel::left(\"c\")\n        .resizable(true)\n        .show(ui, |ui| {});\n}");
        let o = oneriler(&d);
        assert_eq!(o.len(), 1);
        assert_eq!(o[0].kod, "ROL");
        assert_eq!(o[0].satir, 4);
        assert_eq!(o[0].sonra, ".rol(Rol::YanPanel).show(ui, |ui| {});");
    }

    #[test]
    fn kendi_cercevesi_olan_panelde_rol_frame_den_once_girer() {
        let d = dosya(
            "fn f() {
    Panel::bottom(\"d\")
        .frame(zemin)
        .show(ui, |ui| {});
}",
        );
        let o = oneriler(&d);
        assert_eq!(o[0].satir, 3);
        assert_eq!(o[0].sonra, ".rol(Rol::DurumCubugu).frame(zemin)");
        assert!(!o[0].baslik.contains("DİKKAT"));
        let d = dosya(
            "Panel::top(\"u\")
  .frame(c)
  .resizable(true)
  .show(ui, |ui| {});",
        );
        assert!(
            oneriler(&d)[0].baslik.contains("DİKKAT"),
            "frame sonrası resizable taşınmalı"
        );
    }

    #[test]
    fn no_frame_merkez_cercevesiz_kalir() {
        let o = oneriler(&dosya("egui::CentralPanel::no_frame().show(ui, |ui| {});"));
        assert_eq!(o[0].sonra, "egui::CentralPanel::no_frame().rol(Rol::Zemin).frame(egui::Frame::NONE).show(ui, |ui| {});");
    }

    #[test]
    fn isaretli_bolge_onerilmez() {
        let d = dosya("Panel::left(\"c\").rol(Rol::YanPanel).show(ui, |ui| {});");
        assert!(oneriler(&d).is_empty());
    }

    #[test]
    fn sabit_yukseklikli_cubukta_yatay_onerilmez() {
        let a = oneriler(&dosya(
            "Panel::top(\"u\").show(ui, |ui| { ui.horizontal(|ui| {}); });",
        ));
        assert!(a.iter().any(|x| x.kod == "YATAY"));
        let b = oneriler(&dosya(
            "Panel::top(\"u\").exact_size(24.0).show(ui, |ui| { ui.horizontal(|ui| {}); });",
        ));
        assert!(!b.iter().any(|x| x.kod == "YATAY"));
    }

    #[test]
    fn izgara_tabloyla_sarilirsa_onerilmez() {
        let a = oneriler(&dosya(
            "let g = Grid::new(\"x\").striped(true); kilim_tema::tablo(ui, g, |ui| {});",
        ));
        assert!(a.is_empty(), "{a:?}");
        let b = oneriler(&dosya("Grid::new(\"x\").show(ui, |ui| {});"));
        assert_eq!(b[0].kod, "IZGARA");
    }

    #[test]
    fn renk_anlami_siniflanir() {
        let o = oneriler(&dosya("ui.colored_label(egui::Color32::RED, h);\nui.colored_label(Color32::from_rgb(230, 180, 60), t);\nlet x = Color32::from_rgb(1, 2, 3);"));
        let k: Vec<_> = o.iter().map(|x| x.sonra.as_str()).collect();
        assert_eq!(
            k,
            [
                "ui.colored_label(ui.visuals().error_fg_color, h);",
                "ui.colored_label(ui.visuals().warn_fg_color, t);"
            ]
        );
    }

    #[test]
    fn combo_yalniz_sarmali_satirda() {
        let a = oneriler(&dosya(
            "ui.horizontal_wrapped(|ui| { ComboBox::from_id_salt(1).show_ui(ui, |ui| {}); });",
        ));
        assert_eq!(a.iter().filter(|x| x.kod == "COMBO").count(), 1);
        let b = oneriler(&dosya(
            "kilim_tema::birim(ui, 1.0, |ui| { ComboBox::from_id_salt(1).show_ui(ui, |ui| {}); });",
        ));
        assert!(b.is_empty());
    }
}
