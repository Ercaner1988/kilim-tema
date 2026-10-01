//! Kilim kenarlık: şeritten kenar birimi, tam çerçevelerden dört köşe.

use std::path::Path;

use image::{Rgb, Rgb32FImage};

use crate::ortak::*;

type Agirlik = fn(&Rgb<f32>) -> f32;

fn yesil_mi(p: &Rgb<f32>) -> f32 {
    1.0 / (1.0 + (-(p[1] - p[0]) * 255.0 / 12.0).exp())
}

fn altin_mi(p: &Rgb<f32>) -> f32 {
    1.0 / (1.0 + (-(p[0] - p[2]) * 255.0 / 12.0).exp())
}

/// (zemin, motif) ağırlıklı renk ortalamaları.
fn sinif_ortalamalari<'a>(
    px: impl Iterator<Item = &'a Rgb<f32>>,
    motif_mi: Agirlik,
) -> ([f32; 3], [f32; 3]) {
    let (mut z, mut m, mut wz, mut wm) = ([0.0f64; 3], [0.0f64; 3], 0.0f64, 0.0f64);
    for p in px {
        let w = motif_mi(p) as f64;
        for c in 0..3 {
            m[c] += w * p[c] as f64;
            z[c] += (1.0 - w) * p[c] as f64;
        }
        wm += w;
        wz += 1.0 - w;
    }
    (z.map(|v| (v / wz) as f32), m.map(|v| (v / wm) as f32))
}

/// İki sınıflı yeniden renklendirme: dokuyu parlaklık oranıyla korur, sınıf geçişi yumuşak.
fn boya(
    g: &Rgb32FImage,
    motif_mi: Agirlik,
    hedef_zemin: [f32; 3],
    hedef_motif: [f32; 3],
) -> Rgb32FImage {
    let (mut lm, mut lz, mut wm, mut wz) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    for p in g.pixels() {
        let (w, l) = (motif_mi(p) as f64, parlaklik(p) as f64);
        lm += l * w;
        wm += w;
        lz += l * (1.0 - w);
        wz += 1.0 - w;
    }
    let (lm, lz) = ((lm / wm) as f32, (lz / wz) as f32);
    let mut c = g.clone();
    for p in c.pixels_mut() {
        let (w, l) = (motif_mi(p), parlaklik(p));
        *p =
            Rgb([0, 1, 2]
                .map(|k| w * hedef_motif[k] * l / lm + (1.0 - w) * hedef_zemin[k] * l / lz));
    }
    kirp_01(c)
}

/// Bant: doluluğu > %80 olan ardışık satır/sütunlar → (dış baş, iç baş, iç son, dış son).
fn bant_kenarlari(doluluk: &[f32]) -> (u32, u32, u32, u32) {
    let tam: Vec<bool> = doluluk.iter().map(|d| *d > 0.8).collect();
    let bas = tam.iter().position(|t| *t).expect("bant yok");
    let son = tam.iter().rposition(|t| *t).unwrap();
    let ic_bas = bas + tam[bas..].iter().position(|t| !t).unwrap_or(0);
    let ic_son = (0..=son).rev().find(|k| !tam[*k]).map_or(son, |k| k + 1);
    (bas as u32, ic_bas as u32, ic_son as u32, son as u32 + 1)
}

fn koseler(cer: &Rgb32FImage) -> ([Rgb32FImage; 4], (u32, u32)) {
    let (w, h) = cer.dimensions();
    let arka = *cer.get_pixel(5, 5);
    let dolu = |x, y| {
        let p = cer.get_pixel(x, y);
        (p[0] - arka[0]).abs() + (p[1] - arka[1]).abs() + (p[2] - arka[2]).abs() > 60.0 / 255.0
    };
    let sutun: Vec<f32> = (0..w)
        .map(|x| (0..h).filter(|y| dolu(x, *y)).count() as f32 / h as f32)
        .collect();
    let satir: Vec<f32> = (0..h)
        .map(|y| (0..w).filter(|x| dolu(*x, y)).count() as f32 / w as f32)
        .collect();
    let (x0, x1, x2, x3) = bant_kenarlari(&sutun);
    let (y0, y1, y2, y3) = bant_kenarlari(&satir);
    let k = |x: u32, y: u32, xs: u32, ys: u32| kirp(cer, x, y, xs - x, ys - y);
    // sıra: sol-üst, sağ-üst, sağ-alt, sol-alt (kilim-tema Kenarlik.koseler ile aynı)
    (
        [
            k(x0, y0, x1, y1),
            k(x2, y0, x3, y1),
            k(x2, y2, x3, y3),
            k(x0, y2, x1, y3),
        ],
        (x1 - x0, y1 - y0),
    )
}

const KOSE_ADLARI: [&str; 4] = [
    "kose-sol-ust",
    "kose-sag-ust",
    "kose-sag-alt",
    "kose-sol-alt",
];

pub fn kilim(
    kaynak: &Path,
    serit_b: &str,
    cerceve_a: &str,
    cerceve_b: &str,
    cikti: &Path,
    kalinlik: u32,
) {
    let serit = yukle(kaynak, serit_b);
    let (w, h) = serit.dimensions();
    let arka = *serit.get_pixel(5, 5);
    let bantli: Vec<u32> = (0..h)
        .filter(|y| {
            let dolu = (0..w)
                .filter(|x| {
                    let p = serit.get_pixel(*x, *y);
                    (p[0] - arka[0]).abs() + (p[1] - arka[1]).abs() + (p[2] - arka[2]).abs()
                        > 90.0 / 255.0
                })
                .count();
            dolu as f32 / w as f32 > 0.5
        })
        .collect();
    let (y0, y1) = (bantli[0], *bantli.last().unwrap() + 1);
    let p = w / 4;
    // ortadaki birim: görsel kenarındaki bozukluk yok
    let birim_b = kirp(&serit, p, y0, p, y1 - y0);
    let komsu = (0..birim_b.height())
        .flat_map(|y| (0..birim_b.width() - 1).map(move |x| (x, y)))
        .map(|(x, y)| {
            let (a, b) = (birim_b.get_pixel(x, y), birim_b.get_pixel(x + 1, y));
            ((a[0] - b[0]).abs() + (a[1] - b[1]).abs() + (a[2] - b[2]).abs()) / 3.0
        })
        .sum::<f32>()
        / ((birim_b.width() - 1) * birim_b.height()) as f32;
    let sinir = (y0..y1)
        .map(|y| {
            let (a, b) = (serit.get_pixel(p, y), serit.get_pixel(2 * p, y));
            ((a[0] - b[0]).abs() + (a[1] - b[1]).abs() + (a[2] - b[2]).abs()) / 3.0
        })
        .sum::<f32>()
        / (y1 - y0) as f32;
    let dikis_orani = sinir / komsu;
    let (zemin_b, motif_b) = sinif_ortalamalari(birim_b.pixels(), yesil_mi);

    let (kos_a, kal_a) = koseler(&yukle(kaynak, cerceve_a));
    let (kos_b, kal_b) = koseler(&yukle(kaynak, cerceve_b));
    let (zemin_a, motif_a) = sinif_ortalamalari(kos_a.iter().flat_map(|k| k.pixels()), altin_mi);

    let uzun = (birim_b.width() as f32 * kalinlik as f32 / birim_b.height() as f32).round() as u32;
    // A: B şeridi A renklerine boyanır (geometri birebir aynı); B: parlak yeşil köşeler şeride uyar.
    let varyantlar = [
        ("kilim-a", boya(&birim_b, yesil_mi, zemin_a, motif_a), kos_a),
        (
            "kilim-b",
            birim_b.clone(),
            kos_b.map(|k| boya(&k, yesil_mi, zemin_b, motif_b)),
        ),
    ];
    for (ad, kenar, kos) in &varyantlar {
        kaydet(
            &kucult(kenar, uzun, kalinlik),
            &cikti.join(ad).join("kenar.png"),
        );
        for (k, dosya) in kos.iter().zip(KOSE_ADLARI) {
            kaydet(
                &kucult(k, kalinlik, kalinlik),
                &cikti.join(ad).join(format!("{dosya}.png")),
            );
        }
    }
    println!(
        "kilim: birim {}x{} → {uzun}x{kalinlik}, birim sınırı dikiş oranı {dikis_orani:.2}; çerçeve bant kalınlığı A {kal_a:?} B {kal_b:?}",
        birim_b.width(),
        birim_b.height()
    );
    println!(
        "  A zemin {} motif {} | B zemin {} motif {}",
        hex(zemin_a),
        hex(motif_a),
        hex(zemin_b),
        hex(motif_b)
    );
    assert!(dikis_orani < 1.5, "şerit birimi dikişsiz birleşmiyor");
}

#[cfg(test)]
mod sinama {
    use super::*;

    #[test]
    fn bant_dis_ve_ic_kenarlari() {
        // dış boşluk, 3 dolu satır (bant), iç boşluk, 2 dolu satır, dış boşluk
        let d = [0.0, 1.0, 1.0, 1.0, 0.3, 0.3, 1.0, 1.0, 0.0];
        assert_eq!(bant_kenarlari(&d), (1, 4, 6, 8));
    }
}
