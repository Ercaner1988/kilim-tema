//! Tarayıcının önerdiği rol adları kilim-tema'daki `Rol` ile birebir aynı kalmalı (tek kaynak: `Rol::HEPSI`).

use kilim_tarama::kural::ROLLER;

#[test]
fn tarayici_rol_adlari_kilim_tema_ile_ayni() {
    let tema: Vec<&str> = kilim_tema::Rol::HEPSI.iter().map(|r| r.ad()).collect();
    assert_eq!(tema, ROLLER);
}
