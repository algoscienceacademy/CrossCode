use std::{path::PathBuf, process::Command};

use serde::{Deserialize, Serialize};

use crate::builder::swift::SwiftBin;

pub const FORMAT_VERSION: u32 = 1;

pub struct BuildSettings {
    pub debug: bool,
}

// TODO: Min ios version, etc.
pub struct ProjectConfig {
    pub product: String,
    pub version_num: String,
    pub version_string: String,
    pub bundle_id: String,
    pub project_path: PathBuf,
}

#[derive(Deserialize, Serialize)]
struct TomlConfig {
    pub format_version: u32,
    pub project: ProjectTomlConfig,
}

#[derive(Deserialize, Serialize)]
struct ProjectTomlConfig {
    pub version_num: String,
    pub version_string: String,
    pub bundle_id: String,
}

// TODO: Check platforms
#[derive(Deserialize)]
struct SwiftPackageDump {
    name: String,
    targets: Vec<SwiftPackageTarget>,
}

// TODO: Resources
#[derive(Deserialize)]
struct SwiftPackageTarget {
    name: String,
}

impl ProjectConfig {
    pub fn load(project_path: PathBuf, toolchain_path: &str) -> Result<Self, String> {
        let toml_config = TomlConfig::load_or_default(project_path.clone())?;
        let swift = SwiftBin::new(toolchain_path)?;
        let raw_package = swift.command()
            .arg("package")
            .arg("dump-package")
            .current_dir(&project_path)
            .output()
            .map_err(|e| format!("Failed to execute swift command: {}", e))?;
        if !raw_package.status.success() {
            return Err(format!(
                "Failed to dump package: {}",
                String::from_utf8_lossy(&raw_package.stderr)
            ));
        }
        let package: SwiftPackageDump = serde_json::from_slice(&raw_package.stdout)
            .map_err(|e| format!("Failed to parse package dump: {}", e))?;

        Ok(ProjectConfig {
            product: package.name,
            version_num: toml_config.project.version_num,
            version_string: toml_config.project.version_string,
            bundle_id: toml_config.project.bundle_id,
            project_path,
        })
    }
}

impl TomlConfig {
    pub fn default(bundle_id: &str) -> Self {
        TomlConfig {
            format_version: FORMAT_VERSION,
            project: ProjectTomlConfig {
                version_num: "1".to_string(),
                version_string: "1.0.0".to_string(),
                bundle_id: bundle_id.to_string(),
            },
        }
    }

    pub fn load_or_default(project_path: PathBuf) -> Result<Self, String> {
        if project_path.exists() {
            Self::load(project_path)
        } else {
            let config = Self::default("com.example.myapp");
            config.save(project_path)?;
            Ok(config)
        }
    }

    fn load(project_path: PathBuf) -> Result<Self, String> {
        let content =
            std::fs::read_to_string(project_path.join("ycode.toml")).map_err(|e| e.to_string())?;
        let config: TomlConfig = toml::from_str(&content).map_err(|e| e.to_string())?;
        if config.format_version != FORMAT_VERSION {
            return Err(format!(
                "Unsupported format version: {}, expected: {}",
                config.format_version, FORMAT_VERSION
            ));
        }
        Ok(config)
    }

    pub fn save(&self, project_path: PathBuf) -> Result<(), String> {
        let content = toml::to_string(self).map_err(|e| e.to_string())?;
        std::fs::write(project_path.join("ycode.toml"), content).map_err(|e| e.to_string())?;
        Ok(())
    }
}
