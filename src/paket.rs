//! Tema paketi: paleti ve vurguyu veri olarak taşıyan, el-Fihrist gibi bir merkezde
//! saklanıp araçlara dağıtılan küçük metin dosyası (`anahtar=değer`, `;` yorum).
//! Araç yalnız etkin paketi tutar (`<uygulama>.tema`); katalog yalnız kullanıcı tema
//! değiştirirken sorulur. Eksik anahtar yerleşik değeri korur, bilinmeyen anahtar
//! yok sayılır (ileriye uyum); `sema` desteklenenden büyükse paket reddedilir.

use std::path::PathBuf;
use std::sync::RwLock;

use egui::Color32;

use crate::renk::{yerlesik_acik, yerlesik_koyu};
use crate::{Kilim, Varyant};

/// Bu sürümün anladığı en yüksek paket şeması.
pub const SEMA: u32 = 1;

#[derive(Clone, Debug, PartialEq)]
pub struct TemaPaketi {
    pub id: String,
    pub ad: String,
    /// Hangi kenarlık dokusu (`a` cam göbeği + altın, `b` kırmızı + yeşil).
    pub kenarlik: Varyant,
    pub acik: Kilim,
    pub koyu: Kilim,
    pub vurgu_acik: Color32,
    pub vurgu_koyu: Color32,
}

/// Paket + katalog arasındaki sözleşme. Katalog (el-Fihrist) bunu uygular; tema
/// seçici yalnız menü açıkken çağırır, uygulayıcı ucuz olmalı.
pub trait TemaKaynagi {
    /// `(id, görünen ad)` çiftleri.
    fn liste(&self) -> Vec<(String, String)>;
    fn getir(&self, id: &str) -> Option<TemaPaketi>;
}

const ALANLAR: [&str; 14] = [
    "zem",
    "yuzey",
    "cizgi",
    "cizgi_koyu",
    "murekkep",
    "soluk",
    "cini",
    "mercan",
    "hardal",
    "yesil",
    "sari",
    "kirmizi",
    "gri",
    "kontrast",
];

fn alan_oku(k: &Kilim, ad: &str) -> Option<Color32> {
    Some(match ad {
        "zem" => k.zem,
        "yuzey" => k.yuzey,
        "cizgi" => k.cizgi,
        "cizgi_koyu" => k.cizgi_koyu,
        "murekkep" => k.murekkep,
        "soluk" => k.soluk,
        "cini" => k.cini,
        "mercan" => k.mercan,
        "hardal" => k.hardal,
        "yesil" => k.yesil,
        "sari" => k.sari,
        "kirmizi" => k.kirmizi,
        "gri" => k.gri,
        "kontrast" => k.kontrast,
        _ => return None,
    })
}

fn alan_yaz(k: &mut Kilim, ad: &str, c: Color32) -> bool {
    let hedef = match ad {
        "zem" => &mut k.zem,
        "yuzey" => &mut k.yuzey,
        "cizgi" => &mut k.cizgi,
        "cizgi_koyu" => &mut k.cizgi_koyu,
        "murekkep" => &mut k.murekkep,
        "soluk" => &mut k.soluk,
        "cini" => &mut k.cini,
        "mercan" => &mut k.mercan,
        "hardal" => &mut k.hardal,
        "yesil" => &mut k.yesil,
        "sari" => &mut k.sari,
        "kirmizi" => &mut k.kirmizi,
        "gri" => &mut k.gri,
        "kontrast" => &mut k.kontrast,
        _ => return false,
    };
    *hedef = c;
    true
}

fn hex_coz(s: &str) -> Option<Color32> {
    let s = s.strip_prefix('#')?;
    if s.len() != 6 {
        return None;
    }
    let n = u32::from_str_radix(s, 16).ok()?;
    Some(Color32::from_rgb((n >> 16) as u8, (n >> 8) as u8, n as u8))
}

fn hex_yaz(c: Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", c.r(), c.g(), c.b())
}

fn renk(anahtar: &str, deger: &str) -> Result<Color32, String> {
    hex_coz(deger).ok_or_else(|| format!("{anahtar}: '{deger}' #RRGGBB değil"))
}

impl TemaPaketi {
    /// Koddaki yerleşik tema, verilen kenarlık dokusuyla. Katalog tohumu ve sınama için.
    pub fn yerlesik(id: &str, ad: &str, kenarlik: Varyant) -> Self {
        Self {
            id: id.into(),
            ad: ad.into(),
            kenarlik,
            acik: yerlesik_acik(),
            koyu: yerlesik_koyu(),
            vurgu_acik: kenarlik.yerlesik_vurgu(false),
            vurgu_koyu: kenarlik.yerlesik_vurgu(true),
        }
    }

    pub fn coz(metin: &str) -> Result<Self, String> {
        let mut p = Self::yerlesik("", "", Varyant::CamGobegiAltin);
        let mut sema = None;
        for (no, satir) in metin.lines().enumerate() {
            let s = satir.trim();
            if s.is_empty() || s.starts_with(';') {
                continue;
            }
            let (k, v) = s
                .split_once('=')
                .ok_or(format!("satır {}: '=' yok", no + 1))?;
            let (k, v) = (k.trim(), v.trim());
            match k {
                "sema" => {
                    sema = Some(
                        v.parse::<u32>()
                            .map_err(|_| "sema sayı olmalı".to_string())?,
                    )
                }
                "id" => p.id = v.into(),
                "ad" => p.ad = v.into(),
                "kenarlik" => {
                    p.kenarlik = match v {
                        "a" => Varyant::CamGobegiAltin,
                        "b" => Varyant::KirmiziYesil,
                        _ => return Err(format!("kenarlik: '{v}' a ya da b değil")),
                    }
                }
                "vurgu.acik" => p.vurgu_acik = renk(k, v)?,
                "vurgu.koyu" => p.vurgu_koyu = renk(k, v)?,
                _ => {
                    if let Some(a) = k.strip_prefix("acik.") {
                        alan_yaz(&mut p.acik, a, renk(k, v)?);
                    } else if let Some(a) = k.strip_prefix("koyu.") {
                        alan_yaz(&mut p.koyu, a, renk(k, v)?);
                    }
                }
            }
        }
        match sema {
            Some(n) if n <= SEMA => {}
            Some(n) => {
                return Err(format!(
                    "sema {n} bu sürümde desteklenmiyor (en çok {SEMA})"
                ))
            }
            None => return Err("sema yok".into()),
        }
        if p.id.is_empty()
            || !p
                .id
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err("id boş ya da [a-z0-9-] dışında".into());
        }
        if p.ad.is_empty() {
            p.ad = p.id.clone();
        }
        Ok(p)
    }

    pub fn kod(&self) -> String {
        let mut s = format!(
            "; kilim-tema paketi\nsema={SEMA}\nid={}\nad={}\nkenarlik={}\nvurgu.acik={}\nvurgu.koyu={}\n",
            self.id,
            self.ad,
            if self.kenarlik == Varyant::CamGobegiAltin { "a" } else { "b" },
            hex_yaz(self.vurgu_acik),
            hex_yaz(self.vurgu_koyu),
        );
        for (onek, k) in [("acik", &self.acik), ("koyu", &self.koyu)] {
            for a in ALANLAR {
                s.push_str(&format!(
                    "{onek}.{a}={}\n",
                    hex_yaz(alan_oku(k, a).expect("bilinen alan"))
                ));
            }
        }
        s
    }
}

static ETKIN: RwLock<Option<TemaPaketi>> = RwLock::new(None);

/// Etkin paketi kurar (`None`: yerleşik tema). `acik()`, `koyu()` ve `Varyant::vurgu` bunu izler.
pub fn paketi_kur(p: Option<TemaPaketi>) {
    *ETKIN.write().unwrap() = p;
}

pub fn etkin_paket() -> Option<TemaPaketi> {
    ETKIN.read().unwrap().clone()
}

pub(crate) fn etkin_palet(koyu: bool) -> Option<Kilim> {
    ETKIN
        .read()
        .unwrap()
        .as_ref()
        .map(|p| if koyu { p.koyu } else { p.acik })
}

pub(crate) fn etkin_vurgu(koyu: bool) -> Option<Color32> {
    ETKIN
        .read()
        .unwrap()
        .as_ref()
        .map(|p| if koyu { p.vurgu_koyu } else { p.vurgu_acik })
}

pub(crate) fn uygulama_dosyasi(uygulama: &str, uzanti: &str) -> Option<PathBuf> {
    let kok = if cfg!(windows) {
        std::env::var_os("APPDATA").map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Application Support"))
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
    }?;
    Some(kok.join("kilim-tema").join(format!("{uygulama}.{uzanti}")))
}

/// `<uygulama>.tema` varsa okuyup kurar. Dosya yok ya da bozuksa yerleşik tema kalır.
pub fn etkin_paketi_yukle(uygulama: &str) -> bool {
    let Some(p) = uygulama_dosyasi(uygulama, "tema")
        .and_then(|d| std::fs::read_to_string(d).ok())
        .and_then(|m| TemaPaketi::coz(&m).ok())
    else {
        return false;
    };
    paketi_kur(Some(p));
    true
}

/// Kaydedemezse sessizce geçer: tema yalnız kolaylıktır.
pub fn etkin_paketi_kaydet(uygulama: &str, p: &TemaPaketi) {
    if let Some(d) = uygulama_dosyasi(uygulama, "tema") {
        let _ = d.parent().map(std::fs::create_dir_all);
        let _ = std::fs::write(d, p.kod());
    }
}

pub fn etkin_paketi_sil(uygulama: &str) {
    if let Some(d) = uygulama_dosyasi(uygulama, "tema") {
        let _ = std::fs::remove_file(d);
    }
}

#[cfg(test)]
mod sinama {
    use super::*;

    fn ornek() -> TemaPaketi {
        let mut p = TemaPaketi::yerlesik("deneme-1", "Deneme: tek satır", Varyant::KirmiziYesil);
        p.acik.zem = Color32::from_rgb(1, 2, 3);
        p.koyu.cini = Color32::from_rgb(250, 251, 252);
        p.vurgu_acik = Color32::from_rgb(9, 8, 7);
        p
    }

    #[test]
    fn kod_gidip_gelir() {
        let p = ornek();
        assert_eq!(TemaPaketi::coz(&p.kod()).unwrap(), p);
    }

    #[test]
    fn eksik_anahtar_yerlesigi_korur_bilinmeyen_yok_sayilir() {
        let p = TemaPaketi::coz(
            "sema=1\nid=x\nacik.zem=#010203\ngelecek.anahtar=1\nacik.yeni_alan=#000000\n",
        )
        .unwrap();
        assert_eq!(p.acik.zem, Color32::from_rgb(1, 2, 3));
        assert_eq!(p.acik.yuzey, yerlesik_acik().yuzey);
        assert_eq!(p.koyu, yerlesik_koyu());
        assert_eq!(p.ad, "x");
    }

    #[test]
    fn bozuk_paket_reddedilir() {
        for m in [
            "id=x\n",
            "sema=2\nid=x\n",
            "sema=1\nid=Büyük\n",
            "sema=1\nid=\n",
            "sema=1\nid=x\nacik.zem=kırmızı\n",
            "sema=1\nid=x\nkenarlik=c\n",
            "sema=1\nid=x\nyarım satır\n",
        ] {
            assert!(TemaPaketi::coz(m).is_err(), "{m:?}");
        }
    }
}
