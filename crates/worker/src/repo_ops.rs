use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

pub struct GitOps {
    pub repo_path: std::path::PathBuf,
}

impl GitOps {
    pub fn clone(
        repo_url: &str,
        base_branch: &str,
        target_dir: &Path,
        ssh_key_path: &Path,
    ) -> Result<Self> {
        let ssh_cmd = format!(
            "ssh -i {} -o StrictHostKeyChecking=no",
            ssh_key_path.display()
        );

        let output = Command::new("git")
            .env("GIT_SSH_COMMAND", &ssh_cmd)
            .args([
                "clone",
                "-b",
                base_branch,
                "--single-branch",
                repo_url,
                target_dir.to_str().unwrap(),
            ])
            .output()
            .context("Failed to execute git clone")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Git clone failed: {}", stderr);
        }

        Ok(Self {
            repo_path: target_dir.to_path_buf(),
        })
    }

    pub fn create_branch(&self, branch_name: &str) -> Result<()> {
        let output = Command::new("git")
            .args(["checkout", "-b", branch_name])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to create branch")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Git checkout -b failed: {}", stderr);
        }

        Ok(())
    }

    pub fn add_all(&self) -> Result<()> {
        let output = Command::new("git")
            .args(["add", "."])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to git add")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Git add failed: {}", stderr);
        }

        Ok(())
    }

    pub fn commit(&self, message: &str) -> Result<()> {
        let output = Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to commit")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("nothing to commit") {
                return Ok(());
            }
            anyhow::bail!("Git commit failed: {}", stderr);
        }

        Ok(())
    }

    pub fn push(&self, branch_name: &str, ssh_key_path: &Path) -> Result<()> {
        let ssh_cmd = format!(
            "ssh -i {} -o StrictHostKeyChecking=no",
            ssh_key_path.display()
        );

        let output = Command::new("git")
            .env("GIT_SSH_COMMAND", &ssh_cmd)
            .args(["push", "-u", "origin", branch_name])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to push")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Git push failed: {}", stderr);
        }

        Ok(())
    }

    pub fn diff(&self) -> Result<String> {
        let output = Command::new("git")
            .args(["diff", "HEAD"])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to get git diff")?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    pub fn fetch(&self, ssh_key_path: &Path) -> Result<()> {
        let ssh_cmd = format!(
            "ssh -i {} -o StrictHostKeyChecking=no",
            ssh_key_path.display()
        );

        let output = Command::new("git")
            .env("GIT_SSH_COMMAND", &ssh_cmd)
            .args(["fetch", "--all"])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to execute git fetch")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("Git fetch warning: {}", stderr);
        }

        Ok(())
    }

    pub fn force_checkout(&self, branch: &str) -> Result<()> {
        let output = Command::new("git")
            .args(["checkout", "-f", branch])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to force checkout")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let output_reset = Command::new("git")
                .args(["reset", "--hard", &format!("origin/{}", branch)])
                .current_dir(&self.repo_path)
                .output()
                .context("Failed to reset")?;

            if !output_reset.status.success() {
                let stderr_reset = String::from_utf8_lossy(&output_reset.stderr);
                anyhow::bail!(
                    "Git checkout and reset failed: {} / {}",
                    stderr,
                    stderr_reset
                );
            }
        }

        Ok(())
    }

    pub fn delete_branch_if_exists(&self, branch: &str) -> Result<()> {
        let output = Command::new("git")
            .args(["branch", "-D", branch])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to delete branch")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.contains("not found") && !stderr.contains("does not exist") {
                tracing::debug!("Branch delete info: {}", stderr);
            }
        }

        Ok(())
    }
}
