use serde::Serialize;
use winreg::enums::*;
use winreg::RegKey;

#[derive(Serialize, Debug, Clone)]
pub struct SoftwareInfo {
    pub name: String,
    pub version: Option<String>,
    pub publisher: Option<String>,
}

pub fn collect_installed_software() -> Vec<SoftwareInfo> {
    let mut apps = Vec::new();

    let paths = [
        (HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"),
        (HKEY_LOCAL_MACHINE, r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall"),
        (HKEY_CURRENT_USER, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"),
    ];

    for (root, path) in paths {
        if let Ok(uninstall_key) = RegKey::predef(root).open_subkey(path) {
            for subkey_name in uninstall_key.enum_keys().filter_map(Result::ok) {
                if let Ok(app_key) = uninstall_key.open_subkey(&subkey_name) {
                    let name: String = app_key.get_value("DisplayName").unwrap_or_default();
                    if name.is_empty() { continue; }
                    
                    let version: Option<String> = app_key.get_value("DisplayVersion").ok();
                    let publisher: Option<String> = app_key.get_value("Publisher").ok();
                    
                    apps.push(SoftwareInfo { name, version, publisher });
                }
            }
        }
    }
    
    apps.sort_by(|a, b| a.name.cmp(&b.name));
    apps.dedup_by(|a, b| a.name == b.name);
    
    apps
}