use egui_kittest::{kittest::Queryable, Harness};
use kilim_tema::{pencere_dugmesi, Aydinlik, Gorunum, PencereDugmesi as P, TemaTercihi, Varyant};

#[test]
fn pencere_dugmeleri_tum_temalarda_erisilebilir_ve_etkin() {
    for gorunum in [Gorunum::Sade, Gorunum::Kilim] {
        for aydinlik in [Aydinlik::Acik, Aydinlik::Koyu] {
            let mut h = Harness::new_ui_state(
                move |ui, secim: &mut Option<P>| {
                    let mut t = TemaTercihi::varsayilan(Varyant::CamGobegiAltin);
                    t.gorunum = gorunum;
                    t.aydinlik = aydinlik;
                    kilim_tema::giydir(ui.ctx(), &t);
                    ui.horizontal(|ui| {
                        for tur in [P::Tasi, P::Buyut, P::GeriAl, P::Gizle] {
                            if pencere_dugmesi(ui, tur, 24.0).clicked() {
                                *secim = Some(tur);
                            }
                        }
                    });
                },
                None,
            );
            for tur in [P::Tasi, P::Buyut, P::GeriAl, P::Gizle] {
                h.get_by_label(tur.etiket()).click_accesskit();
                h.run();
                assert_eq!(*h.state(), Some(tur));
            }
        }
    }
}
