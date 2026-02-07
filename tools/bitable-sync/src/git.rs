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

/// Stage, commit, and push changes
pub fn commit_and_push(repo_path: &Path, files: &[&str]) -> Result<()> {
    // Stage specific files
    for file in files {
        git(repo_path, &["add", file])
            .with_context(|| format!("Failed to stage {}", file))?;
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
