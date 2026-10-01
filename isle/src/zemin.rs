//! Kâğıt karosu (döşenir) ve leke örtüsü (gerilir).

use std::path::Path;

use image::{ImageBuffer, Rgb32FImage, Rgba};

use crate::ortak::*;

/// Kenar boyunca (yatay: sütunlar arası, dikey: satırlar arası) `p` uzaklıktaki
/// piksellerin ortalama farkı. Kanal ortalaması.
fn kayma_farki(g: &Rgb32FImage, yatay: bool, p: u32) -> f32 {
    let (w, h) = g.dimensions();
    let (mut t, mut n) = (0.0f64, 0u64);
    for y in 0..h - if yatay { 0 } else { p } {
        for x in 0..w - if yatay { p } else { 0 } {
            let (a, b) = (
                g.get_pixel(x, y),
                if yatay {
                    g.get_pixel(x + p, y)
                } else {
                    g.get_pixel(x, y + p)
                },
            );
            t += ((a[0] - b[0]).abs() + (a[1] - b[1]).abs() + (a[2] - b[2]).abs()) as f64 / 3.0;
            n += 1;
        }
    }
    (t / n as f64) as f32
}

/// Gemini karoyu kendi içinde tekrar edebiliyor (2x2, 2x1): en küçük özgün birim.
fn birim(g: &Rgb32FImage, yatay: bool) -> u32 {
    let n = if yatay { g.width() } else { g.height() };
    let komsu = kayma_farki(g, yatay, 1);
    for bolen in [4, 3, 2] {
        let p = n / bolen;
        if kayma_farki(g, yatay, p) < 0.7 * komsu {
            return p;
        }
    }
    n
}

/// Karşılıklı kenarların farkı / komşu piksel farkı. ≈1 dikişsiz.
pub fn kenar_dikisi(g: &Rgb32FImage, yatay: bool) -> f32 {
    let (w, h) = g.dimensions();
    let (mut t, mut n) = (0.0f64, 0u64);
    let m = if yatay { h } else { w };
    for i in 0..m {
        let (a, b) = if yatay {
            (g.get_pixel(w - 1, i), g.get_pixel(0, i))
        } else {
            (g.get_pixel(i, h - 1), g.get_pixel(i, 0))
        };
        t += ((a[0] - b[0]).abs() + (a[1] - b[1]).abs() + (a[2] - b[2]).abs()) as f64 / 3.0;
        n += 1;
    }
    (t / n as f64) as f32 / kayma_farki(g, yatay, 1)
}

/// Kenar şeridini yarım kaydırılmış kopyayla harmanla: kopyanın kenarı özgünün ortası, sarmada sürekli.
fn dikissiz_yap(g: &Rgb32FImage, yatay: bool, bant: f32) -> Rgb32FImage {
    let (w, h) = g.dimensions();
    let n = if yatay { w } else { h };
    ImageBuffer::from_fn(w, h, |x, y| {
        let i = if yatay { x } else { y };
        let a = (i.min(n - 1 - i) as f32 / bant).clamp(0.0, 1.0);
        let (kx, ky) = if yatay {
            ((x + n / 2) % n, y)
        } else {
            (x, (y + n / 2) % n)
        };
        let (p, q) = (g.get_pixel(x, y), g.get_pixel(kx, ky));
        image::Rgb([0, 1, 2].map(|c| p[c] * a + q[c] * (1.0 - a)))
    })
}

/// Tek tek koyu noktaları (lifler ~180/255 üstü, noktalar 165 altı) sarmalı 15x15 medyanla doldur.
fn noktalari_sil(g: &mut Rgb32FImage) -> usize {
    let (w, h) = g.dimensions();
    let koyu: Vec<bool> = g.pixels().map(|p| parlaklik(p) < 165.0 / 255.0).collect();
    let sar = |v: i64, n: u32| v.rem_euclid(n as i64) as u32;
    let mut maske = vec![false; koyu.len()];
    for y in 0..h {
        for x in 0..w {
            if koyu[(y * w + x) as usize] {
                for dy in -3..=3i64 {
                    for dx in -3..=3i64 {
                        maske[(sar(y as i64 + dy, h) * w + sar(x as i64 + dx, w)) as usize] = true;
                    }
                }
            }
        }
    }
    let ozgun = g.clone();
    let mut pencere = Vec::with_capacity(225);
    for (i, _) in maske.iter().enumerate().filter(|(_, m)| **m) {
        let (x, y) = (i as u32 % w, i as u32 / w);
        let mut yeni = [0.0; 3];
        for (c, v) in yeni.iter_mut().enumerate() {
            pencere.clear();
            for dy in -7..=7i64 {
                for dx in -7..=7i64 {
                    pencere.push(ozgun.get_pixel(sar(x as i64 + dx, w), sar(y as i64 + dy, h))[c]);
                }
            }
            pencere.sort_by(f32::total_cmp);
            *v = pencere[pencere.len() / 2];
        }
        g.put_pixel(x, y, image::Rgb(yeni));
    }
    maske.iter().filter(|m| **m).count()
}

pub fn kagit(kaynak: &Path, ad: &str, cikti: &Path) {
    let ham = yukle(kaynak, ad);
    let mut k = kirp(&ham, 0, 0, birim(&ham, true), birim(&ham, false));
    for yatay in [false, true] {
        if kenar_dikisi(&k, yatay) > 1.4 {
            k = dikissiz_yap(&k, yatay, 96.0);
        }
    }
    let silinen = noktalari_sil(&mut k);
    let ort = ortalama(k.pixels());
    for p in k.pixels_mut() {
        for c in 0..3 {
            p[c] = (p[c] * ZEM[c] / ort[c]).clamp(0.0, 1.0);
        }
    }
    kaydet_jpeg(&k, &cikti.join("kagit-karo.jpg"), 90);
    let dk = (kenar_dikisi(&k, false), kenar_dikisi(&k, true));
    println!(
        "kâğıt: birim {}x{}, dikiş üst-alt {:.2} sol-sağ {:.2} (≈1 dikişsiz), silinen nokta pikseli {silinen}, ortalama {}",
        k.width(),
        k.height(),
        dk.0,
        dk.1,
        hex(ortalama(k.pixels()))
    );
    assert!(dk.0.max(dk.1) < 1.4, "karo dikişi görünür");
}

/// egui'de çarpma kipi yok, yalnız alfa "üstüne" var. zem üstünde çarpmayla birebir aynı
/// sonucu veren örtü: a = 1 - min(s), C = zem*(s - min s)/a. Lifli yerlerde yaklaşık.
pub fn leke(kaynak: &Path, ad: &str, cikti: &Path, opaklik: f32) {
    let mut s = yukle(kaynak, ad);
    let (w, h) = s.dimensions();
    let merkez = ortalama(kirp(&s, w / 3, h / 3, w / 3, h / 3).pixels());
    for p in s.pixels_mut() {
        for c in 0..3 {
            p[c] = (1.0 - opaklik) + opaklik * (p[c] / merkez[c]).clamp(0.0, 1.0);
        }
    }
    let sk = kucult(&s, w / 4, h / 4);
    let mut sapma = 0.0f32;
    let ortu = ImageBuffer::from_fn(sk.width(), sk.height(), |x, y| {
        let v = sk.get_pixel(x, y);
        let mn = v[0].min(v[1]).min(v[2]);
        let a = 1.0 - mn;
        let renk = [0, 1, 2].map(|c| ZEM[c] * (v[c] - mn) / a.max(1e-6));
        for c in 0..3 {
            sapma = sapma.max((ZEM[c] * (1.0 - a) + renk[c] * a - ZEM[c] * v[c]).abs() * 255.0);
        }
        Rgba([renk[0], renk[1], renk[2], a])
    });
    kaydet_saydam(&ortu, &cikti.join("leke-ortu.png"));
    let mut sirali: Vec<(f32, [f32; 3])> = sk
        .pixels()
        .map(|p| (parlaklik(p), [p[0], p[1], p[2]]))
        .collect();
    sirali.sort_by(|a, b| a.0.total_cmp(&b.0));
    let dilim = &sirali[..(sirali.len() / 200).max(1)];
    let en_koyu = [0, 1, 2].map(|c| dilim.iter().map(|p| p.1[c]).sum::<f32>() / dilim.len() as f32);
    let k = wcag(MUREKKEP, [0, 1, 2].map(|c| ZEM[c] * en_koyu[c]));
    println!(
        "leke: {}x{}, örtü sapması {sapma:.2}/255, en kavruk yerde mürekkep kontrastı {k:.2}",
        sk.width(),
        sk.height()
    );
    assert!(sapma < 1.5, "örtü çarpmayı karşılamıyor");
    assert!(k >= 4.5, "leke metni okunmaz yapıyor");
}
