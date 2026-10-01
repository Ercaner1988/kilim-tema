//! Tema tercihi: seçici menü + uygulama başına küçük metin dosyasında saklama.

use std::path::PathBuf;

use egui::{ThemePreference, Ui};

use crate::Varyant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gorunum {
    /// Semerkant kâğıdı + kilim kenarlık.
    Kilim,
    /// Dokusuz: yalnız palet (zayıf GPU, sade tercih).
    Sade,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Aydinlik {
    Sistem,
    Acik,
    Koyu,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TemaTercihi {
    pub gorunum: Gorunum,
    pub varyant: Varyant,
    pub aydinlik: Aydinlik,
}

impl TemaTercihi {
    pub fn varsayilan(varyant: Varyant) -> Self {
        Self {
            gorunum: Gorunum::Kilim,
            varyant,
            aydinlik: Aydinlik::Sistem,
        }
    }

    pub fn kod(&self) -> String {
        let g = match self.gorunum {
            Gorunum::Kilim => "kilim",
            Gorunum::Sade => "sade",
        };
        let v = match self.varyant {
            Varyant::CamGobegiAltin => "a",
            Varyant::KirmiziYesil => "b",
        };
        let a = match self.aydinlik {
            Aydinlik::Sistem => "sistem",
            Aydinlik::Acik => "acik",
            Aydinlik::Koyu => "koyu",
        };
        format!("gorunum={g}\nvaryant={v}\naydinlik={a}\n")
    }

    /// Bilinmeyen/eksik satırlar `varsayilan`daki değeri korur.
    pub fn coz(metin: &str, varsayilan: Self) -> Self {
        let mut t = varsayilan;
        for satir in metin.lines() {
            match satir.trim().split_once('=') {
                Some(("gorunum", "kilim")) => t.gorunum = Gorunum::Kilim,
                Some(("gorunum", "sade")) => t.gorunum = Gorunum::Sade,
                Some(("varyant", "a")) => t.varyant = Varyant::CamGobegiAltin,
                Some(("varyant", "b")) => t.varyant = Varyant::KirmiziYesil,
                Some(("aydinlik", "sistem")) => t.aydinlik = Aydinlik::Sistem,
                Some(("aydinlik", "acik")) => t.aydinlik = Aydinlik::Acik,
                Some(("aydinlik", "koyu")) => t.aydinlik = Aydinlik::Koyu,
                _ => {}
            }
        }
        t
    }

    pub fn yukle(uygulama: &str, varsayilan: Self) -> Self {
        dosya(uygulama)
            .and_then(|d| std::fs::read_to_string(d).ok())
            .map_or(varsayilan, |m| Self::coz(&m, varsayilan))
    }

    /// Kaydedemezse sessizce geçer: tercih yalnız kolaylıktır, uygulama çalışmaya devam eder.
    pub fn kaydet(&self, uygulama: &str) {
        if let Some(d) = dosya(uygulama) {
            let _ = d.parent().map(std::fs::create_dir_all);
            let _ = std::fs::write(d, self.kod());
        }
    }

    /// Açık/koyu tercihini egui'ye bildirir; uygulamaların paleti `ctx.theme()`'i izler.
    pub fn uygula(&self, ctx: &egui::Context) {
        ctx.set_theme(match self.aydinlik {
            Aydinlik::Sistem => ThemePreference::System,
            Aydinlik::Acik => ThemePreference::Light,
            Aydinlik::Koyu => ThemePreference::Dark,
        });
    }
}

fn dosya(uygulama: &str) -> Option<PathBuf> {
    let kok = if cfg!(windows) {
        std::env::var_os("APPDATA").map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Application Support"))
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
    }?;
    Some(kok.join("kilim-tema").join(format!("{uygulama}.txt")))
}

/// "Tema" menüsü. Değişiklik olursa `uygula` çağrılmış olarak `true` döner (kaydetmek çağırana kalır).
pub fn tema_secici(ui: &mut Ui, tercih: &mut TemaTercihi) -> bool {
    let once = *tercih;
    let dugme = crate::boncuklu(ui, crate::Boncuk::Kehribar, "Tema");
    egui::containers::menu::MenuButton::from_button(dugme).ui(ui, |ui| {
        ui.label("Görünüm");
        crate::secim(
            ui,
            &mut tercih.gorunum,
            Gorunum::Kilim,
            "Kilim (kâğıt + kenarlık)",
        );
        crate::secim(ui, &mut tercih.gorunum, Gorunum::Sade, "Sade (dokusuz)");
        ui.separator();
        ui.add_enabled_ui(tercih.gorunum == Gorunum::Kilim, |ui| {
            ui.label("Kenarlık");
            crate::secim(
                ui,
                &mut tercih.varyant,
                Varyant::CamGobegiAltin,
                "Cam göbeği + altın",
            );
            crate::secim(
                ui,
                &mut tercih.varyant,
                Varyant::KirmiziYesil,
                "Kırmızı + yeşil",
            );
        });
        ui.separator();
        ui.label("Aydınlık");
        crate::secim(ui, &mut tercih.aydinlik, Aydinlik::Sistem, "Sistemi izle");
        crate::secim(ui, &mut tercih.aydinlik, Aydinlik::Acik, "Açık");
        crate::secim(ui, &mut tercih.aydinlik, Aydinlik::Koyu, "Koyu");
    });
    let degisti = *tercih != once;
    if degisti {
        tercih.uygula(ui.ctx());
    }
    degisti
}

#[cfg(test)]
mod sinama {
    use super::*;

    #[test]
    fn kod_gidip_gelir_ve_bozuk_satir_varsayilani_korur() {
        let t = TemaTercihi {
            gorunum: Gorunum::Sade,
            varyant: Varyant::KirmiziYesil,
            aydinlik: Aydinlik::Koyu,
        };
        let v = TemaTercihi::varsayilan(Varyant::CamGobegiAltin);
        assert_eq!(TemaTercihi::coz(&t.kod(), v), t);
        let yarim = TemaTercihi::coz("varyant=b\nbozuk satır\naydinlik=mor\n", v);
        assert_eq!(
            yarim,
            TemaTercihi {
                varyant: Varyant::KirmiziYesil,
                ..v
            }
        );
        assert_eq!(TemaTercihi::coz("", v), v);
    }
}
