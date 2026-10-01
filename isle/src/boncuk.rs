//! Boncuklar: 3x3 ızgaradan magenta ayıklama, kırpma, 64 px.

use std::path::Path;

use image::Rgba;

use crate::ortak::*;

pub const BONCUKLAR: [&str; 9] = [
    "firuze", "akik", "kehribar", "lapis", "telkari", "sirma", "nazar", "tesbih", "sedef",
];

fn medyan(mut v: Vec<f32>) -> f32 {
    v.sort_by(f32::total_cmp);
    v[v.len() / 2]
}

pub fn boncuklar(kaynak: &Path, ad: &str, cikti: &Path, boyut: u32) {
    let b = yukle(kaynak, ad);
    let (w, h) = b.dimensions();
    let kose: Vec<_> = [(0, 0), (w - 20, 0), (0, h - 20), (w - 20, h - 20)]
        .iter()
        .flat_map(|&(x0, y0)| (0..20).flat_map(move |y| (0..20).map(move |x| (x0 + x, y0 + y))))
        .map(|(x, y)| *b.get_pixel(x, y))
        .collect();
    let m = [0, 1, 2].map(|c| medyan(kose.iter().map(|p| p[c]).collect()));
    // alfa: magentadan uzaklık 40..120 (/255) arası yumuşak; renk: magenta katkısı geri çıkarılır
    let alfa = |x, y| {
        let p = b.get_pixel(x, y);
        let d =
            ((p[0] - m[0]).powi(2) + (p[1] - m[1]).powi(2) + (p[2] - m[2]).powi(2)).sqrt() * 255.0;
        ((d - 40.0) / 80.0).clamp(0.0, 1.0)
    };
    let n = h / 3;
    let mut sacak = 0;
    for (i, ad) in BONCUKLAR.iter().enumerate() {
        let (ox, oy) = ((i as u32 % 3) * n, (i as u32 / 3) * n);
        let ici: Vec<(u32, u32)> = (0..n)
            .flat_map(|y| (0..n).map(move |x| (x, y)))
            .filter(|&(x, y)| alfa(ox + x, oy + y) > 0.5)
            .collect();
        let (x0, x1) = (
            ici.iter().map(|p| p.0).min().unwrap(),
            ici.iter().map(|p| p.0).max().unwrap() + 1,
        );
        let (y0, y1) = (
            ici.iter().map(|p| p.1).min().unwrap(),
            ici.iter().map(|p| p.1).max().unwrap() + 1,
        );
        let kenar = (x1 - x0).max(y1 - y0) + 4;
        let (px, py) = ((kenar - (x1 - x0)) / 2, (kenar - (y1 - y0)) / 2);
        let mut tuval = bos_saydam(kenar, kenar);
        for y in y0..y1 {
            for x in x0..x1 {
                let (sx, sy) = (ox + x, oy + y);
                let a = alfa(sx, sy);
                let p = b.get_pixel(sx, sy);
                let renk = [0, 1, 2].map(|c| (m[c] + (p[c] - m[c]) / a.max(1e-3)).clamp(0.0, 1.0));
                let magentamsi =
                    renk[0] > 150.0 / 255.0 && renk[2] > 150.0 / 255.0 && renk[1] < 90.0 / 255.0;
                // ayıklanamayan yarı saydam saçak: kenardan kırp
                let a = if magentamsi && a < 1.0 { 0.0 } else { a };
                if magentamsi && a > 0.0 && a < 1.0 {
                    sacak += 1;
                }
                tuval.put_pixel(
                    px + x - x0,
                    py + y - y0,
                    Rgba([renk[0], renk[1], renk[2], a]),
                );
            }
        }
        kaydet_saydam(
            &kucult_saydam(&tuval, boyut, boyut),
            &cikti.join("boncuk").join(format!("{ad}.png")),
        );
    }
    println!(
        "boncuk: {} adet, arka plan {}, kalan magenta saçak pikseli {sacak}",
        BONCUKLAR.len(),
        hex(m)
    );
    assert!(sacak < 50, "magenta saçağı kalmış");
}
