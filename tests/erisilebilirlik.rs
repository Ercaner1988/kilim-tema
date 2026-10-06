//! AccessKit (ekran okuyucu, UI Automation, sınama) "tıkla" eylemi radyo ve kutuda çalışmalı.

use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;

#[derive(PartialEq, Clone, Copy, Debug)]
enum Renk {
    Kirmizi,
    Mavi,
}

#[test]
fn radyo_ve_kutu_accesskit_tiklamasina_yanit_verir() {
    let mut h = Harness::new_ui_state(
        |ui, (r, k): &mut (Renk, bool)| {
            kilim_tema::secim(ui, r, Renk::Kirmizi, "Kırmızı");
            kilim_tema::secim(ui, r, Renk::Mavi, "Mavi");
            kilim_tema::kutu(ui, k, "Kalın");
        },
        (Renk::Kirmizi, false),
    );
    h.get_by_label("Mavi").click_accesskit();
    h.run();
    assert_eq!(h.state().0, Renk::Mavi);
    h.get_by_label("Kalın").click_accesskit();
    h.run();
    assert!(h.state().1);
}

/// Tema menüsü: AccessKit ile seçilir (H7) ve seçimde açık kalır (H9): üç ayar tek açılışta.
#[test]
fn tema_menusu_secimde_acik_kalir() {
    use kilim_tema::{Aydinlik, Gorunum, TemaTercihi, Varyant};
    let mut h = Harness::new_ui_state(
        |ui, t: &mut TemaTercihi| {
            kilim_tema::tema_secici(ui, t);
        },
        TemaTercihi::varsayilan(Varyant::CamGobegiAltin),
    );
    h.get_by_label("Tema").click();
    h.run();
    h.get_by_label("Koyu").click_accesskit();
    h.run();
    assert_eq!(h.state().aydinlik, Aydinlik::Koyu, "AccessKit tıklaması");
    h.get_by_label("Kırmızı + yeşil").click();
    h.run();
    assert_eq!(
        h.state().varyant,
        Varyant::KirmiziYesil,
        "menü açık kalmalı"
    );
    h.get_by_label("Sade (dokusuz)").click();
    h.run();
    assert_eq!(h.state().gorunum, Gorunum::Sade);
}
