# 🖼 IMG OPTIMIZE

Resimleri otomatik olarak optimize edilmiş **WebP** formatına dönüştüren, sıfır harici bağımlılık gerektiren hızlı bir CLI (komut satırı) aracı.

## ✨ Özellikler

- **Desteklenen formatlar:** `.jpg`, `.jpeg`, `.png`, `.webp`, `.bmp`, `.tiff`
- **3 kalite profili:** Web (maks sıkıştırma), Dengeli, Yüksek Kalite
- **Google libwebp motoru:** Gerçek lossy WebP sıkıştırma ile çok küçük dosya boyutları
- **Akıllı alpha algılama:** Şeffaflık yoksa RGB kullanarak dosya boyutunu daha da düşürür
- **Otomatik boyutlandırma:** Profile göre akıllı boyutlandırma (en-boy oranı korunur)
- **Paralel işlem:** Çok çekirdekli işlemcilerde tüm dosyalar eş zamanlı işlenir
- **Sıfır bağımlılık:** Sisteminize ek bir yazılım kurmanıza gerek yok — sadece derleyin ve çalıştırın
- **İnteraktif arayüz:** Renkli terminal çıktısı, ilerleme çubuğu ve detaylı özet raporu

## 🎯 Kalite Profilleri

| Profil | WebP Kalite | Maks Genişlik | Maks Yükseklik | Kullanım |
|--------|-------------|---------------|----------------|----------|
| 🌐 Web | 70% | 1024px | 768px | Web siteleri, blog, e-ticaret |
| ⚖️ Dengeli | 78% | 1280px | 960px | Genel kullanım |
| 🎨 Yüksek Kalite | 88% | 1920px | 1440px | Portföy, baskı hazırlık |

### Tahmini Sıkıştırma Oranları

| Orijinal Boyut | Web Profili | Dengeli | Yüksek Kalite |
|-----------------|-------------|---------|---------------|
| 8 MB (4000x3000) | ~40-70 KB | ~60-100 KB | ~150-250 KB |
| 3 MB (2000x1500) | ~30-50 KB | ~40-70 KB | ~80-150 KB |
| 1 MB (1200x900) | ~20-40 KB | ~30-50 KB | ~50-80 KB |

> 💡 Lossy WebP kalite 70-78 aralığında insan gözü ile görsel bozulma fark edilmez. Google'ın PageSpeed aracı da web görselleri için bu aralığı önerir.

## 🚀 Kurulum

### Gereksinimler

- [Rust](https://www.rust-lang.org/tools/install) (stable)

### Derleme

```bash
# Projeyi klonlayın
git clone https://github.com/kullanici/img-optimize.git
cd img-optimize

# Release modunda derleyin (optimizasyonlu, küçük binary)
cargo build --release
```

Derlenen binary: `target/release/img-optimize`

### PATH'e Ekleme

#### Linux / macOS

```bash
# Binary'yi /usr/local/bin'e kopyalayın
sudo cp target/release/img-optimize /usr/local/bin/

# Artık her yerden çalıştırabilirsiniz
img-optimize
```

Veya ev dizininize eklemek isterseniz:

```bash
mkdir -p ~/.local/bin
cp target/release/img-optimize ~/.local/bin/

# ~/.bashrc veya ~/.zshrc dosyanıza ekleyin:
export PATH="$HOME/.local/bin:$PATH"
```

#### Windows

```powershell
# Binary'yi bir klasöre koyun, örneğin:
mkdir C:\tools
copy target\release\img-optimize.exe C:\tools\

# Sistem PATH'ine ekleyin:
# Ayarlar > Sistem > Gelişmiş sistem ayarları > Ortam Değişkenleri
# Path değişkenine C:\tools ekleyin
```

## 📖 Kullanım

1. Optimize etmek istediğiniz resimlerin bulunduğu klasöre gidin
2. `img-optimize` komutunu çalıştırın
3. Kalite profilini seçin (Web / Dengeli / Yüksek Kalite)
4. Dosya listesini gözden geçirin ve onaylayın
5. İşlem tamamlandığında sonuçları inceleyin

```bash
cd /path/to/images
img-optimize
```

## 📁 Çıktı

- Optimize edilen dosyalar mevcut klasörde `oa_optimize/` alt klasörüne kaydedilir
- Orijinal dosyalara dokunulmaz
- Çıktı dosyaları `.webp` uzantılı olur (örn: `foto.jpg` → `oa_optimize/foto.webp`)

## 📐 Boyutlandırma Kuralları

Seçilen profile göre farklı boyut limitleri uygulanır:

| Görsel Tipi | Kural |
|-------------|-------|
| Yatay (genişlik > yükseklik) | Genişlik sınırlanır |
| Dikey (yükseklik ≥ genişlik) | Yükseklik sınırlanır |
| Kare | Yükseklik sınırlanır |

- En-boy oranı (aspect ratio) **her zaman korunur**
- Zaten belirtilen boyuttan küçük olan görsellere boyutlandırma uygulanmaz
- Yeniden boyutlandırma algoritması: **Lanczos3** (en yüksek kalite)

## 🔧 Teknik Detaylar

- **WebP Encoder:** Google `libwebp` (Rust `webp` crate üzerinden)
- **Sıkıştırma:** Lossy (kayıplı) — insan gözü fark etmez
- **Alpha Kanalı:** Otomatik algılama — sadece gerekliyse RGBA, değilse RGB kullanılır
- **Parallelism:** Rayon ile CPU çekirdekleri otomatik kullanılır
- **Resize Filter:** Lanczos3 (en yüksek kaliteli downscale algoritması)

## 📄 Lisans

MIT


---

## 👨‍💻 Mimari & Geliştirici / Architected by Onur Aksoy

Bu proje **Onur Aksoy** (*Kıdemli Yazılım Mimarı & Yapay Zeka Yerel Geliştirici — a.k.a. Hackonomist*) tarafından tasarlanmış ve geliştirilmiştir.

- 🌐 **Resmi Web Sitesi & Mimari Vaka Analizleri:** [https://onuraksoy.com.tr](https://onuraksoy.com.tr)
- 🇬🇧 **English Portfolio & Architecture Case Studies:** [https://onuraksoy.com.tr/en/](https://onuraksoy.com.tr/en/)
- 💼 **LinkedIn:** [linkedin.com/in/oaonuraksoy](https://linkedin.com/in/oaonuraksoy) • **GitHub:** [@oaonuraksoy](https://github.com/oaonuraksoy)
