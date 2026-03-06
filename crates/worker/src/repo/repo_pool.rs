use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;
use crate::repo::repo_ops::GitOps;

struct RepoSlot {
    slot_id: u32,
    path: PathBuf,
    in_use_by: Option<Uuid>,
}

pub struct RepoPool {
    base_path: PathBuf,
    slots: Arc<RwLock<HashMap<String, Vec<RepoSlot>>>>,
}

impl RepoPool {
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            base_path,
            slots: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn hash_repo_url(repo_url: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(repo_url.as_bytes());
        format!("{:x}", hasher.finalize())[..16].to_string()
    }

    pub async fn acquire_slot(
        &self,
        repo_url: &str,
        task_id: Uuid,
        base_branch: &str,
        target_branch: &str,
        ssh_key: &str,
    ) -> Result<PathBuf> {
        let repo_hash = Self::hash_repo_url(repo_url);
        let repo_dir = self.base_path.join(&repo_hash);

        let mut slots = self.slots.write().await;

        let repo_slots = slots.entry(repo_hash.clone()).or_insert_with(Vec::new);

        if let Some(available_slot) = repo_slots.iter_mut().find(|s| s.in_use_by.is_none()) {
            available_slot.in_use_by = Some(task_id);
            let slot_path = available_slot.path.clone();
            let slot_id = available_slot.slot_id;

            drop(slots);

            info!(
                "Reusing slot {} for repo {}, task {}",
                slot_id, repo_hash, task_id
            );

            self.prepare_existing_slot(&slot_path, base_branch, target_branch, ssh_key)
                .await?;

            return Ok(slot_path);
        }

        let slot_id = repo_slots.len() as u32;
        let slot_path = repo_dir.join(slot_id.to_string());

        let slot_exists = slot_path.exists() && slot_path.join(".git").exists();

        drop(slots);

        if slot_exists {
            info!(
                "Reusing existing slot {} on disk for repo {}, task {}",
                slot_id, repo_hash, task_id
            );

            self.prepare_existing_slot(&slot_path, base_branch, target_branch, ssh_key)
                .await?;
        } else {
            info!(
                "Creating new slot {} for repo {}, task {}",
                slot_id, repo_hash, task_id
            );

            tokio::fs::create_dir_all(&repo_dir)
                .await
                .context("Failed to create repo directory")?;

            GitOps::clone(repo_url, base_branch, &slot_path, ssh_key)?;

            let git = GitOps::open(&slot_path)?;
            git.create_branch(target_branch)?;
        }

        let mut slots = self.slots.write().await;
        let repo_slots = slots.get_mut(&repo_hash).context("Repo slots not found")?;
        repo_slots.push(RepoSlot {
            slot_id,
            path: slot_path.clone(),
            in_use_by: Some(task_id),
        });

        Ok(slot_path)
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

        git.force_checkout(base_branch)?;

        git.delete_branch_if_exists(target_branch)?;

        git.create_branch(target_branch)?;

        Ok(())
    }

    pub async fn release_slot(&self, repo_url: &str, task_id: Uuid) -> Result<()> {
        let repo_hash = Self::hash_repo_url(repo_url);

        let mut slots = self.slots.write().await;

        if let Some(repo_slots) = slots.get_mut(&repo_hash) {
            if let Some(slot) = repo_slots.iter_mut().find(|s| s.in_use_by == Some(task_id)) {
                slot.in_use_by = None;
                info!("Released slot {} for task {}", slot.slot_id, task_id);
            }
        }

        Ok(())
    }
}
