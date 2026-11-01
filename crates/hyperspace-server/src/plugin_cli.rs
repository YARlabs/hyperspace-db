use crate::server_args::PluginCommands;
use hyperspace_pdk::package::HspPackage;
use std::path::{Path, PathBuf};

pub async fn handle_plugin_cli(
    action: &PluginCommands,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match action {
        PluginCommands::Pack { dir, out } => {
            let manifest_path = dir.join("plugin.toml");
            if !manifest_path.exists() {
                eprintln!("❌ Error: 'plugin.toml' not found in '{}'", dir.display());
                std::process::exit(1);
            }

            let target_arch = format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS);
            let out_path = out.clone().unwrap_or_else(|| {
                let dir_name = dir.file_name().and_then(|s| s.to_str()).unwrap_or("plugin");
                PathBuf::from(format!("{dir_name}-{target_arch}.hsp"))
            });

            println!(
                "📦 Packing plugin from {} to {}...",
                dir.display(),
                out_path.display()
            );
            match HspPackage::pack_dir(dir, &out_path, None) {
                Ok(created_file) => {
                    let meta = std::fs::metadata(&created_file)?;
                    println!(
                        "✅ Successfully created package: {} ({} bytes)",
                        created_file.display(),
                        meta.len()
                    );
                }
                Err(e) => {
                    eprintln!("❌ Packing failed: {e}");
                    std::process::exit(1);
                }
            }
        }
        PluginCommands::Install {
            target,
            dir,
            offline,
        } => {
            std::fs::create_dir_all(dir)?;
            let path = Path::new(target);
            if path.exists() && path.is_file() {
                println!(
                    "📦 Installing local .hsp package: {} into {}...",
                    target,
                    dir.display()
                );
                match HspPackage::unpack_file(path, dir, &[]) {
                    Ok(res) => {
                        println!(
                            "✅ Successfully installed '{}' v{} into {}",
                            res.name,
                            res.version,
                            dir.display()
                        );
                        println!("💡 Binary unpacked at: {}", res.library_path.display());
                        println!("💡 Tip: If server is running, load it hot with POST /v1/extensions/upload or restart hyperspace-server.");
                    }
                    Err(e) => {
                        eprintln!("❌ Installation failed: {e}");
                        std::process::exit(1);
                    }
                }
            } else if *offline {
                eprintln!(
                    "❌ Error: File '{}' not found and --offline flag is set.",
                    target
                );
                std::process::exit(1);
            } else {
                println!(
                    "🌐 Fetching plugin '{}' from Hyperspace Hub (https://the.yar.ink)...",
                    target
                );
                let target_arch = format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS);
                let url = format!(
                    "https://the.yar.ink/v1/hub/download/{}/latest/{}",
                    target, target_arch
                );
                let client = reqwest::Client::builder().use_rustls_tls().build()?;
                let resp = client.get(&url).send().await?;
                if !resp.status().is_success() {
                    eprintln!(
                        "❌ Hub returned HTTP error {}: failed to download {}",
                        resp.status(),
                        target
                    );
                    std::process::exit(1);
                }
                let bytes = resp.bytes().await?;
                match HspPackage::unpack(&bytes, dir, &[]) {
                    Ok(res) => {
                        println!(
                            "✅ Successfully installed '{}' v{} into {}",
                            res.name,
                            res.version,
                            dir.display()
                        );
                        println!("💡 Binary unpacked at: {}", res.library_path.display());
                        println!("💡 Tip: If server is running, load it hot with POST /v1/extensions/upload or restart hyperspace-server.");
                    }
                    Err(e) => {
                        eprintln!("❌ Installation failed: {e}");
                        std::process::exit(1);
                    }
                }
            }
        }
    }
    Ok(())
}
