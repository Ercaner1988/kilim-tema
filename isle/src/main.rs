//! Gemini ham görsellerini kilim-tema varlıklarına çevirir ve doğrudan `assets/`'e yazar.
//!
//! `cargo run --release -p kilim-isle -- [kaynak klasör]` (varsayılan `Desktop\tema`).
//! Özgün görsellere dokunmaz. Görsel yeniden üretilirse aşağıdaki adı güncelleyin.
//! Her adımın kabul denetimi var (dikiş, kontrast, saçak); biri tutmazsa durur.

mod boncuk;
mod kilim;
mod ortak;
mod zemin;

use std::path::{Path, PathBuf};

/// Gemini dosya adlarının değişken kısmı: `Gemini_Generated_Image_<ad>.jpg`.
const KAGIT: &str = "x3m9j4x3m9j4x3m9"; // ilk sürüm b4xib1b4xib1b4xi (1024 karo, "L" lekeli)
const LEKE: &str = "8tqdt38tqdt38tqd";
const CERCEVE_A: &str = "ca70phca70phca70";
const CERCEVE_B: &str = "amgvlcamgvlcamgv";
const SERIT_B: &str = "16vbq216vbq216vb";
const BONCUK: &str = "u9dob6u9dob6u9do";

/// En kavruk yerde mürekkep kontrastı ≥ 4.5 kalsın diye ölçüldü.
const LEKE_OPAKLIK: f32 = 0.85;
/// Kenarlık ve boncuk dokusu: 2x ekranda 32 mantıksal px.
const UI_KALINLIK: u32 = 64;

fn main() {
    let kaynak = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("C:/Users/buzbe/Desktop/tema"));
    let cikti = Path::new(env!("CARGO_MANIFEST_DIR")).join("../assets");
    zemin::kagit(&kaynak, KAGIT, &cikti);
    zemin::leke(&kaynak, LEKE, &cikti, LEKE_OPAKLIK);
    kilim::kilim(&kaynak, SERIT_B, CERCEVE_A, CERCEVE_B, &cikti, UI_KALINLIK);
    boncuk::boncuklar(&kaynak, BONCUK, &cikti, UI_KALINLIK);
    println!("assets: {}", cikti.canonicalize().unwrap().display());
}
