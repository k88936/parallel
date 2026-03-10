use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;
use xtra::Actor;

use crate::repo::repo_ops::GitOps;

struct RepoSlot {
    slot_id: u32,
    path: PathBuf,
    in_use_by: Option<Uuid>,
}

pub struct AcquireSlot {
    pub repo_url: String,
    pub task_id: Uuid,
    pub base_branch: String,
    pub target_branch: String,
    pub ssh_key: String,
}

pub struct ReleaseSlot {
    pub repo_url: String,
    pub task_id: Uuid,
}

pub struct RepoPoolActor {
    base_path: PathBuf,
    slots: HashMap<String, Vec<RepoSlot>>,
}

impl RepoPoolActor {
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            base_path,
            slots: HashMap::new(),
        }
    }

    fn hash_repo_url(repo_url: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(repo_url.as_bytes());
        format!("{:x}", hasher.finalize())[..16].to_string()
    }

    async fn prepare_existing_slot(
        &self,
        slot_path: &Path,
        base_branch: &str,
        target_branch: &str,
        ssh_key: &str,
    ) -> Result<()> {
        let git = GitOps::open(slot_path)?;
        git.fetch(ssh_key)?;

        Command::new("git")
            .current_dir(slot_path)
            .args(&["clean", "-fd"])
            .output()
            .context("Failed cleaning up repo")?;

        Command::new("git")
            .current_dir(slot_path)
            .args(&[
                "checkout",
                "-B",
                target_branch,
                &format!("origin/{}", base_branch),
                "--force",
            ])
            .output()
            .context("Failed checkout to base")?;

        Ok(())
    }
}

impl Actor for RepoPoolActor {
    type Stop = ();

    async fn stopped(self) -> Self::Stop {}
}

impl xtra::Handler<AcquireSlot> for RepoPoolActor {
    type Return = Result<PathBuf>;

    async fn handle(&mut self, msg: AcquireSlot, _ctx: &mut xtra::Context<Self>) -> Self::Return {
        let repo_hash = Self::hash_repo_url(&msg.repo_url);
        let repo_dir = self.base_path.join(&repo_hash);

        if let Some(available_slot) = self
            .slots
            .entry(repo_hash.clone())
            .or_insert_with(Vec::new)
            .iter_mut()
            .find(|s| s.in_use_by.is_none())
        {
            available_slot.in_use_by = Some(msg.task_id);
            let slot_path = available_slot.path.clone();
            let slot_id = available_slot.slot_id;

            tracing::info!(
                "Reusing slot {} for repo {}, task {}",
                slot_id,
                repo_hash,
                msg.task_id
            );

            self.prepare_existing_slot(&slot_path, &msg.base_branch, &msg.target_branch, &msg.ssh_key)
                .await?;

            return Ok(slot_path);
        }

        let repo_slots = self.slots.entry(repo_hash.clone()).or_insert_with(Vec::new);
        let slot_id = repo_slots.len() as u32;
        let slot_path = repo_dir.join(slot_id.to_string());

        let slot_exists = slot_path.exists() && slot_path.join(".git").exists();

        if slot_exists {
            tracing::info!(
                "Reusing existing slot {} on disk for repo {}, task {}",
                slot_id,
                repo_hash,
                msg.task_id
            );

            self.prepare_existing_slot(&slot_path, &msg.base_branch, &msg.target_branch, &msg.ssh_key)
                .await?;
        } else {
            tracing::info!(
                "Creating new slot {} for repo {}, task {}",
                slot_id,
                repo_hash,
                msg.task_id
            );

            tokio::fs::create_dir_all(&repo_dir)
                .await
                .context("Failed to create repo directory")?;

            GitOps::clone(&msg.repo_url, &msg.base_branch, &slot_path, &msg.ssh_key)?;

            let git = GitOps::open(&slot_path)?;
            git.create_branch(&msg.target_branch)?;
        }

        let repo_slots = self.slots.get_mut(&repo_hash).context("Repo slots not found")?;
        repo_slots.push(RepoSlot {
            slot_id,
            path: slot_path.clone(),
            in_use_by: Some(msg.task_id),
        });

        Ok(slot_path)
    }
}

impl xtra::Handler<ReleaseSlot> for RepoPoolActor {
    type Return = Result<()>;

    async fn handle(&mut self, msg: ReleaseSlot, _ctx: &mut xtra::Context<Self>) -> Self::Return {
        let repo_hash = Self::hash_repo_url(&msg.repo_url);

        if let Some(repo_slots) = self.slots.get_mut(&repo_hash) {
            if let Some(slot) = repo_slots.iter_mut().find(|s| s.in_use_by == Some(msg.task_id)) {
                slot.in_use_by = None;
                tracing::info!("Released slot {} for task {}", slot.slot_id, msg.task_id);
            }
        }

        Ok(())
    }
}
