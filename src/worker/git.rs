use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;
use tracing::{debug, error, info, instrument, warn};

pub struct GitOps {
    repo_path: std::path::PathBuf,
}

impl GitOps {
    #[instrument(skip(target_dir), fields(repo_url, target_dir = %target_dir.display(), ssh_key = %ssh_key_path.display()))]
    pub fn clone(repo_url: &str, target_dir: &Path, ssh_key_path: &Path) -> Result<Self> {
        info!("Starting git clone");

        let ssh_cmd = format!(
            "ssh -i {} -o StrictHostKeyChecking=no",
            ssh_key_path.display()
        );
        debug!(ssh_command = %ssh_cmd, "SSH command configured");

        let start = std::time::Instant::now();
        let output = Command::new("git")
            .env("GIT_SSH_COMMAND", &ssh_cmd)
            .args(["clone", repo_url, target_dir.to_str().unwrap()])
            .output()
            .context("Failed to execute git clone")?;

        let elapsed = start.elapsed();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!(
                exit_code = ?output.status.code(),
                stderr = %stderr,
                elapsed_ms = elapsed.as_millis(),
                "Git clone failed"
            );
            anyhow::bail!("Git clone failed: {}", stderr);
        }

        info!(
            elapsed_ms = elapsed.as_millis(),
            stdout_lines = String::from_utf8_lossy(&output.stdout).lines().count(),
            "Git clone completed successfully"
        );

        Ok(Self {
            repo_path: target_dir.to_path_buf(),
        })
    }

    #[instrument(skip(self), fields(branch_name, repo_path = %self.repo_path.display()))]
    pub fn create_branch(&self, branch_name: &str) -> Result<()> {
        info!("Creating and checking out branch");

        let start = std::time::Instant::now();
        let output = Command::new("git")
            .args(["checkout", "-b", branch_name])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to create branch")?;

        let elapsed = start.elapsed();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!(
                exit_code = ?output.status.code(),
                stderr = %stderr,
                elapsed_ms = elapsed.as_millis(),
                "Git checkout -b failed"
            );
            anyhow::bail!("Git checkout -b failed: {}", stderr);
        }

        info!(
            elapsed_ms = elapsed.as_millis(),
            "Branch created and checked out"
        );
        Ok(())
    }

    #[instrument(skip(self), fields(repo_path = %self.repo_path.display()))]
    pub fn add_all(&self) -> Result<()> {
        debug!("Staging all changes");

        let start = std::time::Instant::now();
        let output = Command::new("git")
            .args(["add", "."])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to git add")?;

        let elapsed = start.elapsed();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!(
                exit_code = ?output.status.code(),
                stderr = %stderr,
                elapsed_ms = elapsed.as_millis(),
                "Git add failed"
            );
            anyhow::bail!("Git add failed: {}", stderr);
        }

        debug!(elapsed_ms = elapsed.as_millis(), "All changes staged");
        Ok(())
    }

    #[instrument(skip(self), fields(message, repo_path = %self.repo_path.display()))]
    pub fn commit(&self, message: &str) -> Result<()> {
        info!("Creating commit");

        let start = std::time::Instant::now();
        let output = Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to commit")?;

        let elapsed = start.elapsed();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("nothing to commit") {
                warn!(
                    elapsed_ms = elapsed.as_millis(),
                    "Nothing to commit, working tree clean"
                );
                return Ok(());
            }
            error!(
                exit_code = ?output.status.code(),
                stderr = %stderr,
                elapsed_ms = elapsed.as_millis(),
                "Git commit failed"
            );
            anyhow::bail!("Git commit failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let commit_hash = stdout
            .lines()
            .find(|line| line.contains("[") && line.contains("]"))
            .and_then(|line| line.split_whitespace().nth(1))
            .map(|s| s.trim_end_matches(']').to_string());

        info!(
            elapsed_ms = elapsed.as_millis(),
            commit_hash = ?commit_hash,
            "Commit created successfully"
        );
        Ok(())
    }

    #[instrument(skip(self, ssh_key_path), fields(branch_name, repo_path = %self.repo_path.display(), ssh_key = %ssh_key_path.display()))]
    pub fn push(&self, branch_name: &str, ssh_key_path: &Path) -> Result<()> {
        info!("Pushing to remote");

        let ssh_cmd = format!(
            "ssh -i {} -o StrictHostKeyChecking=no",
            ssh_key_path.display()
        );
        debug!(ssh_command = %ssh_cmd, "SSH command configured");

        let start = std::time::Instant::now();
        let output = Command::new("git")
            .env("GIT_SSH_COMMAND", &ssh_cmd)
            .args(["push", "-u", "origin", branch_name])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to push")?;

        let elapsed = start.elapsed();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!(
                exit_code = ?output.status.code(),
                stderr = %stderr,
                elapsed_ms = elapsed.as_millis(),
                "Git push failed"
            );
            anyhow::bail!("Git push failed: {}", stderr);
        }

        info!(
            elapsed_ms = elapsed.as_millis(),
            remote = "origin",
            "Push completed successfully"
        );
        Ok(())
    }
}
