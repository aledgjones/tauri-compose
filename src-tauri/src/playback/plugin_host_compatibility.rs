use std::path::Path;

struct UnsupportedEditorHostBundle {
    name: &'static str,
    reason: &'static str,
}

const UNSUPPORTED_EDITOR_HOST_BUNDLES: &[UnsupportedEditorHostBundle] = &[
    UnsupportedEditorHostBundle {
        name: "HALion Sonic SE",
        reason: "it returns a null plugin factory during factory creation on the bounded seam.",
    },
    UnsupportedEditorHostBundle {
        name: "Groove Agent SE",
        reason: "it crashes during controller initialization.",
    },
    UnsupportedEditorHostBundle {
        name: "Kontakt 7",
        reason: "it crashes during component createInstance.",
    },
    UnsupportedEditorHostBundle {
        name: "MT-PowerDrumKit",
        reason: "it currently requires an MVP-host teardown exception we do not want to carry in the bounded seam.",
    },
];

fn bundle_stem(path: &Path) -> Option<&str> {
    path.file_stem().and_then(|value| value.to_str())
}

fn unsupported_editor_host_bundle(path: &Path) -> Option<&'static UnsupportedEditorHostBundle> {
    let bundle_stem = bundle_stem(path)?;

    UNSUPPORTED_EDITOR_HOST_BUNDLES
        .iter()
        .find(|bundle| bundle_stem.eq_ignore_ascii_case(bundle.name))
}

pub fn bounded_editor_host_unsupported_reason(plugin_path: &Path) -> Option<String> {
    unsupported_editor_host_bundle(plugin_path).map(|bundle| {
        format!(
            "Unsupported plugin target for the current macOS native editor host POC: {} remains on the current unsupported list for the bounded seam because {} Other discovered bundles remain eligible unless they demonstrate host-seam issues and are added to this unsupported list.",
            bundle.name, bundle.reason
        )
    })
}
#[cfg(test)]
mod tests {
    use super::bounded_editor_host_unsupported_reason;
    use std::path::Path;

    #[test]
    fn rejects_known_unsupported_vendor_example() {
        let path = Path::new("/Library/Audio/Plug-Ins/VST3/Steinberg/Groove Agent SE.vst3");

        let reason = bounded_editor_host_unsupported_reason(path).unwrap();

        assert!(reason.contains("current unsupported list"));
        assert!(reason.contains("Groove Agent SE"));
        assert!(reason.contains("controller initialization"));
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
    fn allows_halion_sonic_bundle_name() {
        let path = Path::new("/Library/Audio/Plug-Ins/VST3/Steinberg/HALion Sonic.vst3");

        assert_eq!(bounded_editor_host_unsupported_reason(path), None);
    }

    #[test]
    fn rejects_halion_sonic_se_bundle_name() {
        let path = Path::new("/Library/Audio/Plug-Ins/VST3/Steinberg/HALion Sonic SE.vst3");

        let reason = bounded_editor_host_unsupported_reason(path).unwrap();

        assert!(reason.contains("HALion Sonic SE"));
        assert!(reason.contains("null plugin factory"));
    }

    #[test]
    fn allows_non_unsupported_control_case() {
        let path = Path::new("/Library/Audio/Plug-Ins/VST3/BBC Symphony Orchestra.vst3");

        assert_eq!(bounded_editor_host_unsupported_reason(path), None);
    }
}
