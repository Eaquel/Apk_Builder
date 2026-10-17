# Apk_Builder — Koordinasyon

Bu dosya iki oturumun (Claude-A ve Claude-B) ortak hafızasıdır. Kalıcı kararlar,
iş bölümü ve güncel durum burada tutulur. Bir iş bitince ya da yeni bir işe
başlanınca ilgili bölüm güncellenir.

## 1. Proje hedefi

Apk_Builder, Android cihazın üzerinde çalışan, dışarıdan hiçbir araç
(javac, kotlinc, d8, aapt2, apksigner, clang, ld) çağırmadan kaynak koddan
imzalı APK/AAB üreten bir derleme ortamıdır. Uygulamanın kendisi
Kotlin/Gradle ile yazılır; tüm derleme, paketleme, imzalama ve dil
derleyicileri Rust'ta, tek bir crate (`apk_builder_core`) içinde yazılır ve
uygulamaya JNI köprüsüyle (C++ `Builder.cpp`) bağlanır.

Uzun vadeli hedef: Java, Kotlin, C/C++ ve Rust kaynaklarını cihazda derlemek.
JVM tarafı için Dex üretimi, yerel kod tarafı için kendi ara gösterimimiz
(Omni IR), kendi makine kodu üreticimiz (x86, x86_64, ARMv7, ARMv8),
kendi assembler/disassembler'ımız ve kendi bağlayıcımız (ELF/COFF) kullanılır.
Bu bileşenlerin doğruluğu, LLVM/LLD/GCC'nin kendi test paketleriyle ölçülür.

## 2. Mimari ve klasör haritası

| Yol | Ne işe yarar |
| --- | --- |
| `Builder.rs` | Crate kökü (`apk_builder_core`). Derleme grafiği, zamanlayıcı, önbellek, VFS, Gradle/Maven okuma, kaynak (res/arsc/AXML) derleme, Dex yazma, R8 benzeri küçültme (`shrink`), desugar, APK/AAB paketleme, imzalama (v1–v4, RSA, ML-DSA, X.509), inflate/deflate, görüntü işleme, JNI `ffi` modülü ve `compilers` modül kaydı. |
| `Diagnostic_Speech.rs` | Tüm tanı cümlelerinin ve etiketlerinin 14 dile çevirisi (`*_LINES`, `*_WORDS`, İngilizce anahtara göre sıralı). |
| `Terminal.rs`, `Terminal_Speech.rs` | Uygulama içi terminal ve çevirileri. |
| `Vibe_Coding.rs` | Yapay zekâ destekli kodlama yardımcısı (katmanlar: keyless/free/byok). |
| `Android.rs` | Android platform API'sinin derleyiciler için gömülü kaynak tanımları (`THE_PLATFORM`). |
| `Compilers/Java/` | Java derleyicisi (`Java.rs`), tanıları ve otomatik düzelticisi. |
| `Compilers/Kotlin/` | Kotlin derleyicisi (`Kotlin.rs`), tanıları ve otomatik düzelticisi. |
| `Compilers/Cpp.rs`, `Compilers/Rust.rs` | C/C++ ve Rust ön yüzleri (şu an iskelet). |
| `Compilers/Compiler_Infrastructure/Compiler_Infrastructure.rs` | Omni IR: metin/ikili biçim, SSA, doğrulayıcı, yorumlayıcı. |
| `.../Compiler_Infrastructure_Diagnostic.rs` | `EIxxx` tanı kataloğu (sıralı). |
| `.../TableGen.rs` | LLVM TableGen dilinin kendi uygulaması; `.td` → ikili komut tabloları (`Tables/*.bin`, biçim sürümü 2, `Tables.manifest`). |
| `.../Tables/` | `X86.bin`, `X86_64.bin`, `Armv7.bin`, `Armv8.bin` (TableGen çıktısı, `include_bytes!` ile gömülü). |
| `.../Elf.rs`, `.../Pe.rs` | ELF ve COFF/PE okuma-yazma. |
| `.../Linker.rs` | `ld.lld` ve `lld-link` uyumlu bağlayıcı (ELF: x86/ARM/AArch64, COFF/PE, PDB). |
| `.../X86_64.rs`, `.../X86.rs` | x86 hedef tanımı, kodlayıcı (LLVM X86MCCodeEmitter ile birebir), AT&T/Intel yazıcı, AsmMatcher eşleyici, assembler (lexer, ifadeler, direktifler, makrolar, metin çıktısı). `X86.rs` 16/32 bit kipini ve `X86.bin`'i bağlar. |
| `Builder/` | Android uygulaması: `Source/Main/Kotlin/.../Builder.kt` (arayüz), `Source/Main/Native/` (JNI köprüsü, CMake), `build.gradle.kts`. |
| `Gradle/`, `build.gradle.kts`, `settings.gradle.kts`, `gradlew` | Gradle sarmalayıcı ve yapılandırma. |
| `New/Beta`, `New/Test` | Başvuru paketleri (LLVM, Clang, LLD, GCC, binutils, Rust, Kotlin, V8, ICU, TypeScript kaynakları ve test dosyaları). Testler bunları git nesnesi olarak `catalogue()`/`Tree::of` ile okur. |
| `Test/Builder_Test.rs` | Bütün Rust testleri (`mod tests`, her alanın kendi `mod`'u). Lit `RUN:` çalıştırıcısı ve FileCheck eşdeğeri burada. |
| `Test/Builder_Test.py` | Tüm kapı denetimleri (layout, comments, format, lint, strings, speech, independence, suites, linker, assembler ...). |
| `Test/Builder_Test_Baseline.json` | Ratchet paketlerinin taban çizgisi (`--record` ile yazılır, geçen sayı asla düşmez). |
| `Tests/` | Denetimlerin çıktı klasörü, commit'lenmez. |

## 3. Tamamlananlar (yaklaşık %24)

- Çekirdek: derleme grafiği, önbellek, VFS, Gradle/Maven, kaynak derleme,
  Dex yazma, APK/AAB paketleme, imzalama, `release`/`core`/`bridge`/`installable`
  denetimleri.
- Kotlin ve Java derleyicileri: `kotlin-omni` paketi 3790/7711 (3921 kapsam dışı, 0 hata).
- Omni IR altyapısı, `bundles` paketi 605473/605473, `filecheck` 87/90.
- TableGen: `tablegen-lit` 190/456, `tablegen-oracle` 152/459 (kapsam dışılar listeli, 0 hata); dört komut tablosu.
- Bağlayıcı: `lld-elf` 1375/2056, `lld-coff` 341/565 (0 hata).
- x86 (Claude-A, sürüyor): kodlayıcı ve AT&T/Intel yazıcı tamam; AT&T ve Intel assembler
  tamam; karıştırma/FMA/PTERNLOG/FPCLASS yorumları tamam; ELF nesne dosyası çıktısı (`X86_64.rs` içinde
  `object` modülü) tamam; `mc-x86` 598/642, kapsam içi %100 (44 kapsam dışı: COFF/Mach-O/LFI, birkaç
  seçenek ve host LLVM 18 okuyucusunun tanımadığı 5 test, gerekçeleri listeli). Disassembler (`X86_64.rs` içinde `decoder` modülü: LLVM
  X86DisassemblerTables/RecognizableInstr tabloları tablolardan üretilir, X86DisassemblerDecoder portu,
  `llvm-mc --disassemble/--hex/--mdis`) tamam; `mc-x86-disassembler` 188/188, kapsam dışı yok;
  kodla→çöz→aynı komut gidiş-dönüş testi var.

## 4. Kalan işler (öncelik sırasıyla)

1. Assembler'dan ELF/COFF nesne dosyası çıktısı (parçalar, gevşetme, düzeltmeler,
   relocation, `.eh_frame`), bizim bağlayıcıyla bağlayıp bu makinede çalıştırma testi.
2. ARMv7/ARMv8 kodlayıcı + assembler + disassembler (`Armv7.rs`, `Armv8.rs`, tablolar hazır).
3. Omni IR → makine kodu arka ucu (komut seçimi, yazmaç ayırma, çağrı kuralları) x86_64 ve ARMv8 için.
4. C/C++ ön yüzü (`Cpp.rs`) ve Rust ön yüzü (`Rust.rs`).
5. Kotlin/Java derleyicilerinin kapsamını büyütmek (`kotlin-omni` kapsam dışılarını azaltmak).
6. Uygulama (`Builder.kt`) ve yerel köprü: derleyicilerin arayüze bağlanması, cihazda uçtan uca APK.

## 5. Önerilen iş bölümü

Amaç: iki oturum aynı dosyaya dokunmasın.

**Claude-A** (altyapı, makine kodu tarafı)
- `Compilers/Compiler_Infrastructure/` altındaki `TableGen.rs`, `Tables/`, `Tables.manifest`,
  `Elf.rs`, `Pe.rs`, `Linker.rs`, `X86.rs`, `X86_64.rs`, ileride `Armv7.rs`, `Armv8.rs`.
- `Compiler_Infrastructure.rs` (Omni IR) ve makine kodu arka ucu.
- `Compiler_Infrastructure_Diagnostic.rs` içinde `EI100–EI399` aralığı.
- Testlerde: `mod table_gen`, `mod linker_lit`, `mod x86_assembly` ve ileride ARM modülleri;
  Python'da `suites`, `differential`, `linker`, `assembler` denetimleri.

**Claude-B** (dil ön yüzleri, Android tarafı)
- `Compilers/Java/`, `Compilers/Kotlin/`, `Compilers/Cpp.rs`, `Compilers/Rust.rs`.
- `Builder/` (Kotlin uygulaması, JNI köprüsü), `Android.rs`, `Terminal.rs`, `Terminal_Speech.rs`, `Vibe_Coding.rs`.
- `Builder.rs` içindeki çekirdek modüller (paketleme, imzalama, Dex, kaynaklar, Gradle).
- `Compiler_Infrastructure_Diagnostic.rs` içinde `EI400–EI499` aralığı (yeni ön yüz kodları için).
- Testlerde kendi `mod`'ları; Python'da `core`, `release`, `bridge`, `installable`, `kotlin-omni`, `reproducible`.

**Ortak dosyalar** (ikisi de değiştirebilir, küçük ve yalnızca ekleme yapan değişikliklerle):
- `Builder.rs` içindeki `pub mod compilers` kayıt bloğu.
- `Diagnostic_Speech.rs` (yalnızca kendi cümlelerini ekle, tablolar sıralı kalsın).
- `Test/Builder_Test.rs` (yalnızca dosyanın sonuna kendi `mod`'unu ekle; başkasının modülünde
  bir yardımcıyı `pub(super)` yapmak gerekirse PR yorumunda belirt).
- `Test/Builder_Test.py`, `Test/Builder_Test_Baseline.json` (yalnızca kendi paket/denetimlerin).
- Omni IR'de (`Compiler_Infrastructure.rs`) B'nin ihtiyacı olan değişiklikler A'ya PR yorumuyla sorulur.

## 6. Derleme ve test

Ortam: Rust `1.97.1` (`rustup default 1.97.1`, `clippy` ve `rustfmt` bileşenleriyle), JDK 25
(`/usr/lib/jvm/java-25-openjdk-amd64/bin`), Python 3. Üçüncü taraf bağımlılık yok.

```sh
cargo build --lib                                   # hızlı derleme (dev profili optimize)
cargo test --lib -- --exact tests::<mod>::<test>    # tek test
python3 Test/Builder_Test.py layout comments templates dependencies strings speech independence format lint
PATH=/usr/lib/jvm/java-25-openjdk-amd64/bin:$PATH cargo test --release   # tüm testler, ~40 dk
cargo clippy --locked --all-targets -- -D warnings
cargo fmt
python3 Test/Builder_Test.py assembler            # mc-x86 ve mc-x86-disassembler ratchet paketleri
python3 Test/Builder_Test.py linker               # lld-elf, lld-coff
python3 Test/Builder_Test.py <denetim> --record   # taban çizgisini yükselt
cargo test --lib -- --ignored --exact tests::table_gen::write_tables_into_the_tree   # tabloları yeniden üret
```

Yararlı ortam değişkenleri: `APK_BUILDER_SUITE_THREADS`, `APK_BUILDER_MC_FILTER`,
`APK_BUILDER_LLD_FILTER`, `APK_BUILDER_SUITE_TRACE`. Paket ayrıntıları `Tests/suites/<paket>.txt`
ve `Tests/suites/<paket>-out-of-scope.txt` dosyalarına yazılır.

Not: `pkill -f "cargo test"` kullanma (kendi kabuğunu öldürür); `pgrep -x rustc | xargs -r kill` kullan.

## 7. Kod kuralları

1. Kaynak dosyalarda yorum ve doc-comment yok (`comments` kapısı).
2. İsimler tam kelime ve profesyonel (`Compiler_Infrastructure.rs` üslubu); kısaltma yok.
3. Testler yalnızca `Test/Builder_Test.rs` içinde, kendi `mod`'unda; Python denetimleri yalnızca `Test/Builder_Test.py` içinde. Kaynak dosyada `#[test]` yok.
4. Tanılar 14 dilde: `fail("EIxxx", "Sabit İngilizce cümle.")`; değişen veri `.with_context(format!("Etiket: {değer}"))` ile. Her yeni kod kataloğa sıralı girer; her cümle ve etiket `Diagnostic_Speech.rs` içindeki 14 dilin `*_LINES`/`*_WORDS` tablolarına sıralı eklenir.
5. Üçüncü taraf bağımlılık yok; `Cargo.toml`/`Cargo.lock` değişmez; yalnızca `std`.
6. `Compilers/Compiler_Infrastructure/` yalnızca `std`, `crate::diag`, `crate::FailureClass` ve altyapı modüllerini kullanır; `HashMap`/`HashSet`/`RandomState`/`std::env` yok. Aynı girdi her zaman aynı baytları üretir.
7. Dış araçlar (llvm-mc, llvm-tblgen, ld.lld, llvm-readobj, llvm-objdump, gcc, clang ...) yalnızca testlerde kâhin olarak kullanılır; ürün kodu hiçbir dış aracı çağırmaz.
8. Başvuru testlerinin kapsam içi kısmı %100 hedeflenir; kapsam dışı her test adıyla ve gerekçesiyle listelenir. Ratchet sayıları asla düşmez. GPL kod (GCC, binutils) çevrilmez, yalnızca davranış referansıdır.
9. Her commit'ten önce kapılar yeşil: Python kapıları, `cargo test --release`, `rustfmt`, `clippy -D warnings`.
10. Commit mesajları Türkçe ve açıklayıcı; başlık `[Claude-A]` ya da `[Claude-B]` ile başlar; mesaj sonunda `Co-Authored-By` ve `Claude-Session` satırları. Commit mesajında model kimliği yazılmaz.

## 8. Haberleşme ve birleştirme

- Dallar: `Apk_Builder_Beta` (yalnızca PR ile), `Claude-A-Test` (A), `Claude-B-Test` (B).
- Kanallar: A'nın PR'ı `Claude-A-Test → Apk_Builder_Beta`, B'nin PR'ı `Claude-B-Test → Apk_Builder_Beta`.
  Karşı tarafa yazmak için onun PR'ına yorum yazılır. Her yorum, issue ve commit başlığı `[Claude-A]`/`[Claude-B]` ile başlar.
- Görevler GitHub Issues'ta; başlık `[A]` ya da `[B]` ile başlar; üstlenmeden önce etiketlenir.
- Birleştirme: kendi dalında kapılar yeşil → PR yorumuyla duyur → `Apk_Builder_Beta`'yı kendi dalına merge et,
  çakışmaları çöz, testleri yeniden çalıştır → PR'ı birleştir. Karşı taraf sonra Beta'yı kendi dalına merge eder.
  Çakışma karşı tarafın modülündeyse kendin çözme, sor.
- Yasak: force-push, rebase, geçmişi yeniden yazmak, Beta'ya ya da karşı dala doğrudan push, test silmek/atlamak,
  karşı tarafın modülünü sormadan değiştirmek.

## 9. Güncel durum

- Her hesapta tek oturum çalışır: Claude-A ve Claude-B. Eski `claude/...` dalları silinebilir.
- Claude-A: x86 assembler, disassembler ve ELF nesne dosyası çıktısı tamam: `mc-x86` 598/642 (kapsam içi
  %100, 44 kapsam dışı), `mc-x86-disassembler` 188/188; assembler çıktısı bizim bağlayıcıyla bağlanıp bu
  makinede çalışıyor. Sıradaki adımlar: Omni IR → x86_64 arka ucu, sonra ARMv8.
- Claude-B: C ön yüzü (`Compilers/Cpp.rs` içinde C kısmı): önişlemci, sözcük çözümleyici, ayrıştırıcı,
  anlam çözümlemesi ve Omni IR'ye indirgeme. Ölçü: Clang'ın `test/Preprocessor`, `test/Lexer`, `test/Parser`,
  `test/Sema` lit testlerinden C olanları, yeni ratchet paketleri (`clang-c-*`), kapsam dışılar gerekçeli listeli.
  Yeni tanı kodları `EI400–EI499`, testler `mod c_front_end`.
