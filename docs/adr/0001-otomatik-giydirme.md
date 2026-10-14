# ADR 0001: Otomatik giydirme (roller, saptama, kendiliğinden yerleşim)

Durum: kabul, ilk dilim uygulandı (2026-10-13). Karar sahibi: Ercan. Ölçümler bu makinede (Windows 11, release) alındı;
yeniden üretme komutları §9'da.

## 1. Bağlam

İstek (Ercan, aynen): temayı elimizdeki her araca uyacak biçimde esnek işleyecek türde yeniden tasarlamak; önce otomatik
tema giydirme: nereye arka plan, nereye fon, nereye çerçeve, nereye düğme geleceğini saptamak, saptamalara göre
kendiliğinden yerleşim planı çıkarmak; gerekirse araç arayüzlerine tanımlayıcı/işaretçi konmasını talep etmek.

Keşif (durağan tarayıcı `kilim-tarama` ile, `cfg(test)`/`tests`/`examples` hariç; 370 dosya). Ayrıntı ve tüm dosya:satır
listeleri `docs/olcum/plan-*.txt`:

| Araç | Panel (sol/sağ/üst/alt) | Merkez | Window | Frame | Grid | ScrollArea | ComboBox | TextEdit | düğme | seçim etiketi (sekme/liste) | horizontal (+wrapped) | tema kurulumu |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| pasli-beyin | 1/2/4/1 | 1 | 9 | 3 | 6 | 17 | 8 | 25 | 96 | 20 | 37+14 | yalnız `Kilim::uygula` (app.rs:214) + `sarma_kur` (main.rs:78); `cerceve` yok |
| kesfuzzunun | 1/1/2/1 | 1 | 5 | 0 | 0 | 7 | 5 | 9 | 59 | 16 | 27+2 | yalnız `Kilim::uygula` (gui/lib.rs:1058); `sarma_kur` YOK |
| nazar | 0/0/1/1 | 1 | 0 | 2 | 1 | 3 | 0 | 3 | 6 | 4 | 8+1 | tam: `iki_temayi_kur`+`uygula`+`tema_secici`; `cerceve` kilim-tema'dan KOPYALANMIŞ (arayuz.rs:36) |
| el-Fihrist | 1/0/1/1 | 1 | 0 | 0 | 0 | 2 | 0 | 1 | 2 | 1 | 3+0 | HİÇ: `fihrist-gui` kilim-tema'ya bağlı bile değil (düz egui) |
| kunye (3 kitaplık) | 0/0/0/1 | 0 | 1 | 5 | 1 | 6 | 5 | 6 | 20 | 6 | 16+8 | yok; renkler `ui.visuals()`'tan (tema konakta) |

Örnek dosya:satır: pasli `app.rs:158` (üst "gorunum"), `:171` (alt "durum_cubugu", kendi `.frame`'i var), `:275` (üst "ust_serit"),
`:363` (sol "cekmece", `sabit_genislik` elle), `:464` (sağ "detay_panel"), `:512` (merkez); kesf `gui/lib.rs:1141` (menü, `exact_size(24)`),
`:1233` (araç çubuğu), `:1320` (durum), `:1349`/`:1388` (yan paneller), `:1439` (merkez); nazar `arayuz.rs:159/163/167`; el-Fihrist
`app.rs:102/122/132/159`; kunye `sohbet-egui/lib.rs:47` (alt panel, `.frame` verilmiş). Elle renk: pasli 51 `Color32` sabiti (25'i metin/dolgu olarak sınıflandı: 20 Hata [çoğu `Color32::RED` `colored_label`, `KIRMIZI` düğme dolgusu dahil], 4 Uyarı turuncusu, 1 Başarı yeşili; kalanı treemap veri paleti), kesf 25 (5'i metin/dolgu).
`kunye-egui`/`sohbet-egui`/`md-egui` hiç renk sabiti kullanmaz: `ui.visuals().error_fg_color/warn_fg_color/weak_text_color` okurlar.

Üç bulgu tasarımı belirledi:
1. Araçların bölgeleri hep aynı beş egui kabıdır (Panel ×4 yön, CentralPanel, Window, Frame, ScrollArea); dağılım küçük ve kalıplı.
2. İşaretsiz giydirmenin tek kaldıracı `ui.visuals()`'tır (kunye kitaplıkları bunu zaten böyle kullanıyor); `Kilim::gorsel` yalnız 8 alanı kuruyordu.
3. Pasli/kesf/nazar'da elle sabit renk, elle `.frame`, elle `sabit_genislik`; el-Fihrist hiç giydirilmemiş.

Yan bulgu (HEAD b62fcac): `sarma_kur` yayın (release) derlemesinde DERLENMİYORDU (`Style::debug` yalnız `debug_assertions`'ta var;
`cfg!(debug_assertions)` alanı gizlemez). Pasli'nin `cargo build --release`'i kilim-tema yüzünden kırık olurdu. Bu dilimde düzeltildi
(`#[cfg(debug_assertions)]`), `cargo build --release` ile doğrulandı.

## 2. Karar

Karma tasarım, dört katman. Her katman bir öncekinin üstüne biner ve hiçbiri zorunlu değildir.

| Katman | Ne yapar | Araçtan istenen |
|---|---|---|
| 0. Palet (işaretsiz) | `Kilim::gorsel` anlamsal renkleri de kurar: `error_fg_color`, `warn_fg_color` (AA okunur), metin kutusu zemini, şeritli satır dolgusu | hiçbir şey |
| 1. Rol işaretçisi | `.rol(Rol::YanPanel)` çerçeveyi, yazı/sarma kuralını, `sabit_genislik`'i rolden getirir | bölge başına bir `.rol(..)` |
| 2. Saptama | `kilim-tarama`: işaretsiz bölgeyi bulur, rol önerir, işaretçi talep listesini (önce/sonra) üretir | yok (araç sahibi talebi uygular) |
| 3. Doğrulama/plan | Çalışma zamanı defteri, `plan_metni`, `KILIM_ROL_IZ=1` bindirmesi, uyumsuz rol denetimi | yok |

Açılışta tek çağrı: `kilim_tema::giydir(&ctx, &tercih)` = `iki_temayi_kur` + `sarma_kur` + `tercih.uygula` + (hata ayıklamada
`KILIM_ROL_IZ` varsa) bindirme eklentisi.

## 3. Roller ve çizim kuralları

`Rol` (20 rol, `Rol::HEPSI`): Zemin, Yuzey, YanPanel, UstCubuk, DurumCubugu, Kart, Pencere, Acilir, Giris, ListeSatiri, Tablo, Sekme,
Rozet, DugmeBirincil, DugmeIkincil, DugmeTehlikeli, Vurgu, Hata, Uyari, Basari. Adlar `Rol::ad()` ile işaretçide, tarayıcıda ve planda aynıdır
(sınama: `tarama/tests/roller.rs`).

Kural tablosu TEK kaynaktan türer: `Rol::stili_hesapla(&Kilim, koyu, Varyant, kilim_gorunumu)`. Hiçbir rol rengi sabit yazılı değildir; hepsi
`acik()`/`koyu()` paletinden (dolayısıyla etkin tema paketinden) ve `Varyant::vurgu`dan gelir. Kilim görünümünde bölge dolguları yarı saydamdır
(kâğıt lifi görünür), Sade'de opaktır; `etkin_gorunum(ctx)` `TemaTercihi::uygula` ile saklanır, hiç kurulmadıysa Sade.

| Rol | dolgu (palet) | çizgi | köşe | iç boşluk | yazı / sarma | otomatik yerleşim |
|---|---|---|---|---|---|---|
| Zemin | Kilim: saydam; Sade: `zem` | yok | 0 | 8 | tema | merkez; `no_frame` için `.frame(Frame::NONE)` |
| Yuzey | `yuzey` (α .85) | `cizgi` 1 | 4 | 8x6 | tema | |
| YanPanel | `yuzey` (α .78) | ayırıcı çizgi | 0 | 8x6 | Wrap | `sabit_genislik` AÇIK |
| UstCubuk | `yuzey` (α .85) | ayırıcı çizgi | 0 | 8x2 (egui varsayılanı: sabit yükseklikli çubuğu kısaltmaz) | Wrap | |
| DurumCubugu | `zem` (α .9) | ayırıcı çizgi | 0 | 8x2 | `soluk`→AA ayarlı, `Small`, **Truncate** (tek satır) | |
| Kart | `yuzey` (α .9) | `cizgi` 1 | 6 | 8 | tema | |
| Pencere | `yuzey` | `cizgi_koyu` 1 | 8 | 8 | tema, gölge | |
| Acilir | `yuzey` | `cizgi_koyu` 1 | 6 | 6 | tema, gölge | |
| Giris | `giris_zemini` (açık: `yuzey`; koyu: `zem`'den %30 koyu) | `cizgi_koyu` 1 | 4 | 6x4 | tema | |
| ListeSatiri / Sekme | `vurgu` α .35 (seçili) | yok | 3 / 4 | 6x2 / 10x4 | okunur metin | |
| Tablo | `cizgi` α .35 (şerit) | yok | 0 | | okunur metin | |
| Rozet | `cizgi` | yok | pill | 8x1 | `Small` | |
| DugmeBirincil | `Varyant::vurgu` | yok | 4 | 10x4 | `okunur_metin(dolgu)` | |
| DugmeIkincil | `yuzey` | `cizgi_koyu` 1 | 4 | 10x4 | `murekkep` | |
| DugmeTehlikeli | `kirmizi` | yok | 4 | 10x4 | `okunur_metin(dolgu)` | |
| Vurgu / Hata / Uyari / Basari | saydam | | | | `vurgu`/`kirmizi`/`sari`/`yesil` mürekkebe doğru kaydırılıp `zem` ve `yuzey` üstünde ≥ 4.5 | |

Erişilebilirlik kuralı: her rolün yazısı kendi dolgusunun (yüzey ve zem üstündeki görünür renginin) üstünde WCAG AA (4.5:1) olmalı. Açık/koyu ×
iki varyant × Kilim/Sade × 20 rol taranır (`her_rol_her_temada_okunur`). `rol_rengi(ui, rol)` yazı rolünü sabit yüzeye göre değil, `ui.stack().bg_color()`
(üst çerçevelerin harmanlanmış gerçek dolgusu) üstünde ayarlar.

Rol uygulama yolları (ilk dilim): bölge rolleri (Zemin, YanPanel, UstCubuk, DurumCubugu, Kart, Pencere, Acilir) `.rol(..)` ile; düğme rolleri `dugme(ui, rol, metin)`;
yazı rolleri `rol_rengi`. Giris/ListeSatiri/Tablo/Sekme/Rozet/Yuzey kuralları tanımlı ve AA sınanmış, `Rol::stili(ui)` ile okunur; kendilerine ait uygulayıcı API'leri
ikinci dilimdedir (§10). Katman 0 (`Visuals`) bunların çoğunu işaretsiz de sağlar.

## 4. Saptama: üç yol, ölçüm ve karar

### (a) Çalışma zamanı içgözlem

Ölçülen/doğrulanan (egui 0.36.2 kaynağı + deney):
- Erişilebilir: `ctx.viewport(|v| v.prev_pass.widgets)` her widget'ın `id`, `parent_id`, `layer_id`, `rect`, `sense`'ini verir; her `Ui` de bir kayıttır
  (kap ağacı kurulabilir). `UiStack` (`kind()`, `tags()`, `bg_color()`) ve `UiKind` (LeftPanel, TopPanel, CentralPanel, Window, Frame ...) YALNIZ o `Ui`'nin
  içinden okunur, dışarıdan değil.
- Erişilemeyen: kap türü dışarıdan yok (panel `Ui`'leri hep `GenericContainer`, `WidgetType::Panel` hiçbir panel tarafından kurulmuyor); `WidgetInfo` (düğme/metin
  kutusu türü) yalnız hata ayıklama yapısında ya da etkileşilen widget için dolu; panel çerçevesi üst `Ui`'nin stilinden, içerik kapanışı çalışmadan ÖNCE çözülür ve
  beş panel türü aynı `panel_fill`i alır (kanıt: `isaretsiz_bes_panel_ayni_cerceveyi_alir_stil_bunlari_ayiramaz`). Yani saptansa bile işaretsiz bölgenin çerçevesi
  sonradan değiştirilemez: ancak bir sonraki karede ve hangi kapsayıcıya ait olduğunu geometriyle tahmin ederek (kesinlik yok).
- Maliyet (release, 2 koşu): widget listesini dolaşmak 0.28 µs (218 widget) .. 16–20 µs (4018 widget), karenin %0.2'sinden azı. AccessKit ağacını zorla açmak
  kareyi **+%73..+%97** yavaşlatır (218 widget: 208→409 µs; 4018 widget: 4.3→8.2 ms). Üretimde AccessKit yalnız ekran okuyucu varken açılır; yani kap türü/etiket için
  çalışma zamanında AccessKit'e güvenilemez, güvenilirse her kare pahalıdır.
- Karar: kimlik koyucu değil, DOĞRULAYICI ve GÖSTERİCİ olarak kullanılır (§6). İşaretsiz bölgeyi çalışma zamanında saptayıp giydirme: reddedildi.

### (b) Durağan kaynak taraması

`kilim-tarama` (bağımlılıksız, sözcüksel: yorum/dize/ömür/ham dize bilen sözcük ayırıcı + parantez eşleme + zincir `.show(..)` + kapanış gövdesi).

| Ölçüt | sözcüksel (seçilen) | `syn` 2 (deney, depo dışı) | `rg` regex |
|---|---|---|---|
| 5 aracın 370 dosyası, 3.8 MB | 0.24–0.27 s (G/Ç + rapor dahil) | 0.655 s yalnız ayrıştırma+gezinti (2.6x) | |
| Ek derleme/bağımlılık | 0 | +16.6 s, +3 sandık | 0 |
| Panel/Central/Window/Grid/ComboBox/horizontal* sayıları | hepsi `syn` ile AYNI (5 araç) | referans | |
| Panel+Central+Window dosya:satır kümesi (38 bölge) | `rg` ile birebir; `rg` fazladan: `#[cfg(test)]` içindeki 1 (pasli `panel.rs:626`) + `examples/` içindeki 3 (kunye) | | `cfg(test)`/örnek ayrımı yok |
| Zincir/kapanış gövdesi, iç içelik | var (öğe→bölge atama) | var | yok |
| Tür bilgisi (alıcı hangi tip) | YOK | yok (macro/çözümleme olmadan) | yok |

Sınırlar (ölçülmüş): `self.x.ciz(ui)` türü alıcı çözümlenmez; 156 dış çağrıdan (pasli) tanımı tek bulunanların gövdesi plana "dolaylı" eklenir, çok tanımlıysa "tanımı yok/birden çok" der
(kesf'te panellerin 3'ü böyle). Rol önerisi sezgiseldir (yön, içerik, `exact_size`); yanlış öneri mümkündür (örn. kesf'in "Z" logosu Zotero kırmızısı, Hata sanılır; kunye'nin alt paneli
sohbet girişidir, içerik başka işlevde olduğundan DurumCubugu sanılır).
Karar: tarayıcı SEÇİLDİ, `syn` ertelendi: aynı doğruluk, 2.6x yavaş, bağımlılık getiriyor; `syn`'e geçiş yolu açık (tür/makro gerekirse).

### (c) Bildirimsel işaretçiler

`egui::Panel::left("x").rol(Rol::YanPanel).show(ui, ..)`: `RolVer` özelliği (`Panel`, `CentralPanel`, `Window`, `Frame`), `Rolu<K>` sarmalayıcı. Tek jeton ekleme; `show`
çağrısından hemen önce. Kesinlik tam: rol çağıranın açık beyanı.
- Maliyet (release): 4 bölgede kare başına +2.2 µs (n=300, 342→344 µs) .. +9.0 µs (n=20, 40.9→49.9 µs), bölge başına ≈ 1–2 µs; `request_repaint` YOK
  (`isaretleme_bosta_yeniden_cizim_istemez`); bindirme eklentisi yüklüyse (env kapalıyken yüklenmez) +8..+10 µs.
- Çerçeve, yazı/sarma ve `sabit_genislik` işaretçi anında rolden gelir: işaretçi ÇERÇEVEYİ DE belirleyebilen tek yoldur.
- Alt `Ui`'ler `bagil_rol(ui)` ile içinde bulundukları rolü sorabilir (`UiStack` etiketi): bağlam duyarlı bileşenler için.

### Karar

Hibrit: **işaretçi (c) kesinlik ve çerçeve gücü verir; tarayıcı (b) işaretsiz bölgeyi bulur ve işaretçi talebini üretir; çalışma zamanı (a) yalnız doğrular ve gösterir.**
Reddedilenler:
- Yalnız (a): çerçeve çözülüp çizildikten sonra değiştirilemez, kap türü/etiket yok, AccessKit +%73..+97.
- Küresel `Style` ile panel türüne göre giydirme: panel türleri stile göre ayırt edilemez (kanıtlı).
- Sarmalayıcı işlev `kilim_tema::yan_panel(ui, Panel::left(..), |ui| ..)`: çağrı yerini yeniden yazar; `.rol(..)` tek jetondur ve `show*` imzalarını korur.
- Türetme/öznitelik makrosu: egui çağrı zincirleri ifade düzeyindedir; makro eklemek derleme maliyeti getirir, kazanç yok.
- Çalışma zamanında geometri tahminiyle işaretsiz bölge giydirme: kesinlik yok, bir kare gecikme, sürüm yapısında tür yok. İkinci dilim adayı olarak yalnız UYARI üretmek (§10).

## 5. Kendiliğinden yerleşim planı

Kararlar roldan türer ve işaretçi anında UYGULANIR (araç hiçbir şey yazmaz):
- `YanPanel`: içerik `sabit_genislik`'te (panel genişliğini kullanıcı belirler; şişme/daralma/sıçrama yok), Wrap. Ölçüm: 900 px'lik öğe + uzun yatay satırla rolsüz panel
  varsayılanın (360) üstüne şişer, rollü panel kareler arası ≤ 1 px oynar (`yan_panel_rolu_genisligi_kendiliginden_sabitler`).
- `DurumCubugu`: Truncate: uzun ileti çubuğu yükseltmez (`durum_cubugu_rolu_uzun_metni_tek_satirda_keser`), `sarma_kur` çağrılmadan bile.
- `UstCubuk`: egui varsayılan dikey iç boşluğu: `exact_size(24)` çubuğun içeriğini kısaltmaz (`ust_cubuk_rolu_sabit_yukseklikli_cubugun_icerigini_kisaltmaz`; ilk sürümdeki 8x4 boşluk içeriği 19→15 px'e
  indiriyordu, sınama yakaladı).
- `.frame(..)`/`.duzenle(..)`/`rol_ezme`: araç çerçevesi ya da rengi rolün yerine geçer (aşağıda).

Rolden çıkarılamayan kararlar (içerik bilgisi gerekir) tarayıcı önerisi olarak sunulur, uygulanmaz:

| Kod | Saptama | Öneri |
|---|---|---|
| ROL | işaretsiz Panel/Central/Window/Frame(popup, group) | `.rol(Rol::..)` (aracın `.frame`'i varsa ondan ÖNCE; `.frame`'den sonraki egui yöntemleri `.rol`'dan önceye taşınır; `no_frame` için `.frame(Frame::NONE)`) |
| YATAY | üst/yan panelin doğrudan gövdesinde sarmasız `horizontal` (`exact_*` yoksa) | `horizontal_wrapped` (pasli K14) |
| COMBO | `horizontal_wrapped` gövdesinde `birim` içinde olmayan `ComboBox` | `kilim_tema::birim(..)` (yerlesim.rs kök neden 4) |
| IZGARA | `Grid` `tablo` ile sarılı değil | kısa hücre: `tablo`; uzun değer: `alan_satiri` (hücre uzunluğu bilinmez; karar araç sahibinin) |
| RENK | `colored_label/.color/.fill/const` içinde anlamı sezilen sabit renk | `ui.visuals().error_fg_color` / `warn_fg_color` / `rol_rengi(ui, Rol::Basari)` |
| KURULUM | `run_native` var, `giydir` yok | `giydir(&cc.egui_ctx, &tercih)` |

Gösterim: (1) çalışma zamanı: `plan_metni(ctx)` (rol, rect, kap, `sabit_genislik`, UYUMSUZ) ve `KILIM_ROL_IZ=1` bindirmesi (her bölge rol rengiyle çerçeve + ad; uyumsuz kırmızı ünlemli);
(2) durağan: `kilim-tarama [--talep] ad=dizin` düz metin (JSON yok) bölge ağacı + öneri sayıları / talep listesi. Örnek planlar §8 ve `tests/plan_ornek.rs` çıktısı.

## 6. Doğrulama ve ceza: uyumsuz rol

İşaretçi `ui.stack()`'ten gerçek kabı okur (`LeftPanel`, `TopPanel`, `Window` ...) ve rolün beklenen kapla uyumunu deftere yazar: `.rol(Rol::YanPanel)` bir üst panele konursa plan "UYUMSUZ"
der, bindirme kırmızı çizer (`uyumsuz_rol_yakalanir`). Defter her karede sıfırlanır (`defter_her_karede_sifirlanir`).

## 7. Araca özgü esneklik

- Varsayılan: hiçbir şey yapmayan araç `giydir` ile palet + sarma alır (katman 0).
- Bölge başına: `.rol(r).duzenle(|s| s.dolgu = ..)`; tam çerçeve: `.rol(r).frame(Frame::new()..)` (rolün yazı/sarma/`sabit_genislik` kuralları sürer); `show_separator_line`.
- Bağlam başına: `rol_ezme(ctx, Rol::DurumCubugu, |s| s.dolgu = ..)`; bölge ezmesi bağlam ezmesinden önce kazanır (`arac_rolu_ezebilir_cerceveyi_degistirebilir`).
- Tema paketi roller için de geçerlidir (`tests/rol_paket.rs`).
- Tema bağımsız kitaplıklar (kunye-egui, sohbet-egui, md-egui) `ui.visuals()` ile çalışmaya devam eder; katman 0 onları işaretsiz giydirir. Bu kitaplıklara `.rol(..)` koymak onları kilim-tema'ya
  bağlar: karar Ercan'ındır (§11); tarayıcı bu dosyalarda "kilim-tema bağımlılığı YOK" uyarısı basar.

## 8. Çıkan planlar (özet)

Tam metin: `docs/olcum/plan-*.txt`, talepler: `docs/olcum/talep-*.txt` (üretim: `kilim-tarama [--talep] ad=dizin`).

| Araç | Rol taşıyabilen bölge | öneri sayıları |
|---|---|---|
| pasli-beyin | 20 (hepsi işaretsiz) | ROL 20, RENK 25, COMBO 4, YATAY 1, KURULUM 1 |
| kesfuzzunun | 11 | ROL 11, RENK 5, KURULUM 1 |
| nazar | 3 | ROL 3, IZGARA 1, KURULUM 1 |
| el-Fihrist | 4 | ROL 4, YATAY 1, KURULUM 1 |
| kunye | 2 (kitaplık) | ROL 2, IZGARA 1 |

İşaretçiler konmuş iskeletlerin çalışma zamanı planı (`cargo test --test plan_ornek -- --nocapture`; kesf):
```
UstCubuk     16,12   1248x18  kap=TopPanel
UstCubuk     16,36   1248x32  kap=TopPanel
DurumCubugu  16,771  1248x19  kap=BottomPanel
YanPanel     16,79    183x683 kap=LeftPanel   sabit_genislik
YanPanel     961,79   303x683 kap=RightPanel  sabit_genislik
Zemin        216,81   728x679 kap=CentralPanel
Pencere      25,63    322x364 kap=Window
```

## 9. Sınamalar ve yeniden üretme

- `cargo test --workspace` (kilim-tema, kilim-palet, kilim-tarama), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt -p kilim-tema -p kilim-palet -p kilim-tarama`.
- `tests/rol.rs` (20 sınama), `tests/plan_ornek.rs` (3), `tests/rol_paket.rs`, `tests/api_uyum.rs` (eski adların derlemeyle denetimi), `tarama/` (sözcük, tarama, kural, dizin, rol adı eşitliği).
- Ölçümler: `cargo test --release --test olcum -- --ignored --nocapture` (işaretçi maliyeti, içgözlem maliyeti).
- Görsel: `WGPU_BACKEND=vulkan KILIM_GORSEL_DIZIN=<dizin> cargo test --test gorsel -- --ignored` dört temada PNG üretir (bu doğrulama metin kutusunun koyu
  temada panelde kaybolduğunu yakaladı; sınama eklendi).
- Bozma kanıtları (her biri düzeltme geri alınınca ilgili sınama KIZARDI, sonra geri konuldu): YanPanel `sabit_genislik` kapalı; DurumCubugu Truncate yok; kayıtta `request_repaint`;
  `rol_rengi` statik yüzeye göre; yazı rengi AA ayarsız; aracın `.frame`'i yok sayılır; defter sıfırlanmaz; uyum denetimi yok; rol etiketi yok; bindirme çizmez; Kilim görünümünde dolgu opak;
  `Visuals` hata rengi yok; UstCubuk dikey boşluk 4; metin kutusu zemini = zem; tarayıcı: iç içelik koparma, `cfg(test)` taranması, dize içi çağrı sayımı, Grid/tablo ayrımı, `exact_*` ayrımı, rol adı sırası,
  dizin yürüyüşünde test dizini/bağımlılık; `api_uyum`: bir ad yeniden dışa aktarımdan çıkarılınca derlenmez.

## 10. Sonuçlar ve ikinci dilim

Sonuçlar: araç başına gereken iş "bir `giydir` + bölge başına bir jeton"a iner; kural değişikliği tek dosyada (`rol.rs`) yapılır; tarayıcı bu işi mekanik listeler.
Riskler: katman 0 `Visuals` değişikliği (hata/uyarı rengi, metin kutusu zemini, şeritli satır) `Kilim::uygula` kullanan pasli/kesf/nazar'ın görünümünü hemen değiştirir.
İkinci dilim adayları: Giris/ListeSatiri/Tablo/Sekme/Rozet için uygulayıcı API (`oge(ui, Rol::Sekme, ..)`); çalışma zamanında geometriyle "işaretsiz bölge" UYARISI (yalnız uyarı);
tarayıcıya `--uygula` (yama üretimi); `syn` yalnız alıcı türü gerekirse; `ScrollArea`/`Grid` için rol; tema_secici'ye rol önizlemesi.

## 11. Açık sorular (Ercan)

Yanıtlananlar (2026-10-13/14): 2 → evet, tüm araçlara yansısın; 3 → evet, kunye kitaplıkları da `.rol` kullanır (konak kurulumu yine `giydir`); kilim bağımlılığı her yerde git rev (pasli dahil, yol bağımlılığı yok). Nazar'ın küçük penceresi için istenen ince kenarlık `cerceve_kalinlikli` ile karşılandı (d4f3504). 2026-10-14: 1 → evet, `kilim-palet` ayrımı kalır; 4 → alan alan birleştirme (aşağıda, uygulanacak); 5 → evet, `kapi.toml` açılır (kunye beyanı örnek; ilk koşunun kırmızıları aynı işte giderilir, muafiyet gerekçeli). Açık soru kalmadı.

1. Sandık ayrımı: kilim-tema 1690 kod satırına çıkacaktı (sınır 1500); renk çekirdeği `kilim-palet`e ayrıldı, kilim-tema API'si aynı (yeniden dışa aktarım). Uygun mu?
2. Katman 0 görünüm değişikliği (hata/uyarı rengi paletten, metin kutusu zemini, şeritli satır dolgusu) pasli/kesf/nazar'a rev yükseltince hemen yansır: kabul mü?
3. kunye kitaplıklarına `.rol` konsun mu (bağımlılık) yoksa yalnız konak uygulamaları mı işaretlesin?
4. `Frame` semantiği: `Frame::new().fill(x).rol(Rol::Kart)` rol çerçevesini kullanır, aracın `fill`'ini yok sayar (egui `Frame` alanları sorgulanamaz); renkli özel çerçeveler için tarayıcı Kart önermez.
   **Karar (2026-10-14): alan alan birleştirme.** "Sorgulanamaz" yanlıştı: egui 0.36 `Frame` alanları `pub`. Aracın açıkça verdiği alan kazanır (`fill` saydam değilse, `stroke` boş değilse, `inner_margin`/`outer_margin`/`corner_radius`/`shadow` varsayılan değilse), verilmeyen alan rol çerçevesinden gelir. `Rolu<Frame>::show` bugün `self.kap`'ı atıyor (sessiz kayıp); düzeltme + sınama ayrı iş, sonra tarayıcı renkli çerçevelere de Kart önerebilir.
5. `kilim-tema` için `kapi.toml` açılsın mı (şu an `kapi . --guvenilen-kod` çalışmaz: beyan yok)?
