use plist::{Dictionary, Value};
use std::{
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct InvalidBundleError {
    message: String,
}

impl fmt::Display for InvalidBundleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Cannot parse the application bundle! {}", self.message)
    }
}

impl Error for InvalidBundleError {}

pub struct Bundle {
    pub app_info: Dictionary,
    pub bundle_dir: PathBuf,

    app_extensions: Vec<Bundle>,
    frameworks: Vec<Bundle>,
    libraries: Vec<String>,
}

impl Bundle {
    pub fn new(bundle_dir: PathBuf) -> Result<Self, InvalidBundleError> {
        let mut bundle_path = bundle_dir;
        // Remove trailing slash/backslash
        if let Some(path_str) = bundle_path.to_str() {
            if path_str.ends_with('/') || path_str.ends_with('\\') {
                bundle_path = PathBuf::from(&path_str[..path_str.len() - 1]);
            }
        }

        let info_plist_path = bundle_path.join("Info.plist");
        assert_bundle(
            info_plist_path.exists(),
            &format!("No Info.plist here: {}", info_plist_path.display()),
        )?;

        let plist_data = fs::read(&info_plist_path).map_err(|e| InvalidBundleError {
            message: format!("Failed to read Info.plist: {}", e),
        })?;

        let app_info = plist::from_bytes(&plist_data).map_err(|e| InvalidBundleError {
            message: format!("Failed to parse Info.plist: {}", e),
        })?;

        // Load app extensions from PlugIns directory
        let plug_ins_dir = bundle_path.join("PlugIns");
        let app_extensions = if plug_ins_dir.exists() {
            fs::read_dir(&plug_ins_dir)
                .map_err(|e| InvalidBundleError {
                    message: format!("Failed to read PlugIns directory: {}", e),
                })?
                .filter_map(|entry| entry.ok())
                .filter(|entry| {
                    entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
                        && entry.path().join("Info.plist").exists()
                })
                .filter_map(|entry| Bundle::new(entry.path()).ok())
                .collect()
        } else {
            Vec::new()
        };

        // Load frameworks from Frameworks directory
        let frameworks_dir = bundle_path.join("Frameworks");
        let frameworks = if frameworks_dir.exists() {
            fs::read_dir(&frameworks_dir)
                .map_err(|e| InvalidBundleError {
                    message: format!("Failed to read Frameworks directory: {}", e),
                })?
                .filter_map(|entry| entry.ok())
                .filter(|entry| {
                    entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
                        && entry.path().join("Info.plist").exists()
                })
                .filter_map(|entry| Bundle::new(entry.path()).ok())
                .collect()
        } else {
            Vec::new()
        };

        // Find all .dylib files in the bundle directory (recursive)
        let libraries = find_dylibs(&bundle_path, &bundle_path)?;

        Ok(Bundle {
            app_info,
            bundle_dir: bundle_path,
            app_extensions,
            frameworks,
            libraries,
        })
    }

    pub fn set_bundle_identifier(&mut self, id: &str) {
        self.app_info.insert(
            "CFBundleIdentifier".to_string(),
            Value::String(id.to_string()),
        );
    }

    pub fn bundle_identifier(&self) -> Option<&str> {
        self.app_info
            .get("CFBundleIdentifier")
            .and_then(|v| v.as_string())
    }

    pub fn bundle_name(&self) -> Option<&str> {
        self.app_info
            .get("CFBundleName")
            .and_then(|v| v.as_string())
    }

    pub fn libraries(&self) -> &[String] {
        &self.libraries
    }

    pub fn frameworks(&self) -> &[Bundle] {
        &self.frameworks
    }

    pub fn app_extensions(&self) -> &[Bundle] {
        &self.app_extensions
    }

    pub fn app_extensions_mut(&mut self) -> &mut [Bundle] {
        &mut self.app_extensions
    }

    pub fn sub_bundles(&self) -> Vec<&Bundle> {
        self.frameworks
            .iter()
            .chain(self.app_extensions.iter())
            .collect()
    }

    pub fn write_info(&self) -> Result<(), InvalidBundleError> {
        let info_plist_path = self.bundle_dir.join("Info.plist");
        let result = plist::to_file_binary(&info_plist_path, &self.app_info);

        if result.is_err() {
            return Err(InvalidBundleError {
                message: format!("Failed to write Info.plist: {}", result.unwrap_err()),
            });
        }
        Ok(())
    }
}

fn assert_bundle(condition: bool, msg: &str) -> Result<(), InvalidBundleError> {
    if !condition {
        Err(InvalidBundleError {
            message: msg.to_string(),
        })
    } else {
        Ok(())
    }
}

fn find_dylibs(dir: &Path, bundle_root: &Path) -> Result<Vec<String>, InvalidBundleError> {
    let mut libraries = Vec::new();

    fn collect_dylibs(
        dir: &Path,
        bundle_root: &Path,
        libraries: &mut Vec<String>,
    ) -> Result<(), InvalidBundleError> {
        let entries = fs::read_dir(dir).map_err(|e| InvalidBundleError {
            message: format!("Failed to read directory {}: {}", dir.display(), e),
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| InvalidBundleError {
                message: format!("Failed to read directory entry: {}", e),
            })?;

            let path = entry.path();
            let file_type = entry.file_type().map_err(|e| InvalidBundleError {
                message: format!("Failed to get file type: {}", e),
            })?;

            if file_type.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.ends_with(".dylib") {
                        // Get relative path from bundle root
                        if let Ok(relative_path) = path.strip_prefix(bundle_root) {
                            if let Some(relative_str) = relative_path.to_str() {
                                libraries.push(relative_str.to_string());
                            }
                        }
                    }
                }
            } else if file_type.is_dir() {
                collect_dylibs(&path, bundle_root, libraries)?;
            }
        }
        Ok(())
    }

    collect_dylibs(dir, bundle_root, &mut libraries)?;
    Ok(libraries)
}
