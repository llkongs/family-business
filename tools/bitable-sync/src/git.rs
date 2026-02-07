use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Run a git command in the given repository directory
fn git(repo_path: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_path)
        .output()
        .with_context(|| format!("Failed to execute: git {}", args.join(" ")))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        anyhow::bail!(
            "git {} failed (exit {}): {}",
            args.join(" "),
            output.status.code().unwrap_or(-1),
            stderr
        );
    }

    if !stderr.is_empty() {
        tracing::debug!("git stderr: {}", stderr.trim());
    }

    Ok(stdout)
}

/// Check if there are any changes to commit
pub fn has_changes(repo_path: &Path) -> Result<bool> {
    let status = git(repo_path, &["status", "--porcelain"])?;
    Ok(!status.trim().is_empty())
}

/// Stage, commit, and push changes.
/// Uses `git add -A` on managed directories so that deleted files are
/// correctly removed from the index (fixes historical garbage accumulation).
pub fn commit_and_push(repo_path: &Path, files: &[&str]) -> Result<()> {
    // Stage managed directories with -A to capture additions AND deletions.
    // Individual files outside these dirs are staged explicitly.
    let managed_dirs = ["src/data", "public/videos", "public/images"];

    for dir in &managed_dirs {
        let dir_path = repo_path.join(dir);
        if dir_path.exists() {
            git(repo_path, &["add", "-A", dir])
                .with_context(|| format!("Failed to stage directory {}", dir))?;
        }
    }

    // Also stage any explicitly listed files not already covered
    for file in files {
        let covered = managed_dirs.iter().any(|d| file.starts_with(d));
        if !covered {
            git(repo_path, &["add", file])
                .with_context(|| format!("Failed to stage {}", file))?;
        }
    }

    // Check if there are staged changes
    let diff = git(repo_path, &["diff", "--cached", "--name-only"])?;
    if diff.trim().is_empty() {
        tracing::info!("No changes to commit");
        return Ok(());
    }

    tracing::info!("Staged files:\n{}", diff.trim());

    // Commit
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string();
    let message = format!("chore: sync product data from bitable ({})", now);
    git(repo_path, &["commit", "-m", &message])?;
    tracing::info!("Committed: {}", message);

    // Push
    git(repo_path, &["push"])?;
    tracing::info!("Pushed to remote");

    Ok(())
}
