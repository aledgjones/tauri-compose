use std::path::Path;

const UNSUPPORTED_EDITOR_HOST_BUNDLE_NAMES: &[&str] = &[
    "HALion Sonic",
    "HALion Sonic SE",
    "Groove Agent SE",
    "Kontakt 7",
    "MT-PowerDrumKit",
];
const UNSUPPORTED_EDITOR_HOST_MESSAGE: &str = "Unsupported plugin target for the current macOS native editor host POC: this bundle is on the current unsupported list for the bounded seam. Current evidence shows HALion Sonic and Groove Agent SE crash during controller initialization, HALion Sonic SE returns a null plugin factory, Kontakt 7 crashes during component createInstance, and MT-PowerDrumKit currently requires an MVP-host teardown exception we do not want to carry in the bounded seam. Other discovered bundles remain eligible unless they demonstrate host-seam issues and are added to this unsupported list.";

fn bundle_stem_matches_any_case_insensitive(path: &Path, names: &[&str]) -> bool {
    let Some(bundle_stem) = path.file_stem().and_then(|value| value.to_str()) else {
        return false;
    };

    names
        .iter()
        .any(|name| bundle_stem.eq_ignore_ascii_case(name))
}

pub fn bounded_editor_host_unsupported_reason(plugin_path: &Path) -> Option<String> {
    if bundle_stem_matches_any_case_insensitive(plugin_path, UNSUPPORTED_EDITOR_HOST_BUNDLE_NAMES) {
        return Some(String::from(UNSUPPORTED_EDITOR_HOST_MESSAGE));
    }

    None
}
#[cfg(test)]
mod tests {
    use super::bounded_editor_host_unsupported_reason;
    use std::path::Path;

    #[test]
    fn rejects_known_unsupported_vendor_example() {
        let path = Path::new("/Library/Audio/Plug-Ins/VST3/Steinberg/HALion Sonic.vst3");

        let reason = bounded_editor_host_unsupported_reason(path).unwrap();

        assert!(reason.contains("current unsupported list"));
        assert!(reason.contains("HALion Sonic"));
    }

    #[test]
    fn allows_unknown_bundle_even_without_a_vendor_folder() {
        let path = Path::new("/tmp/Some Future Plugin.vst3");

        assert_eq!(bounded_editor_host_unsupported_reason(path), None);
    }

    #[test]
    fn rejects_kontakt_7_bundle_name() {
        let path = Path::new("/Library/Audio/Plug-Ins/VST3/Kontakt 7.vst3");

        let reason = bounded_editor_host_unsupported_reason(path).unwrap();

        assert!(reason.contains("Kontakt 7"));
        assert!(reason.contains("component createInstance"));
    }

    #[test]
    fn rejects_mt_power_drumkit_bundle_name() {
        let path = Path::new("/Library/Audio/Plug-Ins/VST3/MT-PowerDrumKit.vst3");

        let reason = bounded_editor_host_unsupported_reason(path).unwrap();

        assert!(reason.contains("MT-PowerDrumKit"));
        assert!(reason.contains("MVP-host teardown exception"));
    }

    #[test]
    fn allows_non_unsupported_control_case() {
        let path = Path::new("/Library/Audio/Plug-Ins/VST3/BBC Symphony Orchestra.vst3");

        assert_eq!(bounded_editor_host_unsupported_reason(path), None);
    }
}
