use crate::contracts::{Locale, ThemeMode};
use serde::{Deserialize, Serialize};
use std::{fs, io, path::{Path, PathBuf}};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CloseBehavior { Quit, Tray, Background }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub theme: ThemeMode,
    pub locale: Locale,
    pub check_updates: bool,
    pub close_behavior: CloseBehavior,
    pub zoom_mode: String,
    pub upscale_small_images: bool,
    pub brush_color: String,
    pub brush_thickness: u16,
    pub mosaic_block_size: u16,
    pub compress_format: String,
    pub compress_target_mb: f64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: ThemeMode::Dark, locale: Locale::SimplifiedChinese, check_updates: true,
            close_behavior: CloseBehavior::Quit, zoom_mode: "fit".into(), upscale_small_images: false,
            brush_color: "#ff0000".into(), brush_thickness: 5, mosaic_block_size: 10,
            compress_format: "JPEG".into(), compress_target_mb: 1.0,
        }
    }
}

impl Settings {
    pub fn valid(&self) -> bool {
        matches!(self.zoom_mode.as_str(), "fit" | "actual")
            && self.brush_color.len() == 7 && self.brush_color.starts_with('#')
            && self.brush_color.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)
            && (1..=50).contains(&self.brush_thickness)
            && (5..=100).contains(&self.mosaic_block_size)
            && matches!(self.compress_format.as_str(), "JPEG" | "PNG" | "WebP" | "GIF")
            && (0.01..=1000.0).contains(&self.compress_target_mb)
    }
}

pub struct SettingsStore {
    path: PathBuf,
    pub portable: bool,
}

impl SettingsStore {
    pub fn new(exe_dir: &Path, installed_config: PathBuf) -> Self {
        let portable = exe_dir.join("portable.marker").is_file();
        let dir = if portable { exe_dir } else { &installed_config };
        Self { path: dir.join("settings.ini"), portable }
    }

    pub fn load(&self) -> io::Result<Settings> {
        let content = match fs::read_to_string(&self.path) {
            Ok(content) => content,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                #[cfg(windows)]
                if !self.portable {
                    return legacy_settings();
                }
                return Ok(Settings::default());
            }
            Err(error) => return Err(error),
        };
        Ok(Self::parse(&content))
    }

    fn parse(content: &str) -> Settings {
        let mut settings = Settings::default();
        let mut section = "";
        for line in content.trim_start_matches('\u{feff}').lines().map(str::trim) {
            if let Some(name) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                section = name;
                continue;
            }
            let Some((key, value)) = line.split_once('=') else { continue };
            let mut candidate = settings.clone();
            match (section, key.trim(), value.trim()) {
                ("appearance", "theme", "dark") => candidate.theme = ThemeMode::Dark,
                ("appearance", "theme", "light") => candidate.theme = ThemeMode::Light,
                ("appearance", "theme", "system") => candidate.theme = ThemeMode::System,
                ("general", "language", "en") => candidate.locale = Locale::English,
                ("general", "language", "zh-CN" | "zh_CN") => candidate.locale = Locale::SimplifiedChinese,
                ("general", "checkUpdates", value) => candidate.check_updates = value.parse().unwrap_or(settings.check_updates),
                ("general", "closeBehavior", "quit") => candidate.close_behavior = CloseBehavior::Quit,
                ("general", "closeBehavior", "tray") => candidate.close_behavior = CloseBehavior::Tray,
                ("general", "closeBehavior", "background") => candidate.close_behavior = CloseBehavior::Background,
                ("viewing", "zoomMode", value) => candidate.zoom_mode = value.into(),
                ("viewing", "upscaleSmallImages", value) => candidate.upscale_small_images = value.parse().unwrap_or(settings.upscale_small_images),
                ("editing", "brushColor", value) => candidate.brush_color = value.into(),
                ("editing", "brushThickness", value) => candidate.brush_thickness = value.parse().unwrap_or(settings.brush_thickness),
                ("editing", "mosaicBlockSize", value) => candidate.mosaic_block_size = value.parse().unwrap_or(settings.mosaic_block_size),
                ("compression", "defaultFormat", value) => candidate.compress_format = value.into(),
                ("compression", "targetMB", value) => candidate.compress_target_mb = value.parse().unwrap_or(settings.compress_target_mb),
                _ => continue,
            }
            if candidate.valid() { settings = candidate; }
        }
        settings
    }

    pub fn save(&self, s: &Settings) -> io::Result<()> {
        if !s.valid() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid desktop settings"));
        }
        let theme = match s.theme { ThemeMode::Dark => "dark", ThemeMode::Light => "light", ThemeMode::System => "system" };
        let locale = match s.locale { Locale::SimplifiedChinese => "zh-CN", Locale::English => "en" };
        let close = match s.close_behavior { CloseBehavior::Quit => "quit", CloseBehavior::Tray => "tray", CloseBehavior::Background => "background" };
        let content = format!(concat!(
            "[general]\nlanguage={}\ncheckUpdates={}\ncloseBehavior={}\n\n",
            "[appearance]\ntheme={}\n\n[viewing]\nzoomMode={}\nupscaleSmallImages={}\n\n",
            "[editing]\nbrushColor={}\nbrushThickness={}\nmosaicBlockSize={}\n\n",
            "[compression]\ndefaultFormat={}\ntargetMB={}\n"),
            locale, s.check_updates, close, theme, s.zoom_mode, s.upscale_small_images,
            s.brush_color, s.brush_thickness, s.mosaic_block_size, s.compress_format, s.compress_target_mb);
        let parent = self.path.parent().expect("settings directory");
        fs::create_dir_all(parent)?;
        let staging = self.path.with_extension(format!("{}.tmp", std::process::id()));
        let result = fs::write(&staging, content).and_then(|()| fs::rename(&staging, &self.path));
        if result.is_err() { let _ = fs::remove_file(staging); }
        result
    }
}

#[cfg(windows)]
fn legacy_settings() -> io::Result<Settings> {
    use winreg::{enums::HKEY_CURRENT_USER, RegKey};
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let root = match hkcu.open_subkey(r"Software\PicGlance\PicGlance") {
        Ok(root) => root,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Settings::default()),
        Err(error) => return Err(error),
    };
    let mut content = String::new();
    for (section, names) in [
        ("general", &["language", "checkUpdates", "closeBehavior"][..]),
        ("appearance", &["theme"][..]),
        ("viewing", &["zoomMode", "upscaleSmallImages"][..]),
        ("editing", &["brushColor", "brushThickness", "mosaicBlockSize"][..]),
        ("compression", &["defaultFormat", "targetMB"][..]),
    ] {
        content.push_str(&format!("[{section}]\n"));
        let Ok(key) = root.open_subkey(section) else { continue };
        for name in names {
            if let Ok(value) = key.get_value::<String, _>(name) {
                content.push_str(&format!("{name}={value}\n"));
            } else if let Ok(value) = key.get_value::<u32, _>(name) {
                let value = match (*name, value) {
                    ("checkUpdates" | "upscaleSmallImages", 0) => "false".to_owned(),
                    ("checkUpdates" | "upscaleSmallImages", 1) => "true".to_owned(),
                    _ => value.to_string(),
                };
                content.push_str(&format!("{name}={value}\n"));
            }
        }
    }
    Ok(SettingsStore::parse(&content))
}
