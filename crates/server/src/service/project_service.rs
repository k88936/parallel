use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use parallel_common::{is_root_project_id, Project, RepoConfig, SshKeyConfig, ROOT_PROJECT_ID};

use crate::errors::{ServerError, ServerResult};
use crate::repository::{ProjectRepository, ProjectRepositoryTrait};

pub struct ProjectService {
    repository: Arc<ProjectRepository>,
}

impl ProjectService {
    pub fn new(repository: Arc<ProjectRepository>) -> Self {
        Self { repository }
    }
}

fn generate_project_id() -> String {
    Uuid::new_v4().to_string()
}

#[async_trait]
impl ProjectServiceTrait for ProjectService {
    async fn create(
        &self,
        name: String,
        repos: Vec<RepoConfig>,
        ssh_keys: Vec<SshKeyConfig>,
        parent_id: Option<String>,
    ) -> Result<String> {
        let parent = match parent_id.as_ref() {
            Some(pid) => {
                if is_root_project_id(pid) {
                    None
                } else {
                    Some(self.repository.find_by_id(pid).await?)
                }
            }
            None => None,
        };

        let project_id = generate_project_id();
        let effective_parent_id = parent.map(|p| p.id).or(parent_id);

        self.repository.create(
            project_id.clone(),
            name,
            &repos,
            &ssh_keys,
            effective_parent_id,
        ).await?;

        Ok(project_id)
    }

    async fn get(&self, project_id: &str) -> ServerResult<Project> {
        self.repository.find_by_id(project_id).await
    }

    async fn get_root(&self) -> ServerResult<Project> {
        self.repository.find_by_id(ROOT_PROJECT_ID).await
    }

    async fn list(&self, params: ProjectListParams) -> Result<ProjectListResult> {
        let limit = params.limit.unwrap_or(50);
        let fetch_limit = limit + 1;
        let sort_direction = params.sort_direction.as_deref().unwrap_or("desc");

        let projects = self.repository.find_many(
            params.search.as_deref(),
            sort_direction,
            fetch_limit,
        ).await?;

        let has_more = projects.len() > limit as usize;
        let projects: Vec<Project> = projects
            .into_iter()
            .take(limit as usize)
            .collect();

        let total = self.repository.count_all().await?;

        Ok(ProjectListResult {
            projects,
            total,
            has_more,
        })
    }

    async fn update(
        &self,
        project_id: &str,
        name: Option<String>,
        repos: Option<Vec<RepoConfig>>,
        ssh_keys: Option<Vec<SshKeyConfig>>,
        parent_id: Option<Option<String>>,
    ) -> ServerResult<Project> {
        if is_root_project_id(project_id) && parent_id.is_some() {
            return Err(ServerError::InvalidOperation("Cannot change parent of root project".to_string()));
        }

        if let Some(Some(pid)) = parent_id.as_ref() {
            if pid == project_id {
                return Err(ServerError::InvalidOperation("Project cannot be its own parent".to_string()));
            }
            if !is_root_project_id(pid) {
                self.repository.find_by_id(pid).await?;
            }
        }

        self.repository.update(
            project_id,
            name,
            repos.as_ref(),
            ssh_keys.as_ref(),
            parent_id,
        ).await
    }

    async fn delete(&self, project_id: &str) -> ServerResult<()> {
        if is_root_project_id(project_id) {
            return Err(ServerError::InvalidOperation("Cannot delete root project".to_string()));
        }

        let children = self.repository.find_children(project_id).await
            .map_err(|e| ServerError::DatabaseError(e.to_string()))?;
        
        if !children.is_empty() {
            return Err(ServerError::InvalidOperation("Cannot delete project with children. Delete or move children first.".to_string()));
        }

        self.repository.delete(project_id).await
    }

    async fn get_children(&self, project_id: &str) -> Result<Vec<Project>> {
        self.repository.find_children(project_id).await
    }

    async fn get_repo(&self, project_id: &str, repo_name: &str) -> ServerResult<Option<RepoConfig>> {
        let project = self.repository.find_by_id(project_id).await?;
        Ok(project.repos.into_iter().find(|r| r.name == repo_name))
    }

    async fn get_ssh_key(&self, project_id: &str, key_name: &str) -> ServerResult<Option<SshKeyConfig>> {
        let project = self.repository.find_by_id(project_id).await?;
        Ok(project.ssh_keys.into_iter().find(|k| k.name == key_name))
    }
}

pub struct ProjectListParams {
    pub search: Option<String>,
    pub sort_direction: Option<String>,
    pub limit: Option<u64>,
}

pub struct ProjectListResult {
    pub projects: Vec<Project>,
    pub total: u64,
    pub has_more: bool,
}

#[async_trait]
pub trait ProjectServiceTrait: Send + Sync {
    async fn create(
        &self,
        name: String,
        repos: Vec<RepoConfig>,
        ssh_keys: Vec<SshKeyConfig>,
        parent_id: Option<String>,
    ) -> Result<String>;

    async fn get(&self, project_id: &str) -> ServerResult<Project>;

    async fn get_root(&self) -> ServerResult<Project>;

    async fn list(&self, params: ProjectListParams) -> Result<ProjectListResult>;

    async fn update(
        &self,
        project_id: &str,
        name: Option<String>,
        repos: Option<Vec<RepoConfig>>,
        ssh_keys: Option<Vec<SshKeyConfig>>,
        parent_id: Option<Option<String>>,
    ) -> ServerResult<Project>;

    async fn delete(&self, project_id: &str) -> ServerResult<()>;

    async fn get_children(&self, project_id: &str) -> Result<Vec<Project>>;

    async fn get_repo(&self, project_id: &str, repo_name: &str) -> ServerResult<Option<RepoConfig>>;

    async fn get_ssh_key(&self, project_id: &str, key_name: &str) -> ServerResult<Option<SshKeyConfig>>;
}