//! Rust kaynağını sözcüklere ayırır: yorumlar atılır, dize içerikleri tek sözcük olur (içindeki
//! parantez/süslü parantez yapıyı bozmaz). Tam bir ayrıştırıcı değildir; egui çağrı zincirlerini
//! bulmaya yetecek kadar: ad, dize, sayı, simge (`::` birleşik), ömür.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tur {
    Ad,
    Dize,
    Sayi,
    Simge,
}

#[derive(Clone, Debug)]
pub struct Sozcuk {
    pub tur: Tur,
    pub metin: String,
    /// 1'den başlar.
    pub satir: u32,
    /// 1'den başlar (karakter sayısıyla).
    pub sutun: u32,
}

impl Sozcuk {
    pub fn simge(&self, s: &str) -> bool {
        self.tur == Tur::Simge && self.metin == s
    }
    pub fn ad(&self, s: &str) -> bool {
        self.tur == Tur::Ad && self.metin == s
    }
}

pub fn ayir(kaynak: &str) -> Vec<Sozcuk> {
    let b: Vec<char> = kaynak.chars().collect();
    let mut cikti = Vec::new();
    let (mut i, mut satir, mut sutun) = (0usize, 1u32, 1u32);
    // Bir karakter ilerler, satır/sütun tutar.
    macro_rules! ilerle {
        () => {{
            if b[i] == '\n' {
                satir += 1;
                sutun = 1;
            } else {
                sutun += 1;
            }
            i += 1;
        }};
    }
    while i < b.len() {
        let c = b[i];
        let (s0, c0) = (satir, sutun);
        let sonra = |k: usize| b.get(i + k).copied();
        if c.is_whitespace() {
            ilerle!();
        } else if c == '/' && sonra(1) == Some('/') {
            while i < b.len() && b[i] != '\n' {
                ilerle!();
            }
        } else if c == '/' && sonra(1) == Some('*') {
            let mut derin = 0;
            while i < b.len() {
                if b[i] == '/' && b.get(i + 1) == Some(&'*') {
                    derin += 1;
                    ilerle!();
                } else if b[i] == '*' && b.get(i + 1) == Some(&'/') {
                    derin -= 1;
                    ilerle!();
                    if derin == 0 {
                        ilerle!();
                        break;
                    }
                }
                ilerle!();
            }
        } else if let Some(onek) = dize_oneki(&b, i) {
            // r"..", r#".."#, b"..", br".." ve düz ".."
            for _ in 0..onek.uzunluk {
                ilerle!();
            }
            let mut icerik = String::new();
            while i < b.len() {
                let ch = b[i];
                if onek.ham {
                    if ch == '"' && (1..=onek.diyez).all(|k| b.get(i + k) == Some(&'#')) {
                        for _ in 0..=onek.diyez {
                            ilerle!();
                        }
                        break;
                    }
                } else if ch == '\\' {
                    ilerle!();
                    if i < b.len() {
                        icerik.push(b[i]);
                        ilerle!();
                    }
                    continue;
                } else if ch == '"' {
                    ilerle!();
                    break;
                }
                icerik.push(ch);
                ilerle!();
            }
            cikti.push(Sozcuk {
                tur: Tur::Dize,
                metin: icerik,
                satir: s0,
                sutun: c0,
            });
        } else if c == '\'' {
            // Karakter ('x', '\n') ya da ömür ('a): kapanış tırnağı doğru yerdeyse karakter.
            let kapanis = if sonra(1) == Some('\\') { 3 } else { 2 };
            if sonra(kapanis) == Some('\'') {
                for _ in 0..=kapanis {
                    ilerle!();
                }
                cikti.push(Sozcuk {
                    tur: Tur::Sayi,
                    metin: "'c'".into(),
                    satir: s0,
                    sutun: c0,
                });
            } else {
                ilerle!();
            }
        } else if c.is_alphabetic() || c == '_' {
            let mut ad = String::new();
            while i < b.len() && (b[i].is_alphanumeric() || b[i] == '_') {
                ad.push(b[i]);
                ilerle!();
            }
            cikti.push(Sozcuk {
                tur: Tur::Ad,
                metin: ad,
                satir: s0,
                sutun: c0,
            });
        } else if c.is_ascii_digit() {
            let mut s = String::new();
            // `1.5`, `0x1F`, `255u8`: nokta yalnız ardından rakam geliyorsa sayıya dahildir (`0..n` bozulmaz).
            while i < b.len()
                && (b[i].is_alphanumeric()
                    || b[i] == '_'
                    || (b[i] == '.' && b.get(i + 1).is_some_and(|d| d.is_ascii_digit())))
            {
                s.push(b[i]);
                ilerle!();
            }
            cikti.push(Sozcuk {
                tur: Tur::Sayi,
                metin: s,
                satir: s0,
                sutun: c0,
            });
        } else if c == ':' && sonra(1) == Some(':') {
            ilerle!();
            ilerle!();
            cikti.push(Sozcuk {
                tur: Tur::Simge,
                metin: "::".into(),
                satir: s0,
                sutun: c0,
            });
        } else {
            ilerle!();
            cikti.push(Sozcuk {
                tur: Tur::Simge,
                metin: c.to_string(),
                satir: s0,
                sutun: c0,
            });
        }
    }
    cikti
}

struct Onek {
    uzunluk: usize,
    ham: bool,
    diyez: usize,
}

/// `i`'de dize başlıyorsa önekin uzunluğu (açılış tırnağı dahil).
fn dize_oneki(b: &[char], i: usize) -> Option<Onek> {
    let at = |k: usize| b.get(i + k).copied();
    // Bir adın içindeki harf (`bar"`) dize öneki sayılmaz.
    if i > 0 && (b[i - 1].is_alphanumeric() || b[i - 1] == '_') {
        return None;
    }
    let mut k = 0;
    if at(k) == Some('b') {
        k += 1;
    }
    let ham = at(k) == Some('r');
    if ham {
        k += 1;
    }
    let mut diyez = 0;
    while ham && at(k) == Some('#') {
        diyez += 1;
        k += 1;
    }
    (at(k) == Some('"')).then_some(Onek {
        uzunluk: k + 1,
        ham,
        diyez,
    })
}

#[cfg(test)]
mod sinama {
    use super::*;

    fn adlar(k: &str) -> Vec<String> {
        ayir(k).into_iter().map(|s| s.metin).collect()
    }

    #[test]
    fn yorum_ve_dize_yapiyi_bozmaz() {
        let t = adlar("a(\"x)}\"); // ) {\n/* ) /* ic */ ) */ b");
        assert_eq!(t, ["a", "(", "x)}", ")", ";", "b"]);
    }

    #[test]
    fn omur_ve_karakter_ayrilir() {
        let t = adlar("fn f<'a>(x: &'a str) { let c = '{'; let k = '\\n'; }");
        // Karakter sabiti süslü parantez sayılmaz: yalnız fonksiyon gövdesinin ikisi kalır.
        assert_eq!(t.iter().filter(|x| *x == "{").count(), 1, "{t:?}");
        assert_eq!(t.iter().filter(|x| *x == "'c'").count(), 2, "{t:?}");
    }

    #[test]
    fn ham_dize_ve_satir_sutun() {
        let s = ayir("x\n  r#\"a\"b\"# y");
        assert_eq!(s[1].metin, "a\"b");
        assert_eq!((s[1].satir, s[1].sutun), (2, 3));
        assert_eq!(s[2].metin, "y");
    }

    #[test]
    fn aralik_noktasi_sayiya_katilmaz() {
        assert_eq!(adlar("0..n 1.5"), ["0", ".", ".", "n", "1.5"]);
    }
}
