use anyhow::Result;
use msixbundle::*;
use std::env;
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    let tools = locate_sdk_tools()?;

    let output = Path::new("./target/msix/");
    if !output.exists() {
        fs::create_dir_all(output)?;
    }

    // Default to running architecture, but allow override via PACK_ARCH env var
    #[cfg(target_arch = "x86_64")]
    let arch = env::var("PACK_ARCH").unwrap_or_else(|_| "x86_64".to_string());

    #[cfg(target_arch = "aarch64")]
    let arch = env::var("PACK_ARCH").unwrap_or_else(|_| "aarch64".to_string());
    
    let (manifest_path, arch_name) = match arch.to_lowercase().as_str() {
        "x86_64" | "x64" => (Path::new("./build/msix/x64"), "x64"),
        "aarch64" | "arm64" => (Path::new("./build/msix/arm64"), "arm64"),
        _ => {
            anyhow::bail!("Unsupported architecture: {}. Use PACK_ARCH=x64 or PACK_ARCH=arm64", arch);
        }
    };

    println!("--- MSIX Packaging ---");
    println!("Target Architecture: {}", arch_name);

    if !manifest_path.exists() {
        anyhow::bail!("Manifest path not found: {}", manifest_path.display());
    }

    let manifest_info = read_manifest_info(manifest_path)?;

    // Search for modelcheck.exe
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let target_dir = Path::new(&manifest_dir).join("target");
    
    // Common paths to check, prioritizing Dioxus and specific architecture folders
    let search_paths = vec![
        target_dir.join("dx/modelcheck/release/windows/app/modelcheck.exe"),
        target_dir.join("x86_64-pc-windows-msvc/release/modelcheck.exe"),
        target_dir.join("aarch64-pc-windows-msvc/release/modelcheck.exe"),
        target_dir.join("release/modelcheck.exe"),
    ];

    let mut found_bin = None;
    for path in search_paths {
        if path.exists() {
            found_bin = Some(path);
            break;
        }
    }

    let bin_src = found_bin.ok_or_else(|| {
        anyhow::anyhow!("Could not find modelcheck.exe in target directory. Run 'dx build --release' first.")
    })?;

    println!("Found binary: {}", bin_src.display());

    // Copy to manifest path
    let dest = manifest_path.join("modelcheck.exe");
    fs::copy(&bin_src, &dest)?;
    println!("Copied binary to {}", dest.display());

    println!("Generating visual assets...");
    generate_assets(manifest_path)?;

    println!("Compiling resources...");
    compile_resources_pri(
        &tools,
        &PriOptions {
            appx_content_dir: manifest_path,
            default_language: "en-us",
            target_os_version: "10.0.0",
            keep_priconfig: false,
            overwrite: true,
            makepri_override: None,
        },
    )?;

    println!("Packing MSIX...");
    let msix = pack_arch(
        &tools,
        manifest_path,
        output,
        &manifest_info,
        arch_name,
        true,
    )?;

    println!("Success! MSIX created at: {}", msix.display());

    Ok(())
}

fn generate_assets(manifest_path: &Path) -> Result<()> {
    let assets_dir = manifest_path.join("Assets");
    if !assets_dir.exists() {
        fs::create_dir_all(&assets_dir)?;
    }

    let svg_path = Path::new("assets/logo-padded.svg");
    if !svg_path.exists() {
        anyhow::bail!("Source logo not found at {}", svg_path.display());
    }
    let svg_data = fs::read(svg_path)?;

    let opt = resvg::usvg::Options::default();
    let rtree = resvg::usvg::Tree::from_data(&svg_data, &opt)?;
    let size = rtree.size();

    let targets = vec![
        ("StoreLogo.png", 50, 50),
        ("Square150x150Logo.png", 150, 150),
        ("Square44x44Logo.png", 44, 44),
        ("Wide310x150.png", 310, 150),
    ];

    for (name, w, h) in targets {
        let mut pixmap = resvg::tiny_skia::Pixmap::new(w, h).ok_or_else(|| {
            anyhow::anyhow!("Failed to create pixmap for {}", name)
        })?;

        // Calculate scale to fit while maintaining aspect ratio
        let sx = w as f32 / size.width();
        let sy = h as f32 / size.height();
        let scale = sx.min(sy);

        // Calculate translation to center the logo
        let tx = (w as f32 - size.width() * scale) / 2.0;
        let ty = (h as f32 - size.height() * scale) / 2.0;

        let transform = resvg::usvg::Transform::from_row(scale, 0.0, 0.0, scale, tx, ty);
        resvg::render(&rtree, transform, &mut pixmap.as_mut());

        let out_path = assets_dir.join(name);
        pixmap.save_png(&out_path).map_err(|e| {
            anyhow::anyhow!("Failed to save {}: {}", name, e)
        })?;
        println!("  - Generated {}", name);
    }

    Ok(())
}
