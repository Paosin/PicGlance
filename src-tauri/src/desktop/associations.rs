use std::{io, path::Path};
use windows_sys::Win32::UI::Shell::{SHChangeNotify, SHCNE_ASSOCCHANGED, SHCNF_FLUSH, SHCNF_IDLIST};
use winreg::{enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_NONE}, RegKey, RegValue};

#[cfg(test)]
#[path = "association_tests.rs"]
mod tests;

pub const FORMATS: [&str; 10] = ["jpg", "jpeg", "png", "bmp", "gif", "webp", "svg", "tiff", "tif", "ico"];
const APP: &str = r"Software\PicGlance\Desktop";
const CAPABILITIES: &str = r"Software\PicGlance\Desktop\Capabilities";
const FILES: &str = r"Software\PicGlance\Desktop\Capabilities\FileAssociations";
const CLASSES: &str = r"Software\Classes";
const LEGACY_VERB: &str = r"Software\Classes\*\shell\PicGlance";
const DESKTOP_VERB: &str = r"Software\Classes\*\shell\PicGlance.Desktop";
const LEGACY_VISIBILITY: &str = "ProgrammaticAccessOnly";
const LEGACY_MIGRATED: &str = "LegacyVerbSuppressed";

fn current_user() -> RegKey { RegKey::predef(HKEY_CURRENT_USER) }

fn remove(result: io::Result<()>) -> io::Result<()> {
    match result {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        result => result,
    }
}

fn notify() {
    unsafe { SHChangeNotify(SHCNE_ASSOCCHANGED as i32, SHCNF_IDLIST | SHCNF_FLUSH, std::ptr::null(), std::ptr::null()) };
}

fn suppress_legacy_verb(hkcu: &RegKey) -> io::Result<()> {
    let legacy = match hkcu.open_subkey_with_flags(LEGACY_VERB, KEY_READ | KEY_WRITE) {
        Ok(key) => key,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    match legacy.get_raw_value(LEGACY_VISIBILITY) {
        Ok(_) => return Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => (),
        Err(error) => return Err(error),
    }
    // The Qt app refreshes the label and command on startup. The visibility
    // value survives those writes and leaves its ProgID and defaults intact.
    let owner = hkcu.create_subkey(APP)?.0;
    owner.set_value(LEGACY_MIGRATED, &1u32)?;
    legacy.set_value(LEGACY_VISIBILITY, &"")
}

fn restore_legacy_verb(hkcu: &RegKey) -> io::Result<()> {
    let migrated = hkcu.open_subkey(APP).and_then(|key| key.get_value::<u32, _>(LEGACY_MIGRATED));
    match migrated {
        Ok(1) => (),
        Ok(_) => return Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    }
    match hkcu.open_subkey_with_flags(LEGACY_VERB, KEY_WRITE) {
        Ok(key) => remove(key.delete_value(LEGACY_VISIBILITY)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

pub fn supported() -> io::Result<Vec<String>> {
    match current_user().open_subkey(APP) {
        Ok(key) => Ok(key.get_value::<String, _>("Supported")?.split(',').map(str::to_owned).collect()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(error),
    }
}

pub fn selected() -> io::Result<Vec<String>> {
    let hkcu = current_user();
    match hkcu.open_subkey(FILES) {
        Ok(key) => key.enum_values().map(|value| value.map(|(name, _)| name.trim_start_matches('.').to_owned())).collect(),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(error),
    }
}

fn remove_format(hkcu: &RegKey, ext: &str, remove_verb: bool) -> io::Result<()> {
    let prog_id = format!("PicGlance.Desktop.{ext}");
    let open_with = format!(r"{CLASSES}\.{ext}\OpenWithProgids");
    match hkcu.open_subkey_with_flags(&open_with, KEY_WRITE) {
        Ok(key) => remove(key.delete_value(&prog_id))?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => (),
        Err(error) => return Err(error),
    }
    remove(hkcu.delete_subkey_all(format!(r"{CLASSES}\{prog_id}")))?;
    if remove_verb {
        remove(hkcu.delete_subkey_all(format!(r"{CLASSES}\SystemFileAssociations\.{ext}\shell\PicGlance.Desktop")))?;
    }
    match hkcu.open_subkey_with_flags(FILES, KEY_WRITE) {
        Ok(key) => remove(key.delete_value(format!(".{ext}"))),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn apply(hkcu: &RegKey, exe: &Path, enabled: &[String], formats: &[String]) -> io::Result<()> {
    let exe = exe.to_str().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid executable path"))?;
    let command = format!("\"{exe}\" -- \"%1\"");
    let verb = hkcu.create_subkey(DESKTOP_VERB)?.0;
    verb.set_value("", &"Open with PicGlance")?;
    verb.set_value("Icon", &format!("\"{exe}\",0"))?;
    let applies_to = formats.iter().map(|ext| format!("System.FileExtension:=\".{ext}\""))
        .collect::<Vec<_>>().join(" OR ");
    verb.set_value("AppliesTo", &applies_to)?;
    verb.create_subkey("command")?.0.set_value("", &command)?;
    let files = if enabled.is_empty() {
        None
    } else {
        let capabilities = hkcu.create_subkey(CAPABILITIES)?.0;
        capabilities.set_value("ApplicationName", &"PicGlance")?;
        capabilities.set_value("ApplicationDescription", &"PicGlance image viewer")?;
        capabilities.set_value("ApplicationIcon", &format!("\"{exe}\",0"))?;
        Some(hkcu.create_subkey(FILES)?.0)
    };
    for ext in formats {
        let prog_id = format!("PicGlance.Desktop.{ext}");
        remove(hkcu.delete_subkey_all(format!(r"{CLASSES}\SystemFileAssociations\.{ext}\shell\PicGlance.Desktop")))?;
        if !enabled.contains(ext) {
            remove_format(hkcu, ext, false)?;
            continue;
        }
        let handler = hkcu.create_subkey(format!(r"{CLASSES}\{prog_id}"))?.0;
        handler.set_value("", &format!("PicGlance .{ext} image"))?;
        handler.create_subkey("DefaultIcon")?.0.set_value("", &format!("\"{exe}\",0"))?;
        handler.create_subkey(r"shell\open\command")?.0.set_value("", &command)?;
        hkcu.create_subkey(format!(r"{CLASSES}\.{ext}\OpenWithProgids"))?.0
            .set_raw_value(&prog_id, &RegValue { bytes: (&[][..]).into(), vtype: REG_NONE })?;
        if let Some(files) = &files {
            files.set_value(format!(".{ext}"), &prog_id)?;
        }
    }
    if enabled.is_empty() {
        remove_default_apps_entry(hkcu)?;
        remove(hkcu.delete_subkey_all(CAPABILITIES))?;
    } else {
        hkcu.create_subkey(r"Software\RegisteredApplications")?.0.set_value("PicGlance", &CAPABILITIES)?;
    }
    Ok(())
}

fn valid_ext(ext: &str) -> bool {
    !ext.is_empty() && ext.len() <= 16 && ext.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
}

fn formats() -> Vec<String> { FORMATS.map(str::to_owned).into() }

pub fn install(exe: &Path, enabled: &[String]) -> io::Result<()> {
    let formats = formats();
    validate(enabled, &formats)?;
    let hkcu = current_user();
    let previous = supported()?;
    // Record the union first so uninstall can clean partial registrations after an I/O failure.
    let mut touched = formats.clone();
    touched.extend(previous.iter().filter(|ext| valid_ext(ext) && !formats.contains(ext)).cloned());
    let key = hkcu.create_subkey(APP)?.0;
    key.set_value("Supported", &touched.join(","))?;
    for ext in &touched[formats.len()..] { remove_format(&hkcu, ext, true)?; }
    apply(&hkcu, exe, enabled, &formats)?;
    suppress_legacy_verb(&hkcu)?;
    key.set_value("Supported", &formats.join(","))?;
    notify();
    Ok(())
}

fn validate(enabled: &[String], formats: &[String]) -> io::Result<()> {
    if enabled.iter().any(|ext| !formats.contains(ext)) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "unsupported association"));
    }
    Ok(())
}

pub fn configure(exe: &Path, enabled: &[String]) -> io::Result<()> {
    install(exe, enabled)
}

fn remove_default_apps_entry(hkcu: &RegKey) -> io::Result<()> {
    match hkcu.open_subkey_with_flags(r"Software\RegisteredApplications", KEY_READ | KEY_WRITE) {
        Ok(key) if key.get_value::<String, _>("PicGlance").ok().as_deref() == Some(CAPABILITIES) =>
            remove(key.delete_value("PicGlance")),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

pub fn uninstall() -> io::Result<()> {
    let hkcu = current_user();
    remove(hkcu.delete_subkey_all(DESKTOP_VERB))?;
    for ext in supported()?.iter().filter(|ext| valid_ext(ext)) {
        remove_format(&hkcu, ext, true)?;
    }
    remove_default_apps_entry(&hkcu)?;
    restore_legacy_verb(&hkcu)?;
    remove(hkcu.delete_subkey_all(APP))?;
    notify();
    Ok(())
}
