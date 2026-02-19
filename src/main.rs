use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::Mutex;
use std::time::Instant;

use colored::Colorize;
use dialoguer::{Confirm, Select};
use humansize::{format_size, BINARY};
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use walkdir::WalkDir;

/// Desteklenen dosya uzantıları
const SUPPORTED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "bmp", "tiff", "tif"];

/// Çıktı klasörü adı
const OUTPUT_DIR: &str = "oa_optimize";

/// Kalite profilleri
#[derive(Clone, Copy)]
struct QualityProfile {
    name: &'static str,
    description: &'static str,
    /// WebP lossy kalite (0-100). Daha düşük = daha küçük dosya
    quality: f32,
    /// Yatay görseller için maksimum genişlik
    max_landscape_width: u32,
    /// Dikey/kare görseller için maksimum yükseklik
    max_portrait_height: u32,
}

const PROFILES: &[QualityProfile] = &[
    QualityProfile {
        name: "🌐 Web (Maksimum Sıkıştırma)",
        description: "Web siteleri için ideal. Çok küçük dosya boyutu, görsel kalite hâlâ iyi.",
        quality: 70.0,
        max_landscape_width: 1024,
        max_portrait_height: 768,
    },
    QualityProfile {
        name: "⚖️  Dengeli (Önerilen)",
        description: "Kalite ve boyut arasında en iyi denge. Çoğu kullanım için ideal.",
        quality: 78.0,
        max_landscape_width: 1280,
        max_portrait_height: 960,
    },
    QualityProfile {
        name: "🎨 Yüksek Kalite",
        description: "Fotoğraf portföyü ve baskı kalitesi. Daha büyük dosya boyutu.",
        quality: 88.0,
        max_landscape_width: 1920,
        max_portrait_height: 1440,
    },
];

/// Dosya bilgisi yapısı
struct FileInfo {
    path: PathBuf,
    file_name: String,
    size: u64,
}

/// İşlem sonucu yapısı
struct ProcessResult {
    file_name: String,
    original_size: u64,
    new_size: u64,
    success: bool,
    error_msg: Option<String>,
}

fn main() {
    // Windows'ta ANSI renk desteğini etkinleştir
    #[cfg(target_os = "windows")]
    {
        let _ = colored::control::set_virtual_terminal(true);
    }

    // Adım 1 — Karşılama ekranı
    print_banner();

    // Adım 2 — Analiz
    let files = scan_files();
    if files.is_empty() {
        println!(
            "\n{}",
            "📂 Bu klasörde desteklenen resim dosyası bulunamadı (.jpg, .jpeg, .png, .webp, .bmp, .tiff)"
                .yellow()
        );
        wait_for_enter();
        process::exit(0);
    }
    print_file_list(&files);

    // Adım 3 — Kalite profili seçimi
    let profile = select_quality_profile();

    // Adım 4 — Onay
    if !ask_confirmation(&profile) {
        println!("\n{}", "⛔ İşlem iptal edildi.".red().bold());
        wait_for_enter();
        process::exit(0);
    }

    // Adım 5 — Klasör oluşturma
    create_output_dir();

    // Adım 6 — Dönüştürme & Optimize
    let results = process_files(&files, &profile);

    // Adım 7 — Özet
    print_summary(&results);

    // Programı kapatmadan önce kullanıcının sonuçları görmesini bekle
    wait_for_enter();
}

/// Windows'ta çift tıklayınca pencerenin kapanmaması için Enter bekle
fn wait_for_enter() {
    println!(
        "\n{}",
        "Kapatmak için Enter tuşuna basın...".white().dimmed()
    );
    let _ = io::stdin().read(&mut [0u8]);
}

/// Renkli karşılama banner'ı yazdır
fn print_banner() {
    println!();
    println!(
        "{}",
        "╔══════════════════════════════════════════╗".cyan().bold()
    );
    println!(
        "{}",
        "║        🖼  IMG OPTIMIZE  v2.0.0          ║".cyan().bold()
    );
    println!(
        "{}",
        "║   WebP dönüştürücü & görsel optimize     ║".cyan().bold()
    );
    println!(
        "{}",
        "╚══════════════════════════════════════════╝".cyan().bold()
    );
    println!();
}

/// Mevcut klasördeki desteklenen resim dosyalarını tara
fn scan_files() -> Vec<FileInfo> {
    println!("{}", "📂 Klasör taranıyor...".white().bold());
    println!();

    let mut files: Vec<FileInfo> = Vec::new();

    for entry in WalkDir::new(".")
        .max_depth(1)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path().to_path_buf();

        // Dizinleri atla
        if path.is_dir() {
            continue;
        }

        // oa_optimize klasöründeki dosyaları atla
        if path.components().any(|c| c.as_os_str() == OUTPUT_DIR) {
            continue;
        }

        // Uzantıyı kontrol et
        if let Some(ext) = path.extension() {
            let ext_lower = ext.to_string_lossy().to_lowercase();
            if SUPPORTED_EXTENSIONS.contains(&ext_lower.as_str()) {
                let file_name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                let size = match fs::metadata(&path) {
                    Ok(meta) => meta.len(),
                    Err(_) => continue,
                };

                files.push(FileInfo {
                    path,
                    file_name,
                    size,
                });
            }
        }
    }

    // Dosya adına göre sırala
    files.sort_by(|a, b| a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase()));

    files
}

/// Bulunan dosyaları listele
fn print_file_list(files: &[FileInfo]) {
    println!("  {}:", "Bulunan dosyalar".white().bold());

    let total_size: u64 = files.iter().map(|f| f.size).sum();

    for (i, file) in files.iter().enumerate() {
        let prefix = if i == files.len() - 1 {
            "  └─"
        } else {
            "  ├─"
        };
        let size_str = format_size(file.size, BINARY);
        println!(
            "{} {:<30} ({})",
            prefix.white(),
            file.file_name.white(),
            size_str.yellow()
        );
    }

    println!();
    println!(
        "  {}: {} dosya | {}",
        "Toplam".white().bold(),
        files.len().to_string().green().bold(),
        format_size(total_size, BINARY).green().bold()
    );
    println!();
}

/// Kalite profili seçimi
fn select_quality_profile() -> QualityProfile {
    println!("{}", "🎯 Kalite profili seçin:".white().bold());
    println!();

    let items: Vec<String> = PROFILES
        .iter()
        .map(|p| format!("{}\n     {}", p.name, p.description.dimmed()))
        .collect();

    let selection = Select::new()
        .with_prompt("Profil")
        .items(&items)
        .default(0)
        .interact()
        .unwrap_or(0);

    let selected = PROFILES[selection];

    println!();
    println!(
        "  {} kalite={}, max genişlik={}px, max yükseklik={}px",
        "📋 Seçilen ayarlar:".white().bold(),
        format!("{}%", selected.quality).cyan(),
        selected.max_landscape_width.to_string().cyan(),
        selected.max_portrait_height.to_string().cyan(),
    );
    println!();

    selected
}

/// Kullanıcıdan onay al
fn ask_confirmation(profile: &QualityProfile) -> bool {
    let prompt = format!("❓ {} profili ile optimize edilecek. Devam?", profile.name);

    match Confirm::new().with_prompt(prompt).default(true).interact() {
        Ok(confirmed) => confirmed,
        Err(e) => {
            eprintln!("{} Onay alınamadı: {}", "❌".red(), e);
            false
        }
    }
}

/// Çıktı klasörünü oluştur
fn create_output_dir() {
    if let Err(e) = fs::create_dir_all(OUTPUT_DIR) {
        eprintln!("{} Çıktı klasörü oluşturulamadı: {}", "❌".red().bold(), e);
        process::exit(1);
    }
}

/// Tüm dosyaları paralel olarak işle
fn process_files(files: &[FileInfo], profile: &QualityProfile) -> Vec<ProcessResult> {
    println!();

    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("⚙  İşleniyor... [{bar:40.cyan/blue}] {pos}/{len}  {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_bar())
            .progress_chars("█░░"),
    );

    let start_time = Instant::now();
    let results: Mutex<Vec<ProcessResult>> = Mutex::new(Vec::with_capacity(files.len()));
    let profile = *profile; // Copy for thread safety

    files.par_iter().for_each(|file| {
        let result = process_single_file(file, &profile);

        // Progress bar'ı güncelle
        if result.success {
            pb.set_message(result.file_name.clone());
        } else if let Some(ref msg) = result.error_msg {
            pb.println(format!(
                "{}  Atlandı: {} — Hata: {}",
                "⚠".yellow(),
                file.file_name.yellow(),
                msg.yellow()
            ));
        }

        pb.inc(1);

        if let Ok(mut r) = results.lock() {
            r.push(result);
        }
    });

    pb.finish_and_clear();

    let elapsed = start_time.elapsed();
    println!("\n⏱  İşlem süresi: {:.1} saniye\n", elapsed.as_secs_f64());

    let mut final_results = results.into_inner().unwrap_or_default();
    // Dosya adına göre sırala
    final_results.sort_by(|a, b| a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase()));
    final_results
}

/// Görselin gerçekten alpha (şeffaflık) bilgisi içerip içermediğini kontrol et
fn has_real_alpha(img: &DynamicImage) -> bool {
    match img {
        DynamicImage::ImageRgba8(rgba) => {
            // Tüm piksellerin alpha değeri 255 mi kontrol et
            // Eğer hepsi 255 ise, alpha kanalı gereksiz
            rgba.pixels().any(|p| p.0[3] < 255)
        }
        DynamicImage::ImageRgba16(rgba) => rgba.pixels().any(|p| p.0[3] < 65535),
        DynamicImage::ImageRgba32F(rgba) => rgba.pixels().any(|p| p.0[3] < 1.0),
        DynamicImage::ImageLuma8(_)
        | DynamicImage::ImageLuma16(_)
        | DynamicImage::ImageRgb8(_)
        | DynamicImage::ImageRgb16(_)
        | DynamicImage::ImageRgb32F(_)
        | DynamicImage::ImageLumaA8(_)
        | DynamicImage::ImageLumaA16(_) => false,
        _ => false,
    }
}

/// Tek bir dosyayı işle: oku, boyutlandır, WebP olarak kaydet
fn process_single_file(file: &FileInfo, profile: &QualityProfile) -> ProcessResult {
    // Dosya adından çıktı adı oluştur (.webp uzantısı ile)
    let stem = Path::new(&file.file_name)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let output_name = format!("{}.webp", stem);
    let output_path = Path::new(OUTPUT_DIR).join(&output_name);

    // Görseli oku
    let img = match image::open(&file.path) {
        Ok(img) => img,
        Err(e) => {
            return ProcessResult {
                file_name: output_name,
                original_size: file.size,
                new_size: 0,
                success: false,
                error_msg: Some(format!("{}", e)),
            };
        }
    };

    // Boyutlandırma kuralları
    let (width, height) = img.dimensions();
    let resized = if width > height {
        // Yatay görsel
        if width > profile.max_landscape_width {
            img.resize(profile.max_landscape_width, u32::MAX, FilterType::Lanczos3)
        } else {
            img
        }
    } else {
        // Dikey veya kare görsel
        if height > profile.max_portrait_height {
            img.resize(u32::MAX, profile.max_portrait_height, FilterType::Lanczos3)
        } else {
            img
        }
    };

    // Alpha kanalı kontrolü — gereksizse RGB kullan (daha küçük dosya)
    let use_alpha = has_real_alpha(&resized);

    // webp crate ile lossy encode
    let webp_data = if use_alpha {
        // RGBA — şeffaflık var
        let rgba = resized.to_rgba8();
        let (w, h) = (rgba.width(), rgba.height());
        let encoder = webp::Encoder::from_rgba(rgba.as_raw(), w, h);
        encoder.encode(profile.quality)
    } else {
        // RGB — şeffaflık yok, daha küçük dosya
        let rgb = resized.to_rgb8();
        let (w, h) = (rgb.width(), rgb.height());
        let encoder = webp::Encoder::from_rgb(rgb.as_raw(), w, h);
        encoder.encode(profile.quality)
    };

    // Dosyaya yaz
    if let Err(e) = fs::write(&output_path, &*webp_data) {
        return ProcessResult {
            file_name: output_name,
            original_size: file.size,
            new_size: 0,
            success: false,
            error_msg: Some(format!("Dosya yazılamadı: {}", e)),
        };
    }

    // Çıktı dosya boyutunu hesapla
    let new_size = webp_data.len() as u64;

    ProcessResult {
        file_name: output_name,
        original_size: file.size,
        new_size,
        success: true,
        error_msg: None,
    }
}

/// Sonuç özet tablosunu yazdır
fn print_summary(results: &[ProcessResult]) {
    let successful: Vec<&ProcessResult> = results.iter().filter(|r| r.success).collect();

    if successful.is_empty() {
        println!("{}", "❌ Hiçbir dosya işlenemedi!".red().bold());
        return;
    }

    println!("{}", "✅ Tamamlandı!".green().bold());
    println!();

    // Tablo başlığı
    println!(
        "  {:<25} {:>10} {:>10} {:>10}",
        "Dosya".white().bold(),
        "Önce".white().bold(),
        "Sonra".white().bold(),
        "Kazanım".white().bold()
    );
    println!("  {}", "─".repeat(57).white());

    let mut total_original: u64 = 0;
    let mut total_new: u64 = 0;

    for result in &successful {
        total_original += result.original_size;
        total_new += result.new_size;

        let original_str = format_size(result.original_size, BINARY);
        let new_str = format_size(result.new_size, BINARY);
        let savings = calculate_savings(result.original_size, result.new_size);

        let savings_colored = if result.new_size < result.original_size {
            savings.green()
        } else {
            savings.red()
        };

        println!(
            "  {:<25} {:>10} {:>10} {:>10}",
            result.file_name.white(),
            original_str.white(),
            new_str.white(),
            savings_colored
        );
    }

    println!("  {}", "─".repeat(57).white());

    let total_original_str = format_size(total_original, BINARY);
    let total_new_str = format_size(total_new, BINARY);
    let total_savings = calculate_savings(total_original, total_new);

    let total_savings_colored = if total_new < total_original {
        total_savings.green().bold()
    } else {
        total_savings.red().bold()
    };

    println!(
        "  {:<25} {:>10} {:>10} {:>10}",
        "TOPLAM".white().bold(),
        total_original_str.white().bold(),
        total_new_str.white().bold(),
        total_savings_colored
    );

    println!();
    println!(
        "📁 Dosyalar buraya kaydedildi: {}",
        format!("./{}/", OUTPUT_DIR).cyan().bold()
    );
    println!();

    // Atlanan dosyaları listele
    let skipped: Vec<&ProcessResult> = results.iter().filter(|r| !r.success).collect();
    if !skipped.is_empty() {
        println!(
            "{}",
            format!("⚠  {} dosya atlandı (hata)", skipped.len())
                .yellow()
                .bold()
        );
    }
}

/// Kazanım yüzdesini hesapla
fn calculate_savings(original: u64, new: u64) -> String {
    if original == 0 {
        return "0%".to_string();
    }

    if new <= original {
        let percent = (1.0 - (new as f64 / original as f64)) * 100.0;
        format!("-{:.1}%", percent)
    } else {
        let percent = ((new as f64 / original as f64) - 1.0) * 100.0;
        format!("+{:.1}%", percent)
    }
}
