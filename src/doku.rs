//! Gömülü dokular (Desktop\tema\isle.py üretir) ve bir kez yüklenip Context'te saklanmaları.

use std::sync::Arc;

use egui::{
    ColorImage, Context, Id, TextureFilter, TextureHandle, TextureOptions, TextureWrapMode,
};

use crate::{Boncuk, Varyant};

pub struct Kenarlik {
    pub kenar: TextureHandle,
    /// sol-üst, sağ-üst, sağ-alt, sol-alt
    pub koseler: [TextureHandle; 4],
}

pub struct Dokular {
    pub kagit: TextureHandle,
    pub leke: TextureHandle,
    kilim_a: Kenarlik,
    kilim_b: Kenarlik,
    boncuklar: Vec<TextureHandle>,
}

impl Dokular {
    pub fn kenarlik(&self, v: Varyant) -> &Kenarlik {
        match v {
            Varyant::CamGobegiAltin => &self.kilim_a,
            Varyant::KirmiziYesil => &self.kilim_b,
        }
    }

    pub fn boncuk(&self, b: Boncuk) -> &TextureHandle {
        &self.boncuklar[b as usize]
    }
}

fn yukle(ctx: &Context, ad: &str, bayt: &[u8], dosemeli: bool) -> TextureHandle {
    let img = image::load_from_memory(bayt)
        .expect("gömülü tema varlığı çözülemedi")
        .to_rgba8();
    let boyut = [img.width() as usize, img.height() as usize];
    let opt = TextureOptions {
        magnification: TextureFilter::Linear,
        minification: TextureFilter::Linear,
        wrap_mode: if dosemeli {
            TextureWrapMode::Repeat
        } else {
            TextureWrapMode::ClampToEdge
        },
        mipmap_mode: Some(TextureFilter::Linear),
    };
    ctx.load_texture(
        ad,
        ColorImage::from_rgba_unmultiplied(boyut, img.as_raw()),
        opt,
    )
}

macro_rules! varlik {
    ($yol:expr) => {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $yol))
    };
}

macro_rules! kenarlik {
    ($ctx:expr, $v:literal) => {
        Kenarlik {
            kenar: yukle(
                $ctx,
                concat!($v, "/kenar"),
                varlik!(concat!($v, "/kenar.png")),
                true,
            ),
            koseler: [
                yukle(
                    $ctx,
                    concat!($v, "/sol-ust"),
                    varlik!(concat!($v, "/kose-sol-ust.png")),
                    false,
                ),
                yukle(
                    $ctx,
                    concat!($v, "/sag-ust"),
                    varlik!(concat!($v, "/kose-sag-ust.png")),
                    false,
                ),
                yukle(
                    $ctx,
                    concat!($v, "/sag-alt"),
                    varlik!(concat!($v, "/kose-sag-alt.png")),
                    false,
                ),
                yukle(
                    $ctx,
                    concat!($v, "/sol-alt"),
                    varlik!(concat!($v, "/kose-sol-alt.png")),
                    false,
                ),
            ],
        }
    };
}

/// İlk çağrıda dokuları GPU'ya yükler, sonra Context'teki kopyayı döndürür.
pub fn dokular(ctx: &Context) -> Arc<Dokular> {
    let id = Id::new("kilim-tema/dokular");
    if let Some(d) = ctx.data(|m| m.get_temp::<Arc<Dokular>>(id)) {
        return d;
    }
    let boncuk = |ad: &str, bayt: &[u8]| yukle(ctx, ad, bayt, false);
    let d = Arc::new(Dokular {
        kagit: yukle(ctx, "kagit", varlik!("kagit-karo.jpg"), true),
        leke: yukle(ctx, "leke", varlik!("leke-ortu.png"), false),
        kilim_a: kenarlik!(ctx, "kilim-a"),
        kilim_b: kenarlik!(ctx, "kilim-b"),
        // Sıra `Boncuk` enum'uyla aynı olmalı.
        boncuklar: vec![
            boncuk("firuze", varlik!("boncuk/firuze.png")),
            boncuk("akik", varlik!("boncuk/akik.png")),
            boncuk("kehribar", varlik!("boncuk/kehribar.png")),
            boncuk("lapis", varlik!("boncuk/lapis.png")),
            boncuk("telkari", varlik!("boncuk/telkari.png")),
            boncuk("sirma", varlik!("boncuk/sirma.png")),
            boncuk("nazar", varlik!("boncuk/nazar.png")),
            boncuk("tesbih", varlik!("boncuk/tesbih.png")),
            boncuk("sedef", varlik!("boncuk/sedef.png")),
        ],
    });
    ctx.data_mut(|m| m.insert_temp(id, d.clone()));
    d
}
