//! Dizin yürüyüşü + düz metin plan ve işaretçi talep listesi (JSON değil: Ercan'ın CLI kuralı).

use std::collections::BTreeMap;
use std::fmt::Write;
use std::path::Path;

use crate::kural::{isaretli, oneriler, rol_kararlari, rol_oner, Dosya, Oneri};
use crate::tara::{Bulgu, Tur};

pub struct Arac {
    pub ad: String,
    pub dosyalar: Vec<Dosya>,
}

const ATLANAN_DIZINLER: [&str; 5] = ["target", ".git", "node_modules", "graphify-out", ".harness"];
const TEST_DIZINLERI: [&str; 3] = ["tests", "examples", "benches"];

/// `kok` altındaki `.rs` dosyalarını okur; `testler` kapalıyken test/örnek dizinleri ve `sinama*.rs`
/// atlanır (test kodu arayüzü taklit eder, araç arayüzü değildir).
pub fn dizin_tara(kok: &Path, testler: bool) -> std::io::Result<Vec<Dosya>> {
    let mut dosyalar = Vec::new();
    yuru(kok, kok, testler, &mut dosyalar)?;
    dosyalar.sort_by(|a, b| a.yol.cmp(&b.yol));
    Ok(dosyalar)
}

fn yuru(kok: &Path, dizin: &Path, testler: bool, cikti: &mut Vec<Dosya>) -> std::io::Result<()> {
    for e in std::fs::read_dir(dizin)? {
        let yol = e?.path();
        let ad = yol.file_name().and_then(|a| a.to_str()).unwrap_or("");
        if yol.is_dir() {
            if ATLANAN_DIZINLER.contains(&ad) || (!testler && TEST_DIZINLERI.contains(&ad)) {
                continue;
            }
            yuru(kok, &yol, testler, cikti)?;
        } else if ad.ends_with(".rs")
            && (testler || !(ad.starts_with("sinama") || ad == "tests.rs"))
        {
            // Okunamayan (UTF-8 olmayan) dosya sessizce atlanmaz: hata döner.
            let kaynak = std::fs::read_to_string(&yol)?;
            let goreli = yol
                .strip_prefix(kok)
                .unwrap_or(&yol)
                .to_string_lossy()
                .into_owned();
            let mut d = Dosya::oku(&goreli, &kaynak);
            d.kilim_bagimli = sandik_bagimli(kok, &yol);
            cikti.push(d);
        }
    }
    Ok(())
}

/// En yakın `Cargo.toml` (kök dahil) `kilim-tema`yı anıyor mu?
fn sandik_bagimli(kok: &Path, dosya: &Path) -> bool {
    let mut d = dosya.parent();
    while let Some(dizin) = d {
        if let Ok(m) = std::fs::read_to_string(dizin.join("Cargo.toml")) {
            return m.contains("kilim-tema") || m.contains("kilim_tema");
        }
        if dizin == kok {
            break;
        }
        d = dizin.parent();
    }
    false
}

fn ust_bolge_sayisi(d: &Dosya, mut k: usize) -> usize {
    let mut derin = 0;
    while let Some(u) = d.bulgular[k].ust {
        if d.bulgular[u].tur.bolge_mi() && rol_oner(d, u).is_some() {
            derin += 1;
        }
        k = u;
    }
    derin
}

/// Bölgenin doğrudan içeriği + `ui`'yi geçirdiği işlevlerin (aynı araçta tek tanımı varsa) içeriği.
fn icerik_metni(a: &Arac, d: &Dosya, k: usize) -> String {
    let mut parcalar: Vec<String> = Vec::new();
    let mut say = d.icerik(k);
    let cagrilar = say.remove(&Tur::DisCagri).unwrap_or(0);
    parcalar.extend(say.iter().map(|(t, n)| format!("{n} {}", t.ad())));
    if cagrilar > 0 {
        let mut adlar: Vec<(&str, &str)> = d
            .bulgular
            .iter()
            .enumerate()
            .filter(|(j, c)| c.tur == Tur::DisCagri && ust_mu(d, *j, k))
            .map(|(_, c)| (c.kurucu.as_str(), c.detay.as_str()))
            .collect();
        adlar.sort_unstable();
        adlar.dedup();
        for (ad, nitelik) in adlar {
            parcalar.push(match islev_icerigi(a, d, ad, nitelik) {
                Some(m) => format!("{ad}() içinde [{m}]"),
                None => format!("{ad}() (tanımı bu araçta yok ya da birden çok)"),
            });
        }
    }
    if parcalar.is_empty() {
        "boş".into()
    } else {
        parcalar.join(", ")
    }
}

/// `pencere-egui/src/app.rs` → `pencere-egui` (sandık dizini).
fn sandik(yol: &str) -> &str {
    yol.split("/src/").next().unwrap_or(yol)
}

fn ust_mu(d: &Dosya, mut j: usize, k: usize) -> bool {
    while let Some(u) = d.bulgular[j].ust {
        if u == k {
            return true;
        }
        j = u;
    }
    false
}

/// Adı `ad` olan TEK işlevin gövdesindeki öğeler (başka işlevlere yeniden inmez).
fn islev_icerigi(a: &Arac, cagiran: &Dosya, ad: &str, nitelik: &str) -> Option<String> {
    let mut adaylar: Vec<(&Dosya, &crate::tara::Islev)> = a
        .dosyalar
        .iter()
        .flat_map(|d| {
            d.islevler
                .iter()
                .filter(|f| f.ad == ad)
                .map(move |f| (d, f))
        })
        .collect();
    // Belirsizse sırayla daralt: `modul::ad(..)` niteleyicisi, sonra çağıranla aynı sandık.
    if adaylar.len() > 1 && !nitelik.is_empty() {
        let dar: Vec<_> = adaylar
            .iter()
            .copied()
            .filter(|(d, _)| {
                d.yol.ends_with(&format!("/{nitelik}.rs"))
                    || d.yol.contains(&format!("/{nitelik}/"))
            })
            .collect();
        if !dar.is_empty() {
            adaylar = dar;
        }
    }
    if adaylar.len() > 1 {
        let dar: Vec<_> = adaylar
            .iter()
            .copied()
            .filter(|(d, _)| sandik(&d.yol) == sandik(&cagiran.yol))
            .collect();
        if !dar.is_empty() {
            adaylar = dar;
        }
    }
    let [(d, f)] = adaylar[..] else { return None };
    let mut say: BTreeMap<&str, usize> = BTreeMap::new();
    for b in &d.bulgular {
        if b.indeks >= f.govde.0
            && b.indeks < f.govde.1
            && !matches!(
                b.tur,
                Tur::Renk | Tur::TemaRengi | Tur::Kurulum | Tur::DisCagri
            )
        {
            *say.entry(b.tur.ad()).or_insert(0) += 1;
        }
    }
    Some(if say.is_empty() {
        "yalnız çizim".into()
    } else {
        say.iter()
            .map(|(t, n)| format!("{n} {t}"))
            .collect::<Vec<_>>()
            .join(", ")
    })
}

fn kurulum_ozeti(a: &Arac) -> String {
    let mut s = String::new();
    for ad in [
        "giydir",
        "sarma_kur",
        "iki_temayi_kur",
        "uygula",
        "cerceve",
        "tema_secici",
    ] {
        let yer: Vec<String> = a
            .dosyalar
            .iter()
            .flat_map(|d| {
                d.bulgular
                    .iter()
                    .filter(|b| b.tur == Tur::Kurulum && b.kurucu == ad)
                    .map(move |b| format!("{}:{}", d.yol, b.satir))
            })
            .collect();
        let _ = write!(
            s,
            "{ad}: {} | ",
            if yer.is_empty() {
                "YOK".to_string()
            } else {
                yer.join(", ")
            }
        );
    }
    s
}

pub fn plan_metni(a: &Arac) -> String {
    let mut s = format!("=== ARAÇ: {} ({} dosya) ===\n", a.ad, a.dosyalar.len());
    let giris = a
        .dosyalar
        .iter()
        .filter(|d| d.satirlar.iter().any(|l| l.contains("run_native(")))
        .map(|d| d.yol.as_str())
        .collect::<Vec<_>>();
    let _ = writeln!(
        s,
        "Giriş (run_native): {}",
        if giris.is_empty() {
            "yok (kitaplık)".into()
        } else {
            giris.join(", ")
        }
    );
    let _ = writeln!(s, "Kurulum: {}", kurulum_ozeti(a));
    let (mut toplam, mut isaretli_say) = (0, 0);
    let mut turler: BTreeMap<&'static str, usize> = BTreeMap::new();
    for d in &a.dosyalar {
        for (k, b) in d.bulgular.iter().enumerate() {
            *turler.entry(b.tur.ad()).or_insert(0) += 1;
            if rol_oner(d, k).is_some() && b.show.is_some() {
                toplam += 1;
                isaretli_say += usize::from(isaretli(b));
            }
        }
    }
    let _ = writeln!(
        s,
        "Rol taşıyabilen bölge: {toplam} (işaretli {isaretli_say}, işaretsiz {})",
        toplam - isaretli_say
    );
    let _ = writeln!(
        s,
        "Bulunan öğeler: {}",
        turler
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let _ = writeln!(s, "\nBölge ağacı (dosya sırasıyla):");
    for d in &a.dosyalar {
        for (k, b) in d.bulgular.iter().enumerate() {
            if let Some(r) = bolge_satiri(a, d, k, b) {
                s.push_str(&r);
            }
        }
    }
    let ons: Vec<Oneri> = a.dosyalar.iter().flat_map(oneriler).collect();
    let mut kodlar: BTreeMap<&str, usize> = BTreeMap::new();
    for o in &ons {
        *kodlar.entry(o.kod).or_insert(0) += 1;
    }
    let _ = writeln!(
        s,
        "\nÖneri sayıları: {}",
        if kodlar.is_empty() {
            "yok".into()
        } else {
            kodlar
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join(", ")
        }
    );
    s
}

fn bolge_satiri(a: &Arac, d: &Dosya, k: usize, b: &Bulgu) -> Option<String> {
    let rol = rol_oner(d, k);
    if !(b.tur.bolge_mi() && b.show.is_some() && rol.is_some()) {
        return None;
    }
    let girinti = "  ".repeat(1 + ust_bolge_sayisi(d, k));
    let ad =
        b.ad.as_deref()
            .map_or(String::new(), |a| format!("(\"{a}\")"));
    let durum = if isaretli(b) {
        "işaretli"
    } else {
        "İŞARETSİZ"
    };
    let ic = icerik_metni(a, d, k);
    let rol = rol.unwrap_or("-");
    let mut s = format!(
        "{girinti}{}:{}  {}{ad}  -> Rol::{rol}  [{durum}]\n{girinti}    içerik: {ic}\n",
        d.yol,
        b.satir,
        b.tur.ad()
    );
    let kararlar = rol_kararlari(rol);
    if !kararlar.is_empty() {
        let _ = writeln!(s, "{girinti}    karar: {kararlar}");
    }
    Some(s)
}

/// Araç sahibine: dosya:satır, ne yapılacak, önce/sonra.
pub fn talep_metni(a: &Arac) -> String {
    let mut s = format!("=== İŞARETÇİ TALEPLERİ: {} ===\n", a.ad);
    let mut n = 0;
    n += kurulum_talebi(a, &mut s);
    for d in &a.dosyalar {
        let ons = oneriler(d);
        if ons.is_empty() {
            continue;
        }
        let bag = if d.kilim_bagimli {
            ""
        } else {
            " + Cargo.toml: kilim-tema bağımlılığı YOK (kitaplık temadan bağımsız tasarlandıysa talep konak uygulamaya aittir)"
        };
        let _ = writeln!(
            s,
            "\n-- {} (kullanım: use kilim_tema::{{Rol, RolVer}};{bag})",
            d.yol
        );
        for o in ons {
            n += 1;
            let _ = writeln!(
                s,
                "{}:{}  [{}] {}\n    gerekçe: {}\n    önce : {}\n    sonra: {}",
                o.yol, o.satir, o.kod, o.baslik, o.gerekce, o.once, o.sonra
            );
        }
    }
    let _ = writeln!(s, "\nToplam talep: {n}");
    s
}

/// Araç düzeyi talep: giriş (`run_native`) var ama `giydir` çağrılmıyor. Mevcut kurulum çağrıları listelenir
/// (yerini `giydir` alır; `giydir` hepsini yapar: iki tema, sarma, tercih, bindirme).
fn kurulum_talebi(a: &Arac, s: &mut String) -> usize {
    let var = |ad: &str| {
        a.dosyalar.iter().any(|d| {
            d.bulgular
                .iter()
                .any(|b| b.tur == Tur::Kurulum && b.kurucu == ad)
        })
    };
    let Some((d, satir)) = a.dosyalar.iter().find_map(|d| {
        d.satirlar
            .iter()
            .position(|l| l.contains("run_native("))
            .map(|i| (d, i + 1))
    }) else {
        return 0;
    };
    if var("giydir") {
        return 0;
    }
    let eksik: Vec<&str> = ["iki_temayi_kur", "sarma_kur", "uygula"]
        .into_iter()
        .filter(|k| !var(k))
        .collect();
    let _ = writeln!(
        s,
        "\n-- araç düzeyi\n{}:{}  [KURULUM] giriş `run_native` var, `kilim_tema::giydir` yok\n    gerekçe: giydir = iki_temayi_kur + sarma_kur + tercih.uygula + (KILIM_ROL_IZ=1 ise) rol bindirmesi; şu an eksik olanlar: {}\n    önce : Box::new(|cc| {{ .. kendi tema/sarma çağrıları .. }})\n    sonra: Box::new(|cc| {{ let tercih = kilim_tema::TemaTercihi::yukle(\"<uygulama>\", kilim_tema::TemaTercihi::varsayilan(kilim_tema::Varyant::CamGobegiAltin)); kilim_tema::giydir(&cc.egui_ctx, &tercih); .. }})",
        d.yol,
        satir,
        if eksik.is_empty() { "yok (yalnız toplu çağrı)".to_string() } else { eksik.join(", ") }
    );
    1
}
