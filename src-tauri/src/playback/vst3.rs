use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Vst3BundleErrorKind {
    InvalidBundle,
    MissingBundleBinary,
    IoError,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vst3BundleError {
    pub kind: Vst3BundleErrorKind,
    pub message: String,
}

impl Vst3BundleError {
    fn invalid_bundle(message: String) -> Self {
        Self {
            kind: Vst3BundleErrorKind::InvalidBundle,
            message,
        }
    }

    fn missing_bundle_binary(message: String) -> Self {
        Self {
            kind: Vst3BundleErrorKind::MissingBundleBinary,
            message,
        }
    }

    fn io_error(message: String) -> Self {
        Self {
            kind: Vst3BundleErrorKind::IoError,
            message,
        }
    }
}

pub const MACOS_VST3_SEARCH_ROOTS: [&str; 2] = [
    "~/Library/Audio/Plug-Ins/VST3",
    "/Library/Audio/Plug-Ins/VST3",
];

pub fn default_vst3_search_roots() -> Vec<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        MACOS_VST3_SEARCH_ROOTS
            .iter()
            .filter_map(|root| expand_home_dir(root))
            .filter(|root| root.is_dir())
            .collect()
    }

    #[cfg(not(target_os = "macos"))]
    {
        Vec::new()
    }
}

pub fn expand_home_dir(path: &str) -> Option<PathBuf> {
    if path == "~" {
        return std::env::var_os("HOME").map(PathBuf::from);
    }

    if let Some(stripped) = path.strip_prefix("~/") {
        return std::env::var_os("HOME").map(|home| PathBuf::from(home).join(stripped));
    }

    Some(PathBuf::from(path))
}

pub fn validate_vst3_bundle(path: &Path) -> Result<(), Vst3BundleError> {
    if path.extension().and_then(|extension| extension.to_str()) != Some("vst3") {
        return Err(Vst3BundleError::invalid_bundle(format!(
            "Expected a .vst3 bundle path, got '{}'.",
            path.display()
        )));
    }

    if !path.exists() {
        return Err(Vst3BundleError::invalid_bundle(format!(
            "VST3 target does not exist: '{}'.",
            path.display()
        )));
    }

    if !path.is_dir() {
        return Err(Vst3BundleError::invalid_bundle(format!(
            "Expected a VST3 bundle directory, got '{}'.",
            path.display()
        )));
    }

    Ok(())
}

pub fn vst3_bundle_binary_path(path: &Path) -> Result<PathBuf, Vst3BundleError> {
    let macos_dir = path.join("Contents").join("MacOS");
    if !macos_dir.is_dir() {
        return Err(Vst3BundleError::missing_bundle_binary(format!(
            "Expected a Contents/MacOS directory inside '{}'.",
            path.display()
        )));
    }

    if let Some(bundle_name) = path.file_stem().and_then(|value| value.to_str()) {
        let preferred = macos_dir.join(bundle_name);
        if preferred.is_file() {
            return Ok(preferred);
        }
    }

    let mut binaries = Vec::new();
    let read_dir = std::fs::read_dir(&macos_dir).map_err(|error| {
        Vst3BundleError::io_error(format!(
            "Failed to read VST3 bundle binary directory '{}': {error}",
            macos_dir.display()
        ))
    })?;

    for entry in read_dir {
        let entry = entry.map_err(|error| {
            Vst3BundleError::io_error(format!(
                "Failed to inspect VST3 bundle binary entry in '{}': {error}",
                macos_dir.display()
            ))
        })?;
        let path = entry.path();

        if path.is_file() {
            binaries.push(path);
        }
    }

    binaries.sort();

    match binaries.len() {
        0 => Err(Vst3BundleError::missing_bundle_binary(format!(
            "Did not find a bundle executable inside '{}'.",
            macos_dir.display()
        ))),
        _ => Ok(binaries.remove(0)),
    }
}

pub fn list_vst3_candidates_in_root(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut entries = Vec::new();
    let mut roots_to_visit = vec![root.to_path_buf()];
    let mut visited_roots = BTreeSet::new();

    while let Some(current_root) = roots_to_visit.pop() {
        let visited_key = std::fs::canonicalize(&current_root).unwrap_or(current_root.clone());
        if !visited_roots.insert(visited_key) {
            continue;
        }

        let read_dir = std::fs::read_dir(&current_root).map_err(|error| {
            format!(
                "Failed to read VST3 discovery root '{}': {error}",
                current_root.display()
            )
        })?;

        for entry in read_dir {
            let entry = entry.map_err(|error| {
                format!(
                    "Failed to inspect VST3 discovery root entry in '{}': {error}",
                    current_root.display()
                )
            })?;
            let path = entry.path();

            if path.extension().and_then(|value| value.to_str()) == Some("vst3") {
                entries.push(path);
                continue;
            }

            if path.is_dir() {
                roots_to_visit.push(path);
            }
        }
    }

    entries.sort();
    Ok(entries)
}

pub fn discover_first_valid_vst3_in_roots(roots: &[PathBuf]) -> Option<PathBuf> {
    for root in roots {
        if let Ok(candidates) = list_vst3_candidates_in_root(root) {
            for path in candidates {
                if validate_vst3_bundle(&path).is_ok() {
                    return Some(path);
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::{
        discover_first_valid_vst3_in_roots, expand_home_dir, list_vst3_candidates_in_root,
        validate_vst3_bundle, vst3_bundle_binary_path, Vst3BundleErrorKind,
    };
    #[cfg(unix)]
    use std::os::unix::fs::symlink;
    use std::path::Path;

    #[test]
    fn validate_vst3_bundle_rejects_non_vst3_paths() {
        let error = validate_vst3_bundle(Path::new("/tmp/test-plugin.component")).unwrap_err();
        assert_eq!(error.kind, Vst3BundleErrorKind::InvalidBundle);
        assert!(error.message.contains(".vst3 bundle path"));
    }

    #[test]
    fn validate_vst3_bundle_rejects_missing_paths() {
        let error = validate_vst3_bundle(Path::new("/tmp/missing-test-plugin.vst3")).unwrap_err();
        assert_eq!(error.kind, Vst3BundleErrorKind::InvalidBundle);
        assert!(error.message.contains("does not exist"));
    }

    #[test]
    fn validate_vst3_bundle_rejects_non_directory_vst3_paths() {
        let path = std::env::temp_dir().join("vst3-helper-file.vst3");
        std::fs::write(&path, b"not-a-bundle").unwrap();

        let error = validate_vst3_bundle(&path).unwrap_err();
        assert_eq!(error.kind, Vst3BundleErrorKind::InvalidBundle);
        assert!(error.message.contains("bundle directory"));

        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn discover_first_valid_vst3_in_roots_prefers_sorted_valid_bundle() {
        let root = std::env::temp_dir().join(format!("vst3-helper-search-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        std::fs::write(root.join("Ignore Me.vst3"), b"not-a-bundle").unwrap();

        let later_bundle = root.join("Zoo Instrument.vst3");
        let earlier_bundle = root.join("Alpha Instrument.vst3");
        std::fs::create_dir_all(&later_bundle).unwrap();
        std::fs::create_dir_all(&earlier_bundle).unwrap();

        let discovered = discover_first_valid_vst3_in_roots(std::slice::from_ref(&root)).unwrap();
        assert_eq!(discovered, earlier_bundle);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn list_vst3_candidates_in_root_ignores_non_vst3_entries() {
        let root =
            std::env::temp_dir().join(format!("vst3-helper-candidates-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        std::fs::create_dir_all(root.join("Alpha.vst3")).unwrap();
        std::fs::create_dir_all(root.join("Beta.component")).unwrap();

        let candidates = list_vst3_candidates_in_root(&root).unwrap();
        assert_eq!(candidates, vec![root.join("Alpha.vst3")]);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn list_vst3_candidates_in_root_recurses_into_nested_folders() {
        let root = std::env::temp_dir().join(format!("vst3-helper-nested-{}", std::process::id()));
        let nested = root.join("Steinberg").join("VST3");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&nested).unwrap();

        let nested_bundle = nested.join("HALion Sonic.vst3");
        let top_level_bundle = root.join("Alpha.vst3");
        std::fs::create_dir_all(&nested_bundle).unwrap();
        std::fs::create_dir_all(&top_level_bundle).unwrap();

        let candidates = list_vst3_candidates_in_root(&root).unwrap();
        assert_eq!(candidates, vec![top_level_bundle, nested_bundle]);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn list_vst3_candidates_in_root_does_not_descend_into_bundle_contents() {
        let root = std::env::temp_dir().join(format!(
            "vst3-helper-bundle-recursion-{}",
            std::process::id()
        ));
        let bundle = root.join("NestedHost.vst3");
        let fake_nested_bundle = bundle.join("Contents").join("MacOS").join("Ghost.vst3");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&fake_nested_bundle).unwrap();

        let candidates = list_vst3_candidates_in_root(&root).unwrap();
        assert_eq!(candidates, vec![bundle]);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[cfg(unix)]
    #[test]
    fn list_vst3_candidates_in_root_skips_symlink_cycles() {
        let root =
            std::env::temp_dir().join(format!("vst3-helper-symlink-cycle-{}", std::process::id()));
        let loop_dir = root.join("loop");
        let bundle = root.join("Alpha.vst3");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&bundle).unwrap();
        symlink(&root, &loop_dir).unwrap();

        let candidates = list_vst3_candidates_in_root(&root).unwrap();

        assert_eq!(candidates, vec![bundle]);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn expand_home_dir_replaces_tilde_prefix() {
        let previous = std::env::var_os("HOME");
        std::env::set_var("HOME", "/tmp/vst3-helper-home");

        let expanded = expand_home_dir("~/Library/Audio/Plug-Ins/VST3").unwrap();
        assert_eq!(
            expanded,
            Path::new("/tmp/vst3-helper-home/Library/Audio/Plug-Ins/VST3")
        );

        match previous {
            Some(value) => std::env::set_var("HOME", value),
            None => std::env::remove_var("HOME"),
        }
    }

    #[test]
    fn vst3_bundle_binary_path_prefers_named_executable() {
        let root = std::env::temp_dir().join(format!("vst3-helper-bundle-{}", std::process::id()));
        let bundle = root.join("My Instrument.vst3");
        let macos = bundle.join("Contents").join("MacOS");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&macos).unwrap();
        std::fs::write(macos.join("OtherBinary"), b"x").unwrap();
        let expected = macos.join("My Instrument");
        std::fs::write(&expected, b"x").unwrap();

        let resolved = vst3_bundle_binary_path(&bundle).unwrap();
        assert_eq!(resolved, expected);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn list_vst3_candidates_in_root_reports_read_errors() {
        let root =
            std::env::temp_dir().join(format!("vst3-helper-file-root-{}", std::process::id()));
        let _ = std::fs::remove_file(&root);
        std::fs::write(&root, b"not-a-directory").unwrap();

        let error = list_vst3_candidates_in_root(&root).unwrap_err();
        assert!(error.contains("Failed to read VST3 discovery root"));

        let _ = std::fs::remove_file(&root);
    }
}
