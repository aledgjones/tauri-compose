use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeSet, HashMap};
use std::env;
use std::fs;
use std::path::Path;
use std::process::{self, Command};

type DynError = Box<dyn std::error::Error>;

const PACKAGE_JSON_PATH: &str = "package.json";
const PACKAGE_LOCK_PATH: &str = "package-lock.json";
const CARGO_TOML_PATH: &str = "src-tauri/Cargo.toml";
const CARGO_LOCK_PATH: &str = "src-tauri/Cargo.lock";
const TAURI_CONF_PATH: &str = "src-tauri/tauri.conf.json";
const RELEASES_PATH: &str = "releases.md";
const PREP_MANIFEST_PATH: &str = ".release-prep.json";
const INTERNAL_PREP_MANIFEST_PATH: &str = ".git/solo-compose-release-prep.json";
const MIGRATIONS_DIR: &str = "src-tauri/src/migrations";
const MIGRATIONS_MOD_PATH: &str = "src-tauri/src/migrations/mod.rs";
const NEXT_MIGRATION_PATH: &str = "src-tauri/src/migrations/next.rs";
const RELEASE_NOTE_SKIP_MARKER: &str = "[skip-release-notes]";
const RESEED_COMMIT_MESSAGE: &str = "chore(release): reseed next.rs scaffold [skip-release-notes]";
const PENDING_STEPS_START: &str = "const PENDING_STEPS: &[PendingStep] = &[";
const PENDING_STEPS_END: &str = "];";
const GENERATED_MIGRATIONS_START: &str = "// BEGIN GENERATED MIGRATIONS";
const GENERATED_MIGRATIONS_END: &str = "// END GENERATED MIGRATIONS";
const SUPPORT_MODULES_START: &str = "// BEGIN SUPPORT MODULES";
const SUPPORT_MODULES_END: &str = "// END SUPPORT MODULES";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum BumpType {
    Patch,
    Minor,
    Major,
}

impl BumpType {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "patch" => Some(Self::Patch),
            "minor" => Some(Self::Minor),
            "major" => Some(Self::Major),
            _ => None,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::Patch => "patch",
            Self::Minor => "minor",
            Self::Major => "major",
        }
    }

    fn bump(&self, version: &Version) -> Version {
        let mut next = version.clone();

        match self {
            Self::Patch => next.patch += 1,
            Self::Minor => {
                next.minor += 1;
                next.patch = 0;
            }
            Self::Major => {
                next.major += 1;
                next.minor = 0;
                next.patch = 0;
            }
        }

        next.pre = semver::Prerelease::EMPTY;
        next.build = semver::BuildMetadata::EMPTY;
        next
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ReleaseCommand {
    Prepare(BumpType),
    Finalize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct PreparedFileHash {
    path: String,
    blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct ReleasePrepManifest {
    base_commit: String,
    previous_version: String,
    prepared_version: String,
    bump_type: BumpType,
    last_tag: String,
    release_tag: String,
    promoted_migration_path: Option<String>,
    pending_step_count: usize,
    next_scaffold_source: String,
    changed_paths: Vec<String>,
    managed_files: Vec<PreparedFileHash>,
    absent_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PrepareMigrationState {
    promoted_migration_path: Option<String>,
    pending_step_count: usize,
    next_scaffold_source: String,
}

#[derive(Clone)]
struct SnapshotEntry {
    path: String,
    contents: Option<Vec<u8>>,
}

struct WorkspaceSnapshot {
    entries: Vec<SnapshotEntry>,
}

struct PrepManifestInput<'a> {
    base_commit: &'a str,
    last_tag: &'a str,
    previous_version: &'a str,
    prepared_version: &'a str,
    bump_type: BumpType,
    migration_state: &'a PrepareMigrationState,
    changed_paths: &'a BTreeSet<String>,
}

fn main() -> Result<(), DynError> {
    let command = parse_args(env::args().skip(1).collect())?;
    ensure_project_root()?;

    match command {
        ReleaseCommand::Prepare(bump_type) => prepare_release(bump_type),
        ReleaseCommand::Finalize => finalize_release(),
    }
}

fn parse_args(args: Vec<String>) -> Result<ReleaseCommand, DynError> {
    match args.as_slice() {
        [command, bump_type] if command == "prepare" => {
            let bump_type = BumpType::parse(bump_type)
                .ok_or("Error: bump type must be 'patch', 'minor', or 'major'")?;
            Ok(ReleaseCommand::Prepare(bump_type))
        }
        [command] if command == "finalize" => Ok(ReleaseCommand::Finalize),
        [legacy_bump_type] => {
            let bump_type = BumpType::parse(legacy_bump_type).ok_or_else(|| {
                usage_error("Error: expected `prepare <patch|minor|major>` or `finalize`")
            })?;
            Ok(ReleaseCommand::Prepare(bump_type))
        }
        _ => Err(usage_error(
            "Usage: release prepare <patch|minor|major>\n       release finalize",
        )),
    }
}

fn usage_error(message: &str) -> DynError {
    message.to_string().into()
}

fn ensure_project_root() -> Result<(), DynError> {
    let current_dir = env::current_dir()?;
    if !current_dir.join(PACKAGE_JSON_PATH).exists() {
        return Err("Error: This binary must be run from the project root directory (where package.json is located)".into());
    }

    Ok(())
}

fn prepare_release(bump_type: BumpType) -> Result<(), DynError> {
    check_git_workspace_clean()?;

    let root = Path::new(".");
    let base_commit = git_stdout(["rev-parse", "HEAD"])?;
    let current_version = read_package_version(root)?;
    let prepared_version = bump_type.bump(&current_version);
    let prepared_version_str = prepared_version.to_string();

    println!(
        "🚀 Preparing release {} → {} ({})",
        current_version,
        prepared_version_str,
        bump_type.as_str()
    );

    let snapshot = capture_prepare_snapshot(root, &prepared_version_str)?;
    let prepare_result = (|| -> Result<PrepareMigrationState, DynError> {
        let migration_state = prepare_migration_state(root, &prepared_version_str)?;
        update_version_files(root, &current_version.to_string(), &prepared_version_str)?;

        println!("🔄 Running cargo check to refresh the prepared release state...");
        run_command_checked(
            Command::new("cargo")
                .args(["check"])
                .current_dir("src-tauri"),
            "cargo check failed",
        )?;

        let last_tag = get_last_tag()?;
        let commits = get_commits_since_tag(&last_tag)?;
        let categorized = categorize_commits(commits);
        update_releases_md(
            root.join(RELEASES_PATH).as_path(),
            &prepared_version_str,
            categorized,
        )?;

        let changed_paths = collect_changed_paths()?;
        let previous_version = current_version.to_string();
        let manifest = build_prep_manifest(
            root,
            PrepManifestInput {
                base_commit: &base_commit,
                last_tag: &last_tag,
                previous_version: &previous_version,
                prepared_version: &prepared_version_str,
                bump_type,
                migration_state: &migration_state,
                changed_paths: &changed_paths,
            },
        )?;
        write_prep_manifest(root, &manifest)?;
        write_internal_prep_manifest(root, &manifest)?;

        Ok(migration_state)
    })();

    let migration_state = match prepare_result {
        Ok(state) => state,
        Err(error) => {
            restore_prepare_snapshot(root, snapshot)?;
            return Err(format!(
                "prepare-release failed and changes were rolled back: {}",
                error
            )
            .into());
        }
    };

    println!("✅ Prepared release {}", prepared_version_str);
    println!(
        "   - Pending migration steps: {}",
        migration_state.pending_step_count
    );
    if let Some(path) = &migration_state.promoted_migration_path {
        println!("   - Promoted pending migration to {}", path);
    } else {
        println!("   - No pending migration steps were promoted");
    }
    println!("   - Review the working tree, build/test it, then run finalize-release");

    Ok(())
}

fn finalize_release() -> Result<(), DynError> {
    let root = Path::new(".");
    let manifest = read_prep_manifest(root)?;
    validate_finalize_state(root, &manifest)?;

    let release_commit_message = format!("chore(release): v{}", manifest.prepared_version);
    let stage_paths = tracked_stage_paths(&manifest);

    stage_paths_for_commit(&stage_paths)?;
    run_command_checked(
        Command::new("git").args(["commit", "-m", &release_commit_message]),
        "Failed to create release commit",
    )?;
    run_command_checked(
        Command::new("git").args(["tag", &manifest.release_tag]),
        "Failed to create release tag",
    )?;

    if let Err(error) = finalize_post_tag(root, &manifest) {
        return Err(post_tag_failure_message(&manifest.release_tag, &error).into());
    }

    println!(
        "🚀 Pushing branch HEAD and {} to origin...",
        manifest.release_tag
    );
    run_command_checked(
        Command::new("git").args(["push", "origin", "HEAD", &manifest.release_tag]),
        "Failed to push release commits and tag",
    )?;

    println!("🎉 Finalized release {}", manifest.prepared_version);
    Ok(())
}

fn finalize_post_tag(root: &Path, manifest: &ReleasePrepManifest) -> Result<(), String> {
    reseed_development_migration_state(root, &manifest.next_scaffold_source)?;

    let manifest_path = root.join(PREP_MANIFEST_PATH);
    if manifest_path.exists() {
        fs::remove_file(&manifest_path).map_err(|error| error.to_string())?;
    }
    let internal_manifest_path = root.join(INTERNAL_PREP_MANIFEST_PATH);
    if internal_manifest_path.exists() {
        fs::remove_file(&internal_manifest_path).map_err(|error| error.to_string())?;
    }

    stage_paths_for_commit(&reseed_stage_paths()).map_err(|error| error.to_string())?;

    run_command_checked(
        Command::new("git").args(["commit", "-m", RESEED_COMMIT_MESSAGE]),
        "Failed to create post-release reseed commit",
    )
    .map_err(|error| error.to_string())?;

    println!(
        "✅ Tagged {} and restored development-time next.rs scaffold",
        manifest.release_tag
    );
    Ok(())
}

fn read_package_version(root: &Path) -> Result<Version, DynError> {
    let package_json = fs::read_to_string(root.join(PACKAGE_JSON_PATH))?;
    let package_json: Value = serde_json::from_str(&package_json)?;
    let version = package_json["version"]
        .as_str()
        .ok_or("Could not find version in package.json")?;

    Ok(Version::parse(version)?)
}

fn update_version_files(
    root: &Path,
    current_version: &str,
    prepared_version: &str,
) -> Result<(), DynError> {
    update_package_json_version(root.join(PACKAGE_JSON_PATH).as_path(), prepared_version)?;
    update_package_lock_version(root.join(PACKAGE_LOCK_PATH).as_path(), prepared_version)?;
    update_cargo_toml_version(
        root.join(CARGO_TOML_PATH).as_path(),
        current_version,
        prepared_version,
    )?;
    update_json_version(root.join(TAURI_CONF_PATH).as_path(), prepared_version)?;
    Ok(())
}

fn update_package_json_version(path: &Path, new_version: &str) -> Result<(), DynError> {
    update_json_version(path, new_version)
}

fn update_package_lock_version(path: &Path, new_version: &str) -> Result<(), DynError> {
    let content = fs::read_to_string(path)?;
    let mut value: Value = serde_json::from_str(&content)?;
    value["version"] = Value::String(new_version.to_string());

    let packages = value
        .get_mut("packages")
        .and_then(Value::as_object_mut)
        .ok_or("package-lock.json is missing the packages object")?;
    let root_package = packages
        .get_mut("")
        .and_then(Value::as_object_mut)
        .ok_or("package-lock.json is missing the root package entry")?;
    root_package.insert(
        String::from("version"),
        Value::String(new_version.to_string()),
    );

    fs::write(path, serde_json::to_string_pretty(&value)?)?;
    Ok(())
}

fn update_json_version(path: &Path, new_version: &str) -> Result<(), DynError> {
    let content = fs::read_to_string(path)?;
    let mut value: Value = serde_json::from_str(&content)?;
    value["version"] = Value::String(new_version.to_string());
    fs::write(path, serde_json::to_string_pretty(&value)?)?;
    Ok(())
}

fn update_cargo_toml_version(
    path: &Path,
    current_version: &str,
    new_version: &str,
) -> Result<(), DynError> {
    let content = fs::read_to_string(path)?;
    let mut updated = Vec::new();
    let mut in_package_section = false;
    let mut replaced = false;

    for line in content.lines() {
        if line.starts_with("[") {
            in_package_section = line == "[package]";
        }

        if in_package_section && !replaced && line.trim_start().starts_with("version = ") {
            updated.push(format!("version = \"{}\"", new_version));
            replaced = true;
            continue;
        }

        updated.push(line.to_string());
    }

    if !replaced {
        return Err(format!(
            "Failed to update version in {} from {} to {}",
            path.display(),
            current_version,
            new_version
        )
        .into());
    }

    fs::write(path, format!("{}\n", updated.join("\n")))?;
    Ok(())
}

fn capture_prepare_snapshot(
    root: &Path,
    prepared_version: &str,
) -> Result<WorkspaceSnapshot, DynError> {
    let tracked_paths = [
        PACKAGE_JSON_PATH,
        PACKAGE_LOCK_PATH,
        CARGO_TOML_PATH,
        CARGO_LOCK_PATH,
        TAURI_CONF_PATH,
        RELEASES_PATH,
        MIGRATIONS_MOD_PATH,
        NEXT_MIGRATION_PATH,
        PREP_MANIFEST_PATH,
        INTERNAL_PREP_MANIFEST_PATH,
    ];

    let mut entries = Vec::new();
    for path in tracked_paths {
        entries.push(SnapshotEntry {
            path: String::from(path),
            contents: read_optional_bytes(root, path)?,
        });
    }
    entries.push(SnapshotEntry {
        path: format!("{}/{}.rs", MIGRATIONS_DIR, prepared_version),
        contents: read_optional_bytes(
            root,
            &format!("{}/{}.rs", MIGRATIONS_DIR, prepared_version),
        )?,
    });

    Ok(WorkspaceSnapshot { entries })
}

fn restore_prepare_snapshot(root: &Path, snapshot: WorkspaceSnapshot) -> Result<(), DynError> {
    for entry in snapshot.entries {
        let path = root.join(&entry.path);
        match entry.contents {
            Some(contents) => {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(path, contents)?;
            }
            None if path.exists() => {
                fs::remove_file(path)?;
            }
            None => {}
        }
    }

    Ok(())
}

fn read_optional_bytes(root: &Path, relative_path: &str) -> Result<Option<Vec<u8>>, DynError> {
    let path = root.join(relative_path);
    if path.exists() {
        Ok(Some(fs::read(path)?))
    } else {
        Ok(None)
    }
}

fn prepare_migration_state(
    root: &Path,
    prepared_version: &str,
) -> Result<PrepareMigrationState, DynError> {
    let next_path = root.join(NEXT_MIGRATION_PATH);
    let next_source = if next_path.exists() {
        Some(fs::read_to_string(&next_path)?)
    } else {
        None
    };
    let next_source = next_source.ok_or("Missing next.rs migration scaffold")?;
    let pending_step_count = count_pending_steps(&next_source)?;
    let mut shipped_versions = discover_shipped_migration_versions(root)?;
    let support_modules = read_support_modules(root)?;
    let next_scaffold_source = build_next_migration_scaffold(&next_source)?;
    let promoted_migration_path = if pending_step_count > 0 {
        let promoted_path = format!("{}/{}.rs", MIGRATIONS_DIR, prepared_version);
        let absolute_promoted_path = root.join(&promoted_path);
        if absolute_promoted_path.exists() {
            return Err(format!("Migration file already exists: {}", promoted_path).into());
        }

        fs::write(&absolute_promoted_path, &next_source)?;
        shipped_versions.push(Version::parse(prepared_version)?);
        Some(promoted_path)
    } else {
        None
    };

    if next_path.exists() {
        fs::remove_file(&next_path)?;
    }

    shipped_versions.sort();
    shipped_versions.dedup();
    fs::write(
        root.join(MIGRATIONS_MOD_PATH),
        build_migrations_mod(&shipped_versions, false, &support_modules),
    )?;

    Ok(PrepareMigrationState {
        promoted_migration_path,
        pending_step_count,
        next_scaffold_source,
    })
}

fn reseed_development_migration_state(
    root: &Path,
    next_scaffold_source: &str,
) -> Result<(), String> {
    let shipped_versions =
        discover_shipped_migration_versions(root).map_err(|error| error.to_string())?;
    let support_modules = read_support_modules(root).map_err(|error| error.to_string())?;
    fs::write(root.join(NEXT_MIGRATION_PATH), next_scaffold_source)
        .map_err(|error| error.to_string())?;
    fs::write(
        root.join(MIGRATIONS_MOD_PATH),
        build_migrations_mod(&shipped_versions, true, &support_modules),
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

fn discover_shipped_migration_versions(root: &Path) -> Result<Vec<Version>, DynError> {
    let migrations_dir = root.join(MIGRATIONS_DIR);
    let mut versions = Vec::new();

    for entry in fs::read_dir(migrations_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        if file_name == "mod.rs" || file_name == "next.rs" || !file_name.ends_with(".rs") {
            continue;
        }

        let version = file_name.trim_end_matches(".rs");
        versions.push(Version::parse(version)?);
    }

    versions.sort();
    Ok(versions)
}

fn count_pending_steps(source: &str) -> Result<usize, DynError> {
    let body = pending_steps_body(source)?;
    let mut current = String::new();
    let mut count = 0;

    for character in strip_comments(body).chars() {
        if character == ',' {
            if !current.trim().is_empty() {
                count += 1;
            }
            current.clear();
        } else {
            current.push(character);
        }
    }

    if !current.trim().is_empty() {
        count += 1;
    }

    Ok(count)
}

fn pending_steps_body(source: &str) -> Result<&str, DynError> {
    let start = source
        .find(PENDING_STEPS_START)
        .ok_or("next.rs is missing the PENDING_STEPS declaration")?;
    let body_start = start + PENDING_STEPS_START.len();
    let body_end = source[body_start..]
        .find(PENDING_STEPS_END)
        .map(|offset| body_start + offset)
        .ok_or("next.rs is missing the end of the PENDING_STEPS declaration")?;

    Ok(&source[body_start..body_end])
}

fn strip_comments(source: &str) -> String {
    let mut result = String::new();
    let mut chars = source.chars().peekable();
    let mut in_block_comment = false;

    while let Some(character) = chars.next() {
        if in_block_comment {
            if character == '*' && chars.peek() == Some(&'/') {
                chars.next();
                in_block_comment = false;
            }
            continue;
        }

        if character == '/' && chars.peek() == Some(&'*') {
            chars.next();
            in_block_comment = true;
            continue;
        }

        if character == '/' && chars.peek() == Some(&'/') {
            chars.next();
            for next_character in chars.by_ref() {
                if next_character == '\n' {
                    result.push('\n');
                    break;
                }
            }
            continue;
        }

        result.push(character);
    }

    result
}

fn read_support_modules(root: &Path) -> Result<String, DynError> {
    let mod_source = fs::read_to_string(root.join(MIGRATIONS_MOD_PATH))?;
    Ok(
        extract_section(&mod_source, SUPPORT_MODULES_START, SUPPORT_MODULES_END)
            .unwrap_or_default()
            .trim_matches('\n')
            .to_string(),
    )
}

fn extract_section<'a>(source: &'a str, start_marker: &str, end_marker: &str) -> Option<&'a str> {
    let start = source.find(start_marker)? + start_marker.len();
    let end = source[start..].find(end_marker)? + start;
    Some(&source[start..end])
}

fn build_migrations_mod(
    versions: &[Version],
    include_pending: bool,
    support_modules: &str,
) -> String {
    let mut lines = Vec::new();

    lines.push(String::from(GENERATED_MIGRATIONS_START));

    for version in versions {
        lines.push(format!("#[path = \"{}.rs\"]", version));
        lines.push(format!("mod {};", version_alias(version)));
    }

    if include_pending {
        lines.push(String::from("#[path = \"next.rs\"]"));
        lines.push(String::from("mod next;"));
    }

    lines.push(String::new());
    lines.push(String::from(SUPPORT_MODULES_START));
    if support_modules.trim().is_empty() {
        lines.push(String::from(
            "// Add non-version helper modules here. The release tool preserves this section.",
        ));
    } else {
        lines.extend(support_modules.trim_matches('\n').lines().map(String::from));
    }
    lines.push(String::from(SUPPORT_MODULES_END));
    lines.push(String::new());

    lines.push(String::from("use lazy_static::lazy_static;"));
    lines.push(String::from("use serde_json::Value;"));
    lines.push(String::new());
    lines.push(String::from(
        "pub type MigrationFn = fn(Value) -> Result<Value, String>;",
    ));
    lines.push(String::new());
    lines.push(String::from("#[derive(Clone, Copy, Debug, PartialEq, Eq)]"));
    lines.push(String::from("pub enum MigrationKind {"));
    lines.push(String::from("    Shipped(&'static str),"));
    lines.push(String::from("    PendingNext,"));
    lines.push(String::from("}"));
    lines.push(String::new());
    lines.push(String::from("#[derive(Clone, Copy)]"));
    lines.push(String::from("pub struct MigrationDescriptor {"));
    lines.push(String::from("    kind: MigrationKind,"));
    lines.push(String::from("    migrate: MigrationFn,"));
    lines.push(String::from("}"));
    lines.push(String::new());
    lines.push(String::from("impl MigrationDescriptor {"));
    lines.push(String::from(
        "    const fn shipped(version: &'static str, migrate: MigrationFn) -> Self {",
    ));
    lines.push(String::from("        Self {"));
    lines.push(String::from(
        "            kind: MigrationKind::Shipped(version),",
    ));
    lines.push(String::from("            migrate,"));
    lines.push(String::from("        }"));
    lines.push(String::from("    }"));
    lines.push(String::new());
    lines.push(String::from(
        "    const fn pending_next(migrate: MigrationFn) -> Self {",
    ));
    lines.push(String::from("        Self {"));
    lines.push(String::from(
        "            kind: MigrationKind::PendingNext,",
    ));
    lines.push(String::from("            migrate,"));
    lines.push(String::from("        }"));
    lines.push(String::from("    }"));
    lines.push(String::new());
    lines.push(String::from(
        "    pub const fn kind(&self) -> MigrationKind {",
    ));
    lines.push(String::from("        self.kind"));
    lines.push(String::from("    }"));
    lines.push(String::new());
    lines.push(String::from(
        "    pub const fn migrate(&self) -> MigrationFn {",
    ));
    lines.push(String::from("        self.migrate"));
    lines.push(String::from("    }"));
    lines.push(String::from("}"));
    lines.push(String::new());
    lines.push(String::from("lazy_static! {"));
    lines.push(String::from(
        "    pub static ref MIGRATIONS: Vec<MigrationDescriptor> = vec![",
    ));

    for version in versions {
        lines.push(format!(
            "        MigrationDescriptor::shipped(\"{}\", {}::migrate),",
            version,
            version_alias(version)
        ));
    }

    if include_pending {
        lines.push(String::from(
            "        MigrationDescriptor::pending_next(next::migrate),",
        ));
    }

    lines.push(String::from("    ];"));
    lines.push(String::from("}"));
    lines.push(String::new());
    lines.push(String::from(GENERATED_MIGRATIONS_END));

    lines.join("\n")
}

fn build_next_migration_scaffold(source: &str) -> Result<String, DynError> {
    let body = pending_steps_body(source)?;
    let body_start = body.as_ptr() as usize - source.as_ptr() as usize;
    let body_end = body_start + body.len();
    let mut scaffold = String::new();
    scaffold.push_str(&source[..body_start]);
    scaffold.push_str("\n    // Add pending migration steps here, in order.\n");
    scaffold.push_str(&source[body_end..]);
    Ok(scaffold)
}

fn version_alias(version: &Version) -> String {
    format!("v{}", version.to_string().replace('.', "_"))
}

fn get_last_tag() -> Result<String, DynError> {
    git_stdout(["describe", "--tags", "--abbrev=0"])
}

fn get_commits_since_tag(tag: &str) -> Result<Vec<String>, DynError> {
    let output = Command::new("git")
        .args(["log", "--oneline", &format!("{}..HEAD", tag)])
        .output()?;

    if !output.status.success() {
        return Err("Failed to get commits".into());
    }

    Ok(String::from_utf8(output.stdout)?
        .lines()
        .filter(|line| should_include_in_release_notes(line))
        .map(|line| line.to_string())
        .collect())
}

fn should_include_in_release_notes(commit: &str) -> bool {
    !commit.contains(RELEASE_NOTE_SKIP_MARKER)
}

fn categorize_commits(commits: Vec<String>) -> HashMap<String, Vec<String>> {
    let mut categorized = HashMap::new();
    categorized.insert(String::from("feat"), Vec::new());
    categorized.insert(String::from("fix"), Vec::new());
    categorized.insert(String::from("chore"), Vec::new());
    categorized.insert(String::from("other"), Vec::new());

    for commit in commits {
        let message = commit
            .split_once(' ')
            .map(|parts| parts.1)
            .unwrap_or(&commit);
        if message.starts_with("feat:") || message.starts_with("feat(") {
            categorized
                .get_mut("feat")
                .expect("feat bucket exists")
                .push(message.to_string());
        } else if message.starts_with("fix:") || message.starts_with("fix(") {
            categorized
                .get_mut("fix")
                .expect("fix bucket exists")
                .push(message.to_string());
        } else if message.starts_with("chore:") || message.starts_with("chore(") {
            categorized
                .get_mut("chore")
                .expect("chore bucket exists")
                .push(message.to_string());
        } else {
            categorized
                .get_mut("other")
                .expect("other bucket exists")
                .push(message.to_string());
        }
    }

    categorized
}

fn update_releases_md(
    releases_path: &Path,
    version: &str,
    categorized: HashMap<String, Vec<String>>,
) -> Result<(), DynError> {
    let mut content = fs::read_to_string(releases_path)?;
    let mut new_section = format!("## Release Notes for v{}\n\n", version);

    append_release_section(
        &mut new_section,
        "### 🏗️ Features\n\n",
        categorized.get("feat"),
    );
    append_release_section(&mut new_section, "### 🐞 Fixes\n\n", categorized.get("fix"));
    append_release_section(
        &mut new_section,
        "### 🔧 Chores\n\n",
        categorized.get("chore"),
    );
    append_release_section(
        &mut new_section,
        "### 📝 Other\n\n",
        categorized.get("other"),
    );

    if content.starts_with("# ") {
        if let Some(header_end) = content.find("\n\n") {
            let header = &content[..header_end + 2];
            let rest = content[header_end + 2..].trim_start().to_string();
            content = format!("{}{}\n{}", header, new_section, rest);
        } else {
            content = format!("{}{}", new_section, content);
        }
    } else {
        content = format!("{}{}", new_section, content);
    }

    fs::write(releases_path, content)?;
    Ok(())
}

fn append_release_section(section: &mut String, header: &str, entries: Option<&Vec<String>>) {
    let Some(entries) = entries else {
        return;
    };

    if entries.is_empty() {
        return;
    }

    section.push_str(header);
    for entry in entries {
        section.push_str(&format!("- {}\n", entry));
    }
    section.push('\n');
}

fn build_prep_manifest(
    root: &Path,
    input: PrepManifestInput<'_>,
) -> Result<ReleasePrepManifest, DynError> {
    let mut managed_files = Vec::new();
    let mut absent_paths = Vec::new();

    for path in input.changed_paths {
        let absolute_path = root.join(path);
        if absolute_path.exists() {
            managed_files.push(PreparedFileHash {
                path: path.clone(),
                blake3: blake3_hex(&absolute_path)?,
            });
        } else {
            absent_paths.push(path.clone());
        }
    }

    managed_files.sort_by(|left, right| left.path.cmp(&right.path));
    absent_paths.sort();

    Ok(ReleasePrepManifest {
        base_commit: input.base_commit.to_string(),
        previous_version: input.previous_version.to_string(),
        prepared_version: input.prepared_version.to_string(),
        bump_type: input.bump_type,
        last_tag: input.last_tag.to_string(),
        release_tag: format!("v{}", input.prepared_version),
        promoted_migration_path: input.migration_state.promoted_migration_path.clone(),
        pending_step_count: input.migration_state.pending_step_count,
        next_scaffold_source: input.migration_state.next_scaffold_source.clone(),
        changed_paths: input.changed_paths.iter().cloned().collect(),
        managed_files,
        absent_paths,
    })
}

fn write_prep_manifest(root: &Path, manifest: &ReleasePrepManifest) -> Result<(), DynError> {
    fs::write(
        root.join(PREP_MANIFEST_PATH),
        serde_json::to_string_pretty(manifest)?,
    )?;
    Ok(())
}

fn write_internal_prep_manifest(
    root: &Path,
    manifest: &ReleasePrepManifest,
) -> Result<(), DynError> {
    let path = root.join(INTERNAL_PREP_MANIFEST_PATH);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(manifest)?)?;
    Ok(())
}

fn read_prep_manifest(root: &Path) -> Result<ReleasePrepManifest, DynError> {
    let content = fs::read_to_string(root.join(PREP_MANIFEST_PATH))
        .map_err(|_| "Missing .release-prep.json; run prepare-release first")?;
    Ok(serde_json::from_str(&content)
        .map_err(|error| format!("Malformed .release-prep.json: {}", error))?)
}

fn read_internal_prep_manifest(root: &Path) -> Result<ReleasePrepManifest, DynError> {
    let content = fs::read_to_string(root.join(INTERNAL_PREP_MANIFEST_PATH))
        .map_err(|_| "Missing internal prepare metadata; rerun prepare-release")?;
    Ok(serde_json::from_str(&content)
        .map_err(|error| format!("Malformed internal prepare metadata: {}", error))?)
}

fn validate_finalize_state(root: &Path, manifest: &ReleasePrepManifest) -> Result<(), DynError> {
    let internal_manifest = read_internal_prep_manifest(root)?;
    if internal_manifest != *manifest {
        return Err(
            "Prepared manifest no longer matches the internal release metadata; rerun prepare-release"
                .into(),
        );
    }

    let head = git_stdout(["rev-parse", "HEAD"])?;
    if head != manifest.base_commit {
        return Err(format!(
            "Prepared release no longer matches the expected base commit (expected {}, found {})",
            manifest.base_commit, head
        )
        .into());
    }

    let changed_paths = collect_changed_paths()?;
    validate_prepared_state_files(root, manifest, &changed_paths)?;

    let existing_tags = git_stdout_allow_empty(["tag", "--list", &manifest.release_tag])?;
    if !existing_tags.trim().is_empty() {
        return Err(format!("Tag {} already exists", manifest.release_tag).into());
    }

    Ok(())
}

fn validate_prepared_state_files(
    root: &Path,
    manifest: &ReleasePrepManifest,
    changed_paths: &BTreeSet<String>,
) -> Result<(), DynError> {
    let mut expected_paths: BTreeSet<String> = manifest.changed_paths.iter().cloned().collect();
    expected_paths.remove(PREP_MANIFEST_PATH);

    if changed_paths != &expected_paths {
        return Err(format!(
            "Prepared working tree has drifted. Expected changed paths {:?}, found {:?}",
            expected_paths, changed_paths
        )
        .into());
    }

    for file in &manifest.managed_files {
        let path = root.join(&file.path);
        if !path.exists() {
            return Err(format!("Prepared file is missing: {}", file.path).into());
        }

        let current_hash = blake3_hex(&path)?;
        if current_hash != file.blake3 {
            return Err(format!("Prepared file content changed: {}", file.path).into());
        }
    }

    for path in &manifest.absent_paths {
        if root.join(path).exists() {
            return Err(format!("Expected {} to remain absent during finalize", path).into());
        }
    }

    Ok(())
}

fn tracked_stage_paths(manifest: &ReleasePrepManifest) -> Vec<String> {
    manifest
        .changed_paths
        .iter()
        .filter(|path| path.as_str() != PREP_MANIFEST_PATH)
        .cloned()
        .collect()
}

fn reseed_stage_paths() -> Vec<String> {
    vec![
        String::from(MIGRATIONS_MOD_PATH),
        String::from(NEXT_MIGRATION_PATH),
    ]
}

fn stage_paths_for_commit(paths: &[String]) -> Result<(), DynError> {
    if paths.is_empty() {
        return Ok(());
    }

    let mut command = Command::new("git");
    command.arg("add").arg("-A").arg("--");
    for path in paths {
        command.arg(path);
    }

    run_command_checked(&mut command, "Failed to stage release files")
}

fn collect_changed_paths() -> Result<BTreeSet<String>, DynError> {
    let output = Command::new("git")
        .args(["status", "--porcelain", "--untracked-files=all"])
        .output()?;

    if !output.status.success() {
        return Err("Failed to inspect git status".into());
    }

    let mut paths = BTreeSet::new();
    for line in String::from_utf8(output.stdout)?.lines() {
        if line.len() < 4 {
            continue;
        }

        let raw_path = &line[3..];
        let normalized_path = raw_path
            .split(" -> ")
            .last()
            .unwrap_or(raw_path)
            .to_string();
        paths.insert(normalized_path);
    }

    Ok(paths)
}

fn blake3_hex(path: &Path) -> Result<String, DynError> {
    let bytes = fs::read(path)?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn git_stdout<const N: usize>(args: [&str; N]) -> Result<String, DynError> {
    let output = Command::new("git").args(args).output()?;
    if !output.status.success() {
        return Err(format!("git command failed: git {}", args.join(" ")).into());
    }

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn git_stdout_allow_empty<const N: usize>(args: [&str; N]) -> Result<String, DynError> {
    let output = Command::new("git").args(args).output()?;
    if !output.status.success() {
        return Err(format!("git command failed: git {}", args.join(" ")).into());
    }

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn run_command_checked(command: &mut Command, error_message: &str) -> Result<(), DynError> {
    let status = command.status()?;
    if !status.success() {
        return Err(error_message.to_string().into());
    }

    Ok(())
}

fn post_tag_failure_message(release_tag: &str, error: &str) -> String {
    format!(
        "{}\nRelease commit and tag {} were already created. Recovery: restore the post-release scaffold if needed, commit it with `{}`, then push with `git push origin HEAD {}`.",
        error, release_tag, RESEED_COMMIT_MESSAGE, release_tag
    )
}

fn check_git_workspace_clean() -> Result<(), DynError> {
    let output = Command::new("git")
        .args(["status", "--porcelain", "--untracked-files=all"])
        .output()?;

    if !output.status.success() {
        return Err("Failed to check git status".into());
    }

    let stdout = String::from_utf8(output.stdout)?;
    if !stdout.trim().is_empty() {
        eprintln!("❌ Error: Git workspace is not clean!");
        eprintln!("   Uncommitted changes found:");
        for line in stdout.lines() {
            eprintln!("   {}", line);
        }
        eprintln!();
        eprintln!("   Please commit or stash your changes before running the release script.");
        process::exit(1);
    }

    println!("✅ Git workspace is clean");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    const NEXT_SCAFFOLD_TEMPLATE: &str = concat!(
        "use serde_json::Value;\n\n",
        "type PendingStep = fn(Value) -> Result<Value, String>;\n\n",
        "const PENDING_STEPS: &[PendingStep] = &[\n",
        "];\n\n",
        "pub fn migrate(mut data: Value) -> Result<Value, String> {\n",
        "    for step in PENDING_STEPS {\n",
        "        data = step(data)?;\n",
        "    }\n\n",
        "    Ok(data)\n",
        "}\n"
    );

    fn make_temp_dir(name: &str) -> PathBuf {
        let unique = format!(
            "compose-release-tests-{}-{}-{}",
            name,
            process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let dir = env::temp_dir().join(unique);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_file(root: &Path, relative_path: &str, content: &str) {
        let path = root.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    #[test]
    fn update_package_lock_version_updates_root_metadata() {
        let root = make_temp_dir("package-lock");
        write_file(
            &root,
            PACKAGE_LOCK_PATH,
            "{\n  \"name\": \"compose\",\n  \"version\": \"1.25.0\",\n  \"packages\": {\n    \"\": {\n      \"name\": \"compose\",\n      \"version\": \"1.25.0\"\n    }\n  }\n}\n",
        );

        update_package_lock_version(&root.join(PACKAGE_LOCK_PATH), "1.25.1").unwrap();

        let updated: Value =
            serde_json::from_str(&fs::read_to_string(root.join(PACKAGE_LOCK_PATH)).unwrap())
                .unwrap();

        assert_eq!(updated["version"], Value::String(String::from("1.25.1")));
        assert_eq!(
            updated["packages"][""]["version"],
            Value::String(String::from("1.25.1"))
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn count_pending_steps_detects_empty_scaffold() {
        let scaffold = build_next_migration_scaffold(NEXT_SCAFFOLD_TEMPLATE).unwrap();
        assert_eq!(count_pending_steps(&scaffold).unwrap(), 0);
    }

    #[test]
    fn count_pending_steps_detects_real_steps() {
        let source = "use serde_json::Value;\n\nconst PENDING_STEPS: &[PendingStep] = &[\n    normalize_text,\n    rewrite_headers,\n];\n";
        assert_eq!(count_pending_steps(source).unwrap(), 2);
    }

    #[test]
    fn count_pending_steps_ignores_line_and_block_comments() {
        let source = "use serde_json::Value;\n\nconst PENDING_STEPS: &[PendingStep] = &[\n    // comment\n    normalize_text, /* keep */\n    /* block\n       comment */\n    rewrite_headers,\n];\n";

        assert_eq!(count_pending_steps(source).unwrap(), 2);
    }

    #[test]
    fn build_migrations_mod_includes_pending_only_when_requested() {
        let versions = vec![Version::parse("1.25.0").unwrap()];

        let with_pending = build_migrations_mod(&versions, true, "");
        let shipped_only = build_migrations_mod(&versions, false, "");

        assert!(with_pending.contains("MigrationDescriptor::pending_next(next::migrate)"));
        assert!(with_pending.contains("mod next;"));
        assert!(!shipped_only.contains("MigrationDescriptor::pending_next(next::migrate)"));
        assert!(!shipped_only.contains("mod next;"));
    }

    #[test]
    fn categorize_commits_excludes_skip_release_note_marker() {
        let commits = vec![
            String::from("abc123 feat: keep me"),
            format!("def456 {}", RESEED_COMMIT_MESSAGE),
        ]
        .into_iter()
        .filter(|commit| should_include_in_release_notes(commit))
        .collect();

        let categorized = categorize_commits(commits);

        assert_eq!(
            categorized.get("feat").unwrap(),
            &vec![String::from("feat: keep me")]
        );
        assert!(categorized.get("chore").unwrap().is_empty());
    }

    #[test]
    fn reseed_development_migration_state_restores_next_scaffold() {
        let root = make_temp_dir("reseed");
        write_file(&root, "src-tauri/src/migrations/1.25.0.rs", "pub fn migrate(_: serde_json::Value) -> Result<serde_json::Value, String> { unreachable!() }\n");
        write_file(&root, MIGRATIONS_MOD_PATH, "placeholder\n");
        write_file(&root, PREP_MANIFEST_PATH, "{}\n");
        let scaffold = build_next_migration_scaffold(NEXT_SCAFFOLD_TEMPLATE).unwrap();

        reseed_development_migration_state(&root, &scaffold).unwrap();

        let next_source = fs::read_to_string(root.join(NEXT_MIGRATION_PATH)).unwrap();
        let mod_source = fs::read_to_string(root.join(MIGRATIONS_MOD_PATH)).unwrap();

        assert!(next_source.contains("const PENDING_STEPS: &[PendingStep] = &["));
        assert!(mod_source.contains("MigrationDescriptor::pending_next(next::migrate)"));
        assert!(mod_source.contains("MigrationDescriptor::shipped(\"1.25.0\", v1_25_0::migrate)"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn prepare_migration_state_promotes_non_empty_next_rs() {
        let root = make_temp_dir("promote");
        write_file(
            &root,
            MIGRATIONS_MOD_PATH,
            &build_migrations_mod(&[Version::parse("1.25.0").unwrap()], true, ""),
        );
        write_file(
            &root,
            NEXT_MIGRATION_PATH,
            "use serde_json::Value;\n\ntype PendingStep = fn(Value) -> Result<Value, String>;\n\nfn normalize_text(data: Value) -> Result<Value, String> { Ok(data) }\n\nconst PENDING_STEPS: &[PendingStep] = &[\n    normalize_text,\n];\n\npub fn migrate(mut data: Value) -> Result<Value, String> {\n    for step in PENDING_STEPS {\n        data = step(data)?;\n    }\n\n    Ok(data)\n}\n",
        );
        write_file(&root, "src-tauri/src/migrations/1.25.0.rs", "pub fn migrate(data: serde_json::Value) -> Result<serde_json::Value, String> { Ok(data) }\n");

        let state = prepare_migration_state(&root, "1.25.1").unwrap();
        let promoted = fs::read_to_string(root.join("src-tauri/src/migrations/1.25.1.rs")).unwrap();
        let mod_source = fs::read_to_string(root.join(MIGRATIONS_MOD_PATH)).unwrap();

        assert_eq!(state.pending_step_count, 1);
        assert_eq!(
            state.promoted_migration_path,
            Some(String::from("src-tauri/src/migrations/1.25.1.rs"))
        );
        assert!(promoted.contains("normalize_text"));
        assert!(!root.join(NEXT_MIGRATION_PATH).exists());
        assert!(!mod_source.contains("MigrationDescriptor::pending_next(next::migrate)"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn validate_prepared_state_files_rejects_drifted_content() {
        let root = make_temp_dir("manifest");
        write_file(
            &root,
            PACKAGE_JSON_PATH,
            "{\n  \"version\": \"1.25.0\"\n}\n",
        );
        write_file(&root, PREP_MANIFEST_PATH, "{}\n");

        let manifest = ReleasePrepManifest {
            base_commit: String::from("abc"),
            previous_version: String::from("1.25.0"),
            prepared_version: String::from("1.25.1"),
            bump_type: BumpType::Patch,
            last_tag: String::from("v1.25.0"),
            release_tag: String::from("v1.25.1"),
            promoted_migration_path: None,
            pending_step_count: 0,
            next_scaffold_source: String::from("scaffold"),
            changed_paths: vec![
                String::from(PACKAGE_JSON_PATH),
                String::from(PREP_MANIFEST_PATH),
            ],
            managed_files: vec![PreparedFileHash {
                path: String::from(PACKAGE_JSON_PATH),
                blake3: blake3_hex(&root.join(PACKAGE_JSON_PATH)).unwrap(),
            }],
            absent_paths: vec![],
        };

        fs::write(root.join(PACKAGE_JSON_PATH), "changed\n").unwrap();
        let changed_paths = BTreeSet::from([
            String::from(PACKAGE_JSON_PATH),
            String::from(PREP_MANIFEST_PATH),
        ]);

        assert!(validate_prepared_state_files(&root, &manifest, &changed_paths).is_err());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn validate_prepared_state_files_allows_legacy_visible_manifest_path() {
        let root = make_temp_dir("legacy-manifest");
        write_file(
            &root,
            PACKAGE_JSON_PATH,
            "{\n  \"version\": \"1.25.0\"\n}\n",
        );
        write_file(&root, PREP_MANIFEST_PATH, "{}\n");

        let manifest = ReleasePrepManifest {
            base_commit: String::from("abc"),
            previous_version: String::from("1.25.0"),
            prepared_version: String::from("1.25.1"),
            bump_type: BumpType::Patch,
            last_tag: String::from("v1.25.0"),
            release_tag: String::from("v1.25.1"),
            promoted_migration_path: None,
            pending_step_count: 0,
            next_scaffold_source: String::from("scaffold"),
            changed_paths: vec![
                String::from(PACKAGE_JSON_PATH),
                String::from(PREP_MANIFEST_PATH),
            ],
            managed_files: vec![PreparedFileHash {
                path: String::from(PACKAGE_JSON_PATH),
                blake3: blake3_hex(&root.join(PACKAGE_JSON_PATH)).unwrap(),
            }],
            absent_paths: vec![],
        };

        let changed_paths = BTreeSet::from([String::from(PACKAGE_JSON_PATH)]);

        validate_prepared_state_files(&root, &manifest, &changed_paths).unwrap();

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reseed_stage_paths_do_not_include_prep_manifest() {
        let paths = reseed_stage_paths();

        assert!(paths.contains(&String::from(MIGRATIONS_MOD_PATH)));
        assert!(paths.contains(&String::from(NEXT_MIGRATION_PATH)));
        assert!(!paths.contains(&String::from(PREP_MANIFEST_PATH)));
    }
}
