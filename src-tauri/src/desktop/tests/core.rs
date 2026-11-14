#[path = "../../../src/contracts/mod.rs"]
#[allow(dead_code)]
mod contracts;
#[path = "../settings.rs"]
mod settings;
#[path = "../logging.rs"]
#[allow(dead_code)]
mod logging;
#[path = "../launch.rs"]
mod launch;
#[path = "../updates.rs"]
mod updates;
#[cfg(windows)]
#[path = "../associations.rs"]
#[allow(dead_code)]
mod associations;

use contracts::{Locale, ThemeMode};
use settings::{Settings, SettingsStore};
use std::{fs, path::PathBuf};

fn temp(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("picglance-desktop-{}-{name}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn portable_settings_are_isolated_from_installed_settings() {
    let root = temp("settings");
    let installed = root.join("installed");
    let portable = root.join("portable");
    fs::create_dir_all(&installed).unwrap();
    fs::create_dir_all(&portable).unwrap();
    fs::write(portable.join("portable.marker"), "").unwrap();
    let config = root.join("config");
    let regular = SettingsStore::new(&installed, config.clone());
    let portable_store = SettingsStore::new(&portable, config);
    assert!(!regular.portable);
    assert!(portable_store.portable);
    regular.save(&Settings { theme: ThemeMode::Light, ..Settings::default() }).unwrap();
    portable_store.save(&Settings { theme: ThemeMode::System, locale: Locale::English, ..Settings::default() }).unwrap();
    assert_eq!(regular.load().unwrap().theme, ThemeMode::Light);
    assert_eq!(portable_store.load().unwrap().theme, ThemeMode::System);
    assert_eq!(portable_store.load().unwrap().locale, Locale::English);
    assert!(portable.join("settings.ini").is_file());
    assert!(!installed.join("settings.ini").exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn portable_qsettings_ini_accepts_legacy_locale_and_preserves_valid_fields() {
    let dir = temp("old-ini");
    fs::write(dir.join("portable.marker"), "").unwrap();
    fs::write(dir.join("settings.ini"),
        "[general]\nlanguage=zh_CN\ncheckUpdates=false\n[appearance]\ntheme=light\n[editing]\nbrushThickness=1000\n").unwrap();
    let settings = SettingsStore::new(&dir, dir.join("other")).load().unwrap();
    assert_eq!(settings.locale, Locale::SimplifiedChinese);
    assert_eq!(settings.theme, ThemeMode::Light);
    assert!(!settings.check_updates);
    assert_eq!(settings.brush_thickness, 5);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn forwarded_path_uses_sender_directory_and_preserves_unicode() {
    let root = temp("launch");
    let name = "测试 图像.jpg";
    let file = root.join(name);
    fs::write(&file, "").unwrap();
    let args = ["picglance.exe", "--", name].map(String::from);
    assert_eq!(launch::paths(&args, &root), [file]);
    assert!(launch::paths(&["app".into(), "--first-image-diagnostic".into(), name.into()], &root).is_empty());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn logs_rotate_and_fall_back_only_for_installed_mode() {
    let root = temp("logs");
    let primary = root.join("primary");
    fs::write(&primary, "").unwrap();
    let fallback = root.join("fallback");
    let log = logging::Log::open(primary.clone(), Some(fallback.clone()), 100).unwrap();
    log.record("test", "first");
    log.record("test", &"x".repeat(75));
    assert!(fs::read_to_string(fallback.join("picglance.log.1")).unwrap().contains("first"));
    assert!(logging::Log::open(primary, None, 100).is_err());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn only_newer_stable_releases_are_offered() {
    assert_eq!(updates::newer_tag("2.9.0", "v2.10.0").as_deref(), Some("v2.10.0"));
    assert_eq!(updates::newer_tag("2.0.0", "v2.0.0"), None);
    assert_eq!(updates::newer_tag("2.0.0", "v3.0.0-beta.1"), None);
    assert_eq!(updates::newer_tag("2.0.0-rc.1", "v2.0.0").as_deref(), Some("v2.0.0"));
}
