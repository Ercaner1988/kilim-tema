//! Taşmasız, kaymasız yerleşim (egui 0.36).
//!
//! Üç kök neden, üç kural (ölçüm: pasli-beyin `.harness/scratch/tasma*.txt`, 2026-10-11):
//! 1. Yatay satırda (`ui.horizontal`) ve `Grid` hücresinde metin varsayılan olarak SARILMAZ
//!    (`Ui::wrap_mode` → `Extend`): uzun satır sağdan taşar. → [`sarma_kur`].
//! 2. `Panel::left/right` genişliğini HER KARE içeriğin kapladığı alandan kaydeder; `ScrollArea::vertical`
//!    yatayda içeriğe göre büyür. Taşan satır paneli şişirir (tuval daralır, metin yeniden sarılır, kaydırma
//!    sıçrar), dolduran öğe yoksa panel daralır; panel sınıra dayanınca taşan kısım kırpılıp görünmez olur.
//!    Genişliğe tavan koymak yalnız kırpmayı öne çeker. → panel içeriği [`sabit_genislik`] içinde.
//! 3. `Grid` sütun genişliğini ÖNCEKİ kareden ölçer: sarılan değer içerik her değişince 2-3 karede oturur
//!    (içerik sıçrar), kısa hücre uzayınca bayat genişlikte sarılı kalır. → uzun değerli form
//!    [`alan_satiri`], kısa hücreli tablo [`tablo`].
//! 4. Taşan TEK öğe bulunduğu Ui'nin `max_rect`'ini büyütür: ondan sonraki her kardeş daha geniş alanda
//!    sarılır ve kenarda kesilir. Sabit genişlikli denetim (metin kutusu 280 px, ComboBox) yan yana
//!    dizilecekse `horizontal_wrapped` + kendi içinde kap açanlar için [`birim`].

use egui::{pos2, vec2, Align, Context, Layout, Rect, TextWrapMode, Ui, UiBuilder};

/// Uygulama açılışında bir kez: her metin (yatay satırda da) kullanılabilir genişlikte sarılır.
/// Taşan öğeyi bulmak için: hata ayıklama yapısını `KILIM_TASMA_IZ=1` ile başlat; ebeveynini genişleten
/// her öğe kırmızı boyanır (egui `show_expand_width`).
pub fn sarma_kur(ctx: &Context) {
    let iz = cfg!(debug_assertions) && std::env::var_os("KILIM_TASMA_IZ").is_some();
    ctx.all_styles_mut(|s| {
        s.wrap_mode = Some(TextWrapMode::Wrap);
        s.debug.show_expand_width |= iz;
    });
}

/// Yan panelin (`Panel::left/right`) içeriğini çizer; panele HEP kullanılabilir genişlik bildirilir, böylece
/// panelin genişliğini içerik değil yalnız kullanıcının sürüklemesi belirler (şişme, daralma, sıçrama yok).
/// Sarılamayan bir öğe yine de taşarsa kırpılır (bulmak için `KILIM_TASMA_IZ=1`, bkz. [`sarma_kur`]).
pub fn sabit_genislik<R>(ui: &mut Ui, icerik: impl FnOnce(&mut Ui) -> R) -> R {
    let alan = ui.available_rect_before_wrap();
    let mut cocuk = ui.new_child(
        UiBuilder::new()
            .id_salt("sabit_genislik")
            .max_rect(alan)
            .layout(*ui.layout()),
    );
    cocuk.set_clip_rect(alan.intersect(ui.clip_rect()));
    let r = icerik(&mut cocuk);
    let alt = cocuk.min_rect().bottom().max(alan.top());
    ui.advance_cursor_after_rect(Rect::from_min_max(alan.min, pos2(alan.right(), alt)));
    r
}

/// Anahtar–değer satırı: anahtar sabit `anahtar_genisligi`nde, sağa yaslı (uzunsa "…" ile kesilir, tamamı
/// üstüne gelince görünür); değer kalan genişlikte sarılır. Uzun değerli formlarda `Grid` yerine bu: ilk
/// karede doğru oturur, içerik değişince sıçramaz.
pub fn alan_satiri<R>(
    ui: &mut Ui,
    anahtar_genisligi: f32,
    anahtar: impl Into<egui::WidgetText>,
    deger: impl FnOnce(&mut Ui) -> R,
) -> R {
    ui.horizontal_top(|ui| {
        let duzen = Layout::right_to_left(Align::Min);
        ui.allocate_ui_with_layout(vec2(anahtar_genisligi, 0.0), duzen, |ui| {
            ui.set_min_width(anahtar_genisligi);
            ui.add(egui::Label::new(anahtar).truncate());
        });
        ui.vertical(deger).inner
    })
    .inner
}

/// Sabit genişlikli birim: sarmalı satırda (`horizontal_wrapped`) TEK PARÇA yerleşir, sığmazsa alt satıra
/// iner. Kendi içinde kap açan denetimler (`ComboBox`, etiketli denetim, iç içe `horizontal`) satırda kalan
/// genişliği "sığar" sanıp taşar ya da metnini harf harf kırar; bunlar bununla sarılır.
pub fn birim<R>(ui: &mut Ui, genislik: f32, icerik: impl FnOnce(&mut Ui) -> R) -> R {
    let boyut = vec2(
        genislik.min(ui.max_rect().width()),
        ui.spacing().interact_size.y,
    );
    ui.allocate_ui(boyut, |ui| {
        ui.set_max_width(boyut.x);
        icerik(ui)
    })
    .inner
}

/// Kısa hücreli tablo: hücreler SARILMAZ (sütun genişliği önceki kareden gelir; sarılan hücre o bayat
/// genişlikte kalırdı). Tablo panelden genişse yatay kaydırılır, kırpılıp görünmez olmaz. İçinde sonsuz
/// genişlik isteyen öğe (`desired_width(f32::INFINITY)`, `available_width()`) kullanılmaz: kaydırma alanında
/// genişlik sınırsızdır.
pub fn tablo<R>(ui: &mut Ui, grid: egui::Grid, icerik: impl FnOnce(&mut Ui) -> R) -> R {
    egui::ScrollArea::horizontal()
        .id_salt(ui.next_auto_id())
        .show(ui, |ui| {
            grid.show(ui, |ui| {
                ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                icerik(ui)
            })
            .inner
        })
        .inner
}
