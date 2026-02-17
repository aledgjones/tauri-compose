use semver::Version;
use serde_json::Value;
use std::env;
use std::fs;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <patch|minor|major>", args[0]);
        std::process::exit(1);
    }

    let bump_type = &args[1];
    if !["patch", "minor", "major"].contains(&bump_type.as_str()) {
        eprintln!("Error: bump type must be 'patch', 'minor', or 'major'");
        std::process::exit(1);
    }

    // Ensure we're running from the project root directory
    let current_dir = env::current_dir()?;
    if !current_dir.join("package.json").exists() {
        eprintln!("Error: This binary must be run from the project root directory (where package.json is located)");
        std::process::exit(1);
    }

    // Ensure git workspace is clean
    check_git_workspace_clean()?;

    println!("🚀 Starting release process for {} bump", bump_type);

    // Get current version from package.json
    let package_json_path = "package.json";
    let package_json_content = fs::read_to_string(package_json_path)?;
    let mut package_json: Value = serde_json::from_str(&package_json_content)?;

    let current_version_str = package_json["version"]
        .as_str()
        .ok_or("Could not find version in package.json")?
        .to_string();

    let mut current_version = Version::parse(&current_version_str)?;

    // Bump version
    match bump_type.as_str() {
        "patch" => current_version.patch += 1,
        "minor" => {
            current_version.minor += 1;
            current_version.patch = 0;
        }
        "major" => {
            current_version.major += 1;
            current_version.minor = 0;
            current_version.patch = 0;
        }
        _ => unreachable!(),
    }

    let new_version = current_version.to_string();
    println!(
        "📦 Bumping version: {} → {}",
        current_version_str, new_version
    );

    // Update package.json
    package_json["version"] = Value::String(new_version.clone());
    let updated_package_json = serde_json::to_string_pretty(&package_json)?;
    fs::write(package_json_path, updated_package_json)?;
    println!("✅ Updated package.json");

    // Update Cargo.toml
    let cargo_toml_path = "src-tauri/Cargo.toml";
    let cargo_toml_content = fs::read_to_string(cargo_toml_path)?;
    let updated_cargo_toml = cargo_toml_content.replace(
        &format!("version = \"{}\"", current_version_str),
        &format!("version = \"{}\"", new_version),
    );
    fs::write(cargo_toml_path, updated_cargo_toml)?;
    println!("✅ Updated Cargo.toml");

    // Update tauri.conf.json
    let tauri_conf_path = "src-tauri/tauri.conf.json";
    let tauri_conf_content = fs::read_to_string(tauri_conf_path)?;
    let updated_tauri_conf = tauri_conf_content.replace(
        &format!("\"version\": \"{}\"", current_version_str),
        &format!("\"version\": \"{}\"", new_version),
    );
    fs::write(tauri_conf_path, updated_tauri_conf)?;
    println!("✅ Updated tauri.conf.json");

    // Update Cargo.lock by running cargo check
    println!("🔄 Updating Cargo.lock...");
    Command::new("cargo")
        .args(["check"])
        .current_dir("src-tauri")
        .status()?;
    println!("✅ Updated Cargo.lock");

    // Get commits since last tag
    let last_tag = get_last_tag()?;
    println!("🏷️  Last tag: {}", last_tag);

    let commits = get_commits_since_tag(&last_tag)?;
    println!("📝 Found {} commits since last release", commits.len());

    // Categorize commits
    let categorized = categorize_commits(commits);

    // Update releases.md
    update_releases_md(&new_version, categorized)?;
    println!("📄 Updated releases.md");

    // Create git tag
    create_git_tag(&new_version)?;
    println!("🏷️  Created git tag: v{}", new_version);

    println!("\n🎉 Release {} completed successfully!", new_version);

    // Push the tag
    println!("🚀 Pushing tag v{}...", new_version);
    let push_status = Command::new("git")
        .args(["push", "origin", &format!("v{}", new_version)])
        .status()?;
    if !push_status.success() {
        eprintln!("❌ Failed to push tag v{}", new_version);
        std::process::exit(1);
    }
    println!("✅ Tag v{} pushed", new_version);

    // Generate release notes from releases.md
    let releases_md = fs::read_to_string("releases.md")?;
    let release_section = extract_release_notes(&releases_md, &new_version);

    // Write release notes to a temporary file
    let mut notes_file = NamedTempFile::new()?;
    notes_file.write_all(release_section.as_bytes())?;
    let notes_path = notes_file.path().to_str().unwrap();

    // Create draft GitHub release with notes-file
    println!("🚀 Creating draft GitHub release...");
    let gh_status = Command::new("gh")
        .args([
            "release",
            "create",
            &format!("v{}", new_version),
            "--draft",
            "--title",
            &format!("v{}", new_version),
            "--notes-file",
            notes_path,
        ])
        .status()?;
    if !gh_status.success() {
        eprintln!(
            "❌ Failed to create draft GitHub release for v{}",
            new_version
        );
        std::process::exit(1);
    }
    println!("✅ Draft GitHub release created for v{}", new_version);

    // Open the draft release in browser
    println!("🌐 Opening draft GitHub release in browser...");
    let repo_url = get_github_repo_url()?;
    let release_edit_url = format!("{}/releases/edit/v{}", repo_url, new_version);
    Command::new("open").arg(&release_edit_url).status()?;
    println!("✅ Draft release opened in browser: {}", release_edit_url);

    Ok(())
}

fn extract_release_notes(releases_md: &str, version: &str) -> String {
    let header = format!("## Release Notes for v{}", version);
    if let Some(start) = releases_md.find(&header) {
        let after_header = &releases_md[start..];
        let end = after_header
            .find("## Release Notes for v")
            .unwrap_or(after_header.len());
        after_header[..end].trim().to_string()
    } else {
        format!("Release notes for v{} not found.", version)
    }
}

fn get_github_repo_url() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(["config", "--get", "remote.origin.url"])
        .output()?;
    if !output.status.success() {
        return Err("Failed to get GitHub repo URL".into());
    }
    let url = String::from_utf8(output.stdout)?.trim().to_string();
    // Convert SSH or HTTPS to web URL
    if url.starts_with("git@github.com:") {
        let repo = url
            .trim_start_matches("git@github.com:")
            .trim_end_matches(".git");
        Ok(format!("https://github.com/{}", repo))
    } else if url.starts_with("https://github.com/") {
        let repo = url
            .trim_start_matches("https://github.com/")
            .trim_end_matches(".git");
        Ok(format!("https://github.com/{}", repo))
    } else {
        Err("Unsupported GitHub remote URL format".into())
    }
}

fn get_last_tag() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(["describe", "--tags", "--abbrev=0"])
        .output()?;

    if !output.status.success() {
        return Err("Failed to get last tag".into());
    }

    let tag = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(tag)
}

fn get_commits_since_tag(tag: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(["log", "--oneline", &format!("{}..HEAD", tag)])
        .output()?;

    if !output.status.success() {
        return Err("Failed to get commits".into());
    }

    let commits = String::from_utf8(output.stdout)?
        .lines()
        .map(|line| line.to_string())
        .collect();

    Ok(commits)
}

fn categorize_commits(commits: Vec<String>) -> std::collections::HashMap<String, Vec<String>> {
    let mut categorized = std::collections::HashMap::new();
    categorized.insert("feat".to_string(), Vec::new());
    categorized.insert("fix".to_string(), Vec::new());
    categorized.insert("chore".to_string(), Vec::new());
    categorized.insert("other".to_string(), Vec::new());

    for commit in commits {
        let message = commit.split_once(' ').map(|x| x.1).unwrap_or(&commit);
        // Accept feat:, feat(SC-XXX):, fix:, fix(SC-XXX):, chore:, chore(SC-XXX):
        if message.starts_with("feat:") || message.starts_with("feat(") {
            categorized
                .get_mut("feat")
                .unwrap()
                .push(message.to_string());
        } else if message.starts_with("fix:") || message.starts_with("fix(") {
            categorized
                .get_mut("fix")
                .unwrap()
                .push(message.to_string());
        } else if message.starts_with("chore:") || message.starts_with("chore(") {
            categorized
                .get_mut("chore")
                .unwrap()
                .push(message.to_string());
        } else {
            categorized
                .get_mut("other")
                .unwrap()
                .push(message.to_string());
        }
    }

    categorized
}

fn update_releases_md(
    version: &str,
    categorized: std::collections::HashMap<String, Vec<String>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let releases_path = "releases.md";
    let mut content = fs::read_to_string(releases_path)?;

    // Create new release section
    let mut new_section = format!("## Release Notes for v{}\n\n", version);

    // Add features
    if let Some(feats) = categorized.get("feat") {
        if !feats.is_empty() {
            new_section.push_str("### 🏗️ Features\n\n");
            for feat in feats {
                new_section.push_str(&format!("- {}\n", feat));
            }
            new_section.push('\n');
        }
    }

    // Add fixes
    if let Some(fixes) = categorized.get("fix") {
        if !fixes.is_empty() {
            new_section.push_str("### 🐞 Fixes\n\n");
            for fix in fixes {
                new_section.push_str(&format!("- {}\n", fix));
            }
            new_section.push('\n');
        }
    }

    // Add chores
    if let Some(chores) = categorized.get("chore") {
        if !chores.is_empty() {
            new_section.push_str("### 🔧 Chores\n\n");
            for chore in chores {
                new_section.push_str(&format!("- {}\n", chore));
            }
            new_section.push('\n');
        }
    }

    // Add other commits
    if let Some(others) = categorized.get("other") {
        if !others.is_empty() {
            new_section.push_str("### 📝 Other\n\n");
            for other in others {
                new_section.push_str(&format!("- {}\n", other));
            }
            new_section.push('\n');
        }
    }

    // Insert at the top (after any header)
    if content.starts_with("# ") {
        // Find the end of the header
        if let Some(header_end) = content.find("\n\n") {
            let header = &content[..header_end + 2];
            let rest = &content[header_end + 2..];
            // Trim leading whitespace from rest to avoid extra blank lines
            let rest = rest.trim_start();
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

fn create_git_tag(version: &str) -> Result<(), Box<dyn std::error::Error>> {
    let tag_name = format!("v{}", version);

    // Add all changes
    Command::new("git").args(["add", "."]).status()?;

    // Commit changes
    Command::new("git")
        .args(["commit", "-m", &format!("chore: release v{}", version)])
        .status()?;

    // Create tag
    Command::new("git").args(["tag", &tag_name]).status()?;

    Ok(())
}

fn check_git_workspace_clean() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
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
        std::process::exit(1);
    }

    println!("✅ Git workspace is clean");
    Ok(())
}
