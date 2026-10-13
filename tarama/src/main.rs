//! `kilim-tarama [--talep] [--testler] <ad>=<dizin> ...` → araç başına düz metin plan (varsayılan) ya da
//! işaretçi talep listesi (`--talep`).

use std::path::Path;

use kilim_tarama::rapor::{dizin_tara, plan_metni, talep_metni, Arac};

fn main() {
    let (mut talep, mut testler, mut araclar) = (false, false, Vec::new());
    for a in std::env::args().skip(1) {
        match a.as_str() {
            "--talep" => talep = true,
            "--testler" => testler = true,
            _ => araclar.push(a),
        }
    }
    if araclar.is_empty() {
        eprintln!("kullanım: kilim-tarama [--talep] [--testler] <ad>=<dizin> ...");
        std::process::exit(2);
    }
    for a in araclar {
        let (ad, dizin) = a.split_once('=').unwrap_or((a.as_str(), a.as_str()));
        match dizin_tara(Path::new(dizin), testler) {
            Ok(dosyalar) => {
                let arac = Arac {
                    ad: ad.to_string(),
                    dosyalar,
                };
                println!(
                    "{}",
                    if talep {
                        talep_metni(&arac)
                    } else {
                        plan_metni(&arac)
                    }
                );
            }
            Err(h) => {
                eprintln!("{ad}: {dizin} taranamadı: {h}");
                std::process::exit(1);
            }
        }
    }
}
