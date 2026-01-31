use semver::Version;
use serde_json::Value;
use std::env;
use std::fs;
use std::process::Command;

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
    println!("📋 Next steps:");
    println!("   1. Review the changes");
    println!("   2. Push the tag: git push origin v{}", new_version);
    println!("   3. Create a GitHub release if needed");

    Ok(())
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
        let message = commit.splitn(2, ' ').nth(1).unwrap_or(&commit);
        if message.starts_with("feat:") {
            categorized
                .get_mut("feat")
                .unwrap()
                .push(message.to_string());
        } else if message.starts_with("fix:") {
            categorized
                .get_mut("fix")
                .unwrap()
                .push(message.to_string());
        } else if message.starts_with("chore:") {
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
            new_section.push_str("\n");
        }
    }

    // Add fixes
    if let Some(fixes) = categorized.get("fix") {
        if !fixes.is_empty() {
            new_section.push_str("### 🐞 Fixes\n\n");
            for fix in fixes {
                new_section.push_str(&format!("- {}\n", fix));
            }
            new_section.push_str("\n");
        }
    }

    // Add chores
    if let Some(chores) = categorized.get("chore") {
        if !chores.is_empty() {
            new_section.push_str("### 🔧 Chores\n\n");
            for chore in chores {
                new_section.push_str(&format!("- {}\n", chore));
            }
            new_section.push_str("\n");
        }
    }

    // Add other commits
    if let Some(others) = categorized.get("other") {
        if !others.is_empty() {
            new_section.push_str("### 📝 Other\n\n");
            for other in others {
                new_section.push_str(&format!("- {}\n", other));
            }
            new_section.push_str("\n");
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
