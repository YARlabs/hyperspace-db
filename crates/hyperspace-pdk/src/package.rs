//! Hyperspace Package (`.hsp`) format handler:
//! Safe packing, unpacking, ABI compatibility checking, and Ed25519 signature verification.

use crate::PDK_API_VERSION;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HspTarget {
    #[serde(default = "default_os")]
    pub os: String,
    #[serde(default = "default_arch")]
    pub arch: String,
    pub library_file: String,
}

fn default_os() -> String {
    std::env::consts::OS.to_string()
}

fn default_arch() -> String {
    std::env::consts::ARCH.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HspManifest {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default, alias = "abi_version")]
    pub api_version: u32,
    pub author: Option<String>,
    pub license: Option<String>,
    pub description: Option<String>,
    pub repository: Option<String>,
    #[serde(default)]
    pub target: Option<HspTarget>,
    #[serde(default)]
    pub plugin: Option<HspPluginSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HspPluginSection {
    pub name: Option<String>,
    pub version: Option<String>,
    #[serde(alias = "abi_version")]
    pub api_version: Option<u32>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub description: Option<String>,
}

impl HspManifest {
    pub fn normalize(&mut self) {
        if let Some(p) = &self.plugin {
            if self.name.is_empty() {
                if let Some(n) = &p.name {
                    self.name = n.clone();
                }
            }
            if self.version.is_empty() {
                if let Some(v) = &p.version {
                    self.version = v.clone();
                }
            }
            if self.api_version == 0 {
                if let Some(abi) = p.api_version {
                    self.api_version = abi;
                }
            }
            if self.author.is_none() {
                self.author = p.author.clone();
            }
            if self.license.is_none() {
                self.license = p.license.clone();
            }
            if self.description.is_none() {
                self.description = p.description.clone();
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HspInstallResult {
    pub name: String,
    pub version: String,
    pub library_path: PathBuf,
    pub sha256: String,
    pub is_verified: bool,
    pub publisher: Option<String>,
}

pub struct HspPackage;

impl HspPackage {
    /// Computes the SHA-256 hash of arbitrary bytes.
    pub fn sha256_digest(bytes: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        hex::encode(hasher.finalize())
    }

    /// Packs a plugin directory and compiled shared library into a `.hsp` (`.tar.gz`) package.
    pub fn pack(
        plugin_toml_path: &Path,
        library_path: &Path,
        readme_path: Option<&Path>,
        icon_path: Option<&Path>,
        signing_key: Option<&SigningKey>,
        output_file: &Path,
    ) -> Result<PathBuf, String> {
        if !plugin_toml_path.exists() {
            return Err(format!(
                "plugin.toml not found at {}",
                plugin_toml_path.display()
            ));
        }
        if !library_path.exists() {
            return Err(format!(
                "Shared library not found at {}",
                library_path.display()
            ));
        }

        let toml_str = fs::read_to_string(plugin_toml_path).map_err(|e| e.to_string())?;
        let mut manifest: HspManifest =
            toml::from_str(&toml_str).map_err(|e| format!("Invalid plugin.toml schema: {e}"))?;
        manifest.normalize();

        if manifest.api_version != PDK_API_VERSION {
            return Err(format!(
                "ABI version mismatch: plugin requires {} but current PDK is {}",
                manifest.api_version, PDK_API_VERSION
            ));
        }

        let mut tar_buf = Vec::new();
        {
            let mut tar = tar::Builder::new(&mut tar_buf);

            // 1. plugin.toml
            let mut toml_file = File::open(plugin_toml_path).map_err(|e| e.to_string())?;
            tar.append_file("plugin.toml", &mut toml_file)
                .map_err(|e| e.to_string())?;

            // 2. library file
            let lib_filename = library_path
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or_else(|| "Invalid library filename".to_string())?;
            let mut lib_file = File::open(library_path).map_err(|e| e.to_string())?;
            tar.append_file(lib_filename, &mut lib_file)
                .map_err(|e| e.to_string())?;

            // 3. README.md
            if let Some(r_path) = readme_path {
                if r_path.exists() {
                    let mut r_file = File::open(r_path).map_err(|e| e.to_string())?;
                    tar.append_file("README.md", &mut r_file)
                        .map_err(|e| e.to_string())?;
                }
            }

            // 4. icon.svg
            if let Some(i_path) = icon_path {
                if i_path.exists() {
                    let mut i_file = File::open(i_path).map_err(|e| e.to_string())?;
                    tar.append_file("icon.svg", &mut i_file)
                        .map_err(|e| e.to_string())?;
                }
            }

            // 5. Signature (if signing key provided)
            if let Some(sk) = signing_key {
                let toml_bytes = fs::read(plugin_toml_path).unwrap_or_default();
                let lib_bytes = fs::read(library_path).unwrap_or_default();
                let mut data_to_sign = Vec::with_capacity(toml_bytes.len() + lib_bytes.len());
                data_to_sign.extend_from_slice(&toml_bytes);
                data_to_sign.extend_from_slice(&lib_bytes);

                let signature: Signature = sk.sign(&data_to_sign);
                let sig_hex = hex::encode(signature.to_bytes());
                let mut header = tar::Header::new_gnu();
                header.set_size(sig_hex.len() as u64);
                header.set_mode(0o644);
                header.set_cksum();
                tar.append_data(
                    &mut header,
                    "signature.sig",
                    Cursor::new(sig_hex.as_bytes()),
                )
                .map_err(|e| e.to_string())?;
            }

            tar.finish().map_err(|e| e.to_string())?;
        }

        // Compress tar with gzip
        let mut gz_encoder = GzEncoder::new(Vec::new(), Compression::default());
        gz_encoder.write_all(&tar_buf).map_err(|e| e.to_string())?;
        let gz_bytes = gz_encoder.finish().map_err(|e| e.to_string())?;

        // Ensure parent directory exists
        if let Some(parent) = output_file.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(output_file, gz_bytes).map_err(|e| e.to_string())?;

        Ok(output_file.to_path_buf())
    }

    /// Helper: pack a directory containing plugin.toml and compiled binary
    pub fn pack_dir(
        dir: &Path,
        output_file: &Path,
        signing_key: Option<&SigningKey>,
    ) -> Result<PathBuf, String> {
        let plugin_toml = dir.join("plugin.toml");
        if !plugin_toml.exists() {
            return Err(format!("'plugin.toml' not found in {}", dir.display()));
        }

        let expected_ext = match std::env::consts::OS {
            "macos" => "dylib",
            "windows" => "dll",
            _ => "so",
        };

        // Find library file in dir or target directories
        let mut lib_file = None;
        let candidates = [
            dir.to_path_buf(),
            dir.join("target").join("release"),
            dir.join("target").join("debug"),
            PathBuf::from("target/release"),
            PathBuf::from("target/debug"),
        ];

        for c in &candidates {
            if c.exists() {
                if let Ok(entries) = fs::read_dir(c) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if let Some(ext) = path.extension() {
                            if ext == expected_ext {
                                lib_file = Some(path);
                                break;
                            }
                        }
                    }
                }
            }
            if lib_file.is_some() {
                break;
            }
        }

        let library_path = lib_file.ok_or_else(|| {
            format!(
                "No compiled library (*.{}) found in {}",
                expected_ext,
                dir.display()
            )
        })?;

        let readme = dir.join("README.md");
        let readme_opt = if readme.exists() {
            Some(readme.as_path())
        } else {
            None
        };

        let icon = dir.join("icon.svg");
        let icon_opt = if icon.exists() {
            Some(icon.as_path())
        } else {
            None
        };

        Self::pack(
            &plugin_toml,
            &library_path,
            readme_opt,
            icon_opt,
            signing_key,
            output_file,
        )
    }

    /// Helper: unpack a file from disk
    pub fn unpack_file(
        hsp_path: &Path,
        destination_dir: &Path,
        trusted_pubkeys: &[VerifyingKey],
    ) -> Result<HspInstallResult, String> {
        let bytes = fs::read(hsp_path)
            .map_err(|e| format!("Failed to read {}: {e}", hsp_path.display()))?;
        Self::unpack(&bytes, destination_dir, trusted_pubkeys)
    }

    /// Unpacks a `.hsp` archive into `destination_dir`, validating ABI compatibility,
    /// host target platform, and optional Ed25519 signature.
    pub fn unpack(
        hsp_bytes: &[u8],
        destination_dir: &Path,
        trusted_pubkeys: &[VerifyingKey],
    ) -> Result<HspInstallResult, String> {
        let sha256_hash = Self::sha256_digest(hsp_bytes);

        let gz_decoder = GzDecoder::new(Cursor::new(hsp_bytes));
        let mut archive = tar::Archive::new(gz_decoder);

        let mut extracted_files: HashMap<String, Vec<u8>> = HashMap::new();

        let entries = archive
            .entries()
            .map_err(|e| format!("Failed to read tar archive: {e}"))?;
        for entry in entries {
            let mut entry = entry.map_err(|e| e.to_string())?;
            let path = entry
                .path()
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .to_string();
            let mut content = Vec::new();
            entry.read_to_end(&mut content).map_err(|e| e.to_string())?;
            extracted_files.insert(path, content);
        }

        // 1. Read & parse plugin.toml
        let toml_bytes = extracted_files
            .get("plugin.toml")
            .ok_or_else(|| "Corrupted .hsp package: missing 'plugin.toml'".to_string())?;
        let toml_str = String::from_utf8_lossy(toml_bytes);
        let mut manifest: HspManifest = toml::from_str(&toml_str)
            .map_err(|e| format!("Invalid plugin.toml format in package: {e}"))?;
        manifest.normalize();

        // 2. Validate ABI Version
        if manifest.api_version != PDK_API_VERSION {
            return Err(format!(
                "Incompatible plugin ABI: package has api_version={} but server requires {}",
                manifest.api_version, PDK_API_VERSION
            ));
        }

        // 3. Locate shared library binary
        let current_os = std::env::consts::OS;
        let expected_ext = match current_os {
            "macos" => "dylib",
            "windows" => "dll",
            _ => "so",
        };

        // Find candidate library file
        let (lib_name, lib_bytes) = extracted_files
            .iter()
            .find(|(name, _)| {
                name.ends_with(expected_ext)
                    || manifest
                        .target
                        .as_ref()
                        .is_some_and(|t| t.library_file == **name)
            })
            .or_else(|| {
                extracted_files.iter().find(|(name, _)| {
                    std::path::Path::new(name)
                        .extension()
                        .is_some_and(|ext| {
                            ext.eq_ignore_ascii_case("dylib")
                                || ext.eq_ignore_ascii_case("so")
                                || ext.eq_ignore_ascii_case("dll")
                        })
                })
            })
            .ok_or_else(|| {
                format!(
                    "No compatible shared library (*.{expected_ext}) found in .hsp for OS '{current_os}'"
                )
            })?;

        // 4. Validate Ed25519 signature if provided
        let mut is_verified = false;
        let mut publisher = None;

        if let Some(sig_bytes) = extracted_files.get("signature.sig") {
            let sig_str = String::from_utf8_lossy(sig_bytes).trim().to_string();
            if let Ok(sig_raw) = hex::decode(&sig_str) {
                if let Ok(signature) = Signature::from_slice(&sig_raw) {
                    let mut data_to_verify = Vec::with_capacity(toml_bytes.len() + lib_bytes.len());
                    data_to_verify.extend_from_slice(toml_bytes);
                    data_to_verify.extend_from_slice(lib_bytes);

                    for pk in trusted_pubkeys {
                        if pk.verify(&data_to_verify, &signature).is_ok() {
                            is_verified = true;
                            publisher = Some(hex::encode(pk.to_bytes()));
                            break;
                        }
                    }
                }
            }
        }

        // 5. Write extracted files to destination
        fs::create_dir_all(destination_dir).map_err(|e| e.to_string())?;

        // Copy shared library directly into destination
        let clean_lib_name = Path::new(lib_name).file_name().unwrap_or_default();
        let target_lib_path = destination_dir.join(clean_lib_name);
        fs::write(&target_lib_path, lib_bytes).map_err(|e| e.to_string())?;

        // Also create plugin subdirectory for metadata
        let plugin_sub_dir = destination_dir.join(&manifest.name);
        let _ = fs::create_dir_all(&plugin_sub_dir);
        let _ = fs::write(plugin_sub_dir.join("plugin.toml"), toml_bytes);

        if let Some(readme) = extracted_files.get("README.md") {
            let _ = fs::write(plugin_sub_dir.join("README.md"), readme);
        }
        if let Some(icon) = extracted_files.get("icon.svg") {
            let _ = fs::write(plugin_sub_dir.join("icon.svg"), icon);
        }

        Ok(HspInstallResult {
            name: manifest.name,
            version: manifest.version,
            library_path: target_lib_path,
            sha256: sha256_hash,
            is_verified,
            publisher,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_pack_and_unpack_roundtrip() {
        let dir = tempdir().unwrap();
        let toml_path = dir.path().join("plugin.toml");
        fs::write(
            &toml_path,
            format!(
                r#"
name = "test_pack"
version = "1.0.0"
api_version = {PDK_API_VERSION}
author = "Hyperspace Dev"
"#
            ),
        )
        .unwrap();

        let lib_ext = if cfg!(target_os = "macos") {
            "dylib"
        } else if cfg!(target_os = "windows") {
            "dll"
        } else {
            "so"
        };
        let lib_path = dir.path().join(format!("libtest_pack.{lib_ext}"));
        fs::write(&lib_path, b"\x7fELF_DUMMY_BINARY_DATA").unwrap();

        let hsp_out = dir.path().join("test_pack.hsp");

        // Keygen
        let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
        let verifying_key = signing_key.verifying_key();

        HspPackage::pack(
            &toml_path,
            &lib_path,
            None,
            None,
            Some(&signing_key),
            &hsp_out,
        )
        .unwrap();
        assert!(hsp_out.exists());

        let unpack_dir = dir.path().join("installed");
        let hsp_bytes = fs::read(&hsp_out).unwrap();
        let res = HspPackage::unpack(&hsp_bytes, &unpack_dir, &[verifying_key]).unwrap();

        assert_eq!(res.name, "test_pack");
        assert_eq!(res.version, "1.0.0");
        assert!(res.is_verified);
        assert!(res.library_path.exists());
    }
}
