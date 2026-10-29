use kilim_tema::{IslevTanimi, Rol, TemaBetigi};

#[test]
fn betik_cepheden_cozulur_kurulur_ve_renk_tasiyir() {
    let betik = TemaBetigi::coz(
        r##"{"sema":1,"kurallar":[{"rol":"DugmeBirincil","normal":{"dolgu":"#123456"}}]}"##,
    )
    .expect("geçerli betik çözülmeli");
    let ctx = egui::Context::default();
    betik
        .kur(&ctx)
        .expect("doğrulanmış betik Context'e kurulmalı");
    let mut out = ctx.run_ui(Default::default(), |ui| {
        kilim_tema::islev(
            ui,
            IslevTanimi {
                id: "ara",
                rol: Rol::DugmeBirincil,
                komut: Some("ara"),
            },
            |ui| ui.button("Ara"),
        );
    });
    out.textures_delta.clear();
    assert!(out.shapes.iter().any(|shape| matches!(
        &shape.shape,
        egui::Shape::Rect(rect) if rect.fill == egui::Color32::from_rgb(0x12, 0x34, 0x56)
    )));
}
