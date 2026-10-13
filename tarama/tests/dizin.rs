//! Dizin yürüyüşü: test/örnek/target atlanır, sandık bağımlılığı en yakın Cargo.toml'dan okunur,
//! araç düzeyi kurulum talebi `run_native` varken `giydir` yoksa çıkar.

use std::fs;

use kilim_tarama::rapor::{dizin_tara, plan_metni, talep_metni, Arac};

fn dosya(kok: &std::path::Path, yol: &str, icerik: &str) {
    let p = kok.join(yol);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, icerik).unwrap();
}

#[test]
fn dizin_taramasi_atlar_bagimliligi_bulur_kurulum_ister() {
    let kok = std::env::temp_dir().join(format!("kilim-tarama-sinama-{}", std::process::id()));
    let _ = fs::remove_dir_all(&kok);
    let panel = "fn f(ui: &mut egui::Ui) {\n    egui::Panel::left(\"a\").show(ui, |ui| {});\n}\n";
    dosya(
        &kok,
        "uygulama/Cargo.toml",
        "[dependencies]\nkilim-tema = { path = \"x\" }\n",
    );
    dosya(
        &kok,
        "uygulama/src/main.rs",
        &format!("fn main() {{ eframe::run_native(\"a\", o, b); }}\n{panel}"),
    );
    dosya(
        &kok,
        "kitaplik/Cargo.toml",
        "[dependencies]\negui = \"0.36\"\n",
    );
    dosya(&kok, "kitaplik/src/lib.rs", panel);
    // Atlanması gerekenler:
    dosya(&kok, "kitaplik/tests/t.rs", panel);
    dosya(&kok, "kitaplik/target/debug/x.rs", panel);
    dosya(&kok, "kitaplik/src/sinama.rs", panel);

    let d = dizin_tara(&kok, false).unwrap();
    let yollar: Vec<&str> = d.iter().map(|x| x.yol.as_str()).collect();
    assert_eq!(
        yollar,
        ["kitaplik/src/lib.rs", "uygulama/src/main.rs"],
        "yalnız üretim kaynakları"
    );
    let bag: Vec<bool> = d.iter().map(|x| x.kilim_bagimli).collect();
    assert_eq!(bag, [false, true], "bağımlılık en yakın Cargo.toml'dan");

    let arac = Arac {
        ad: "deneme".into(),
        dosyalar: d,
    };
    let talep = talep_metni(&arac);
    assert!(talep.contains("[KURULUM]"), "{talep}");
    assert!(
        talep.contains("bağımlılığı YOK"),
        "kitaplık için bağımlılık uyarısı: {talep}"
    );
    assert_eq!(talep.matches("[ROL]").count(), 2, "{talep}");
    let plan = plan_metni(&arac);
    assert!(
        plan.contains("Rol taşıyabilen bölge: 2 (işaretli 0, işaretsiz 2)"),
        "{plan}"
    );
    let _ = fs::remove_dir_all(&kok);
}
