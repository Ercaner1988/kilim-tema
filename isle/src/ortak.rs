//! Görüntü yardımcıları. Bütün değerler 0..1 aralığında f32.

use std::path::Path;

use image::imageops::{self, FilterType};
use image::{DynamicImage, ImageBuffer, Rgb, Rgb32FImage, Rgba, Rgba32FImage};

pub const ZEM: [f32; 3] = [
    0xF4 as f32 / 255.0,
    0xEC as f32 / 255.0,
    0xDB as f32 / 255.0,
];
pub const MUREKKEP: [f32; 3] = [
    0x3A as f32 / 255.0,
    0x2D as f32 / 255.0,
    0x21 as f32 / 255.0,
];

pub fn yukle(kok: &Path, ad: &str) -> Rgb32FImage {
    let yol = kok.join(format!("Gemini_Generated_Image_{ad}.jpg"));
    image::open(&yol)
        .unwrap_or_else(|e| panic!("{} açılamadı: {e}", yol.display()))
        .to_rgb32f()
}

pub fn parlaklik(p: &Rgb<f32>) -> f32 {
    0.299 * p[0] + 0.587 * p[1] + 0.114 * p[2]
}

pub fn wcag(a: [f32; 3], b: [f32; 3]) -> f32 {
    let l = |c: [f32; 3]| {
        let d = |v: f32| {
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * d(c[0]) + 0.7152 * d(c[1]) + 0.0722 * d(c[2])
    };
    let (x, y) = (l(a).min(l(b)), l(a).max(l(b)));
    (y + 0.05) / (x + 0.05)
}

pub fn kucult(g: &Rgb32FImage, w: u32, h: u32) -> Rgb32FImage {
    kirp_01(imageops::resize(g, w, h, FilterType::Lanczos3))
}

/// Saydam kenarda koyu hale olmasın diye önçarpımlı küçültme.
pub fn kucult_saydam(g: &Rgba32FImage, w: u32, h: u32) -> Rgba32FImage {
    let mut on = g.clone();
    for p in on.pixels_mut() {
        for c in 0..3 {
            p[c] *= p[3];
        }
    }
    let mut k = imageops::resize(&on, w, h, FilterType::Lanczos3);
    for p in k.pixels_mut() {
        p[3] = p[3].clamp(0.0, 1.0);
        for c in 0..3 {
            p[c] = if p[3] > 0.0 {
                (p[c] / p[3]).clamp(0.0, 1.0)
            } else {
                0.0
            };
        }
    }
    k
}

pub fn kirp_01(mut g: Rgb32FImage) -> Rgb32FImage {
    for p in g.pixels_mut() {
        for c in 0..3 {
            p[c] = p[c].clamp(0.0, 1.0);
        }
    }
    g
}

pub fn kaydet(g: &Rgb32FImage, yol: &Path) {
    std::fs::create_dir_all(yol.parent().unwrap()).unwrap();
    DynamicImage::ImageRgb32F(kirp_01(g.clone()))
        .to_rgb8()
        .save(yol)
        .unwrap();
}

pub fn kaydet_saydam(g: &Rgba32FImage, yol: &Path) {
    std::fs::create_dir_all(yol.parent().unwrap()).unwrap();
    DynamicImage::ImageRgba32F(g.clone())
        .to_rgba8()
        .save(yol)
        .unwrap();
}

pub fn kaydet_jpeg(g: &Rgb32FImage, yol: &Path, kalite: u8) {
    let dosya = std::fs::File::create(yol).unwrap();
    let rgb = DynamicImage::ImageRgb32F(kirp_01(g.clone())).to_rgb8();
    image::codecs::jpeg::JpegEncoder::new_with_quality(dosya, kalite)
        .encode_image(&rgb)
        .unwrap();
}

pub fn kirp(g: &Rgb32FImage, x: u32, y: u32, w: u32, h: u32) -> Rgb32FImage {
    imageops::crop_imm(g, x, y, w, h).to_image()
}

pub fn ortalama<'a>(px: impl Iterator<Item = &'a Rgb<f32>>) -> [f32; 3] {
    let (mut t, mut n) = ([0.0f64; 3], 0usize);
    for p in px {
        for c in 0..3 {
            t[c] += p[c] as f64;
        }
        n += 1;
    }
    t.map(|v| (v / n as f64) as f32)
}

pub fn hex(c: [f32; 3]) -> String {
    format!(
        "#{:02X}{:02X}{:02X}",
        (c[0] * 255.0).round() as u8,
        (c[1] * 255.0).round() as u8,
        (c[2] * 255.0).round() as u8
    )
}

pub fn bos_saydam(w: u32, h: u32) -> Rgba32FImage {
    ImageBuffer::from_pixel(w, h, Rgba([0.0; 4]))
}
