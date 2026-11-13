use super::*;

#[test]
fn context_menu_only_removes_our_old_handler_and_preserves_the_default() {
    let hkcu = current_user();
    let path = format!(r"Software\PicGlanceDesktopTest\{}-context-menu", std::process::id());
    let root = hkcu.create_subkey(&path).unwrap().0;
    let result = (|| -> io::Result<()> {
        let formats = ["png".to_owned()];
        let exe = Path::new(r"D:\Applications\PicGlance\PicGlance.exe");
        let extension = root.create_subkey(r"Software\Classes\.png")?.0;
        extension.set_value("", &"Other.Image")?;
        apply(&root, exe, &formats, &formats)?;
        assert!(root.open_subkey(r"Software\Classes\PicGlance.Desktop.png").is_ok());

        apply(&root, exe, &[], &formats)?;
        assert_eq!(extension.get_value::<String, _>("")?, "Other.Image");
        assert!(root.open_subkey(r"Software\Classes\PicGlance.Desktop.png").is_err());
        assert!(root.open_subkey(CAPABILITIES).is_err());
        assert!(root.open_subkey(r"Software\RegisteredApplications")?.get_raw_value("PicGlance").is_err());
        assert_eq!(root.open_subkey(format!(r"{DESKTOP_VERB}\command"))?.get_value::<String, _>("")?,
            r#""D:\Applications\PicGlance\PicGlance.exe" -- "%1""#);
        Ok(())
    })();
    hkcu.delete_subkey_all(path).unwrap();
    result.unwrap();
}

#[test]
fn legacy_verb_stays_hidden_when_old_app_rewrites_it_and_is_restored_on_uninstall() {
    let hkcu = current_user();
    let path = format!(r"Software\PicGlanceDesktopTest\{}", std::process::id());
    let root = hkcu.create_subkey(&path).unwrap().0;
    let result = (|| -> io::Result<()> {
        let legacy = root.create_subkey(LEGACY_VERB)?.0;
        legacy.set_value("", &"Open old PicGlance")?;
        let command = legacy.create_subkey("command")?.0;
        command.set_value("", &r#""D:\Program Files\PicGlance\PicGlance.exe" "%1""#)?;
        let default = root.create_subkey(r"Software\Classes\.jpg")?.0;
        default.set_value("", &"PicGlance.Image")?;

        suppress_legacy_verb(&root)?;
        legacy.set_value("", &"Old app rewrote its menu")?;
        assert!(legacy.get_raw_value("ProgrammaticAccessOnly").is_ok());
        assert_eq!(default.get_value::<String, _>("")?, "PicGlance.Image");
        restore_legacy_verb(&root)?;
        assert!(legacy.get_raw_value("ProgrammaticAccessOnly").is_err());
        assert_eq!(legacy.get_value::<String, _>("")?, "Old app rewrote its menu");
        assert_eq!(command.get_value::<String, _>("")?, r#""D:\Program Files\PicGlance\PicGlance.exe" "%1""#);
        Ok(())
    })();
    hkcu.delete_subkey_all(path).unwrap();
    result.unwrap();
}
