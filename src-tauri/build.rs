use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const WHISPER_VERSION: &str = "1.8.2";
const GITHUB_RELEASE_BASE: &str = "https://github.com/ggml-org/whisper.cpp/releases/download";

fn main() {
    tauri_build::build();

    // Only download binaries on Windows and macOS
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "linux" {
        println!("cargo:warning=Linux build: using whisper-rs crate (builds from source)");
        return;
    }

    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));

    // Determine which binary to download
    let (zip_name, lib_name) = match (target_os.as_str(), target_arch.as_str()) {
        ("windows", "x86_64") => ("whisper-bin-x64.zip", "whisper.dll"),
        ("windows", "x86") => ("whisper-bin-Win32.zip", "whisper.dll"),
        ("macos", _) => (
            &format!("whisper-v{}-xcframework.zip", WHISPER_VERSION) as &str,
            "libwhisper.dylib",
        ),
        _ => {
            println!("cargo:warning=Unsupported platform: {}-{}", target_os, target_arch);
            return;
        }
    };

    // Cache directory for downloaded files
    let cache_dir = out_dir.join("whisper-cache");
    fs::create_dir_all(&cache_dir).expect("Failed to create cache directory");

    let zip_path = cache_dir.join(format!("whisper-{}-{}.zip", WHISPER_VERSION, target_arch));
    let lib_output_dir = out_dir.join("whisper-lib");
    fs::create_dir_all(&lib_output_dir).expect("Failed to create lib output directory");

    let lib_path = lib_output_dir.join(lib_name);

    // Check if we already have the library
    if !lib_path.exists() {
        // Download if not cached
        if !zip_path.exists() {
            let url = format!("{}/v{}/{}", GITHUB_RELEASE_BASE, WHISPER_VERSION, zip_name);
            println!("cargo:warning=Downloading whisper.cpp binary from: {}", url);
            download_file(&url, &zip_path).expect("Failed to download whisper.cpp binary");
        }

        // Extract the library
        println!("cargo:warning=Extracting whisper.cpp library...");
        extract_library(&zip_path, &lib_output_dir, lib_name, &target_os, &target_arch)
            .expect("Failed to extract whisper.cpp library");
    }

    // Set linker flags
    println!("cargo:rustc-link-search=native={}", lib_output_dir.display());

    // For Windows, we need to tell the linker about the import library
    if target_os == "windows" {
        // The DLL doesn't have an import lib in the release, so we use runtime loading
        // Just ensure the DLL can be found at runtime
    }

    // Copy library to target directory for runtime
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let target_dir = out_dir
        .ancestors()
        .find(|p| p.ends_with("target") || p.file_name().map(|n| n == "target").unwrap_or(false))
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| out_dir.join("..").join("..").join(".."));

    let runtime_lib_dir = target_dir.join(&profile);
    if runtime_lib_dir.exists() {
        let runtime_lib_path = runtime_lib_dir.join(lib_name);
        if !runtime_lib_path.exists() || fs::metadata(&lib_path).map(|m| m.len()).unwrap_or(0)
            != fs::metadata(&runtime_lib_path).map(|m| m.len()).unwrap_or(0)
        {
            fs::copy(&lib_path, &runtime_lib_path).ok();
            println!("cargo:warning=Copied {} to {}", lib_name, runtime_lib_dir.display());
        }
    }

    // Also write the library path to a file for runtime discovery
    let lib_path_file = out_dir.join("whisper_lib_path.txt");
    fs::write(&lib_path_file, lib_path.to_string_lossy().as_bytes())
        .expect("Failed to write library path file");

    // Rerun if build.rs changes
    println!("cargo:rerun-if-changed=build.rs");
}

fn download_file(url: &str, dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let response = reqwest::blocking::Client::builder()
        .user_agent("flowstt-build")
        .build()?
        .get(url)
        .send()?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {} for URL: {}", response.status(), url).into());
    }

    let bytes = response.bytes()?;
    let mut file = fs::File::create(dest)?;
    file.write_all(&bytes)?;

    Ok(())
}

fn extract_library(
    zip_path: &Path,
    output_dir: &Path,
    lib_name: &str,
    target_os: &str,
    target_arch: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    if target_os == "macos" {
        // xcframework structure is complex - find the dylib for the right architecture
        let framework_arch = match target_arch {
            "x86_64" => "macos-x86_64",
            "aarch64" => "macos-arm64",
            _ => "macos-arm64", // default to arm64
        };

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();

            // Look for the dylib in the correct architecture folder
            if name.contains(framework_arch) && name.ends_with(".dylib") {
                let output_path = output_dir.join(lib_name);
                let mut output_file = fs::File::create(&output_path)?;
                io::copy(&mut file, &mut output_file)?;
                println!("cargo:warning=Extracted {} from {}", lib_name, name);
                return Ok(());
            }
        }

        // Fallback: look for any dylib
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();

            if name.ends_with(".dylib") && !name.contains("ios") {
                let output_path = output_dir.join(lib_name);
                let mut output_file = fs::File::create(&output_path)?;
                io::copy(&mut file, &mut output_file)?;
                println!("cargo:warning=Extracted {} from {} (fallback)", lib_name, name);
                return Ok(());
            }
        }

        return Err("Could not find dylib in xcframework".into());
    } else {
        // Windows: find the DLL in the archive
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();

            if name.ends_with(lib_name) {
                let output_path = output_dir.join(lib_name);
                let mut output_file = fs::File::create(&output_path)?;
                io::copy(&mut file, &mut output_file)?;
                println!("cargo:warning=Extracted {}", lib_name);
                return Ok(());
            }
        }

        return Err(format!("Could not find {} in archive", lib_name).into());
    }
}
