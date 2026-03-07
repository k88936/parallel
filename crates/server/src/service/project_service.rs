use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use parallel_protocol::{Project, RepoConfig, SshKeyConfig};

use crate::errors::ServerResult;
use crate::repository::{ProjectRepository, ProjectRepositoryTrait};
pub struct ProjectService {
    repository: Arc<ProjectRepository>,
}

impl ProjectService {
    pub fn new(repository: Arc<ProjectRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl ProjectServiceTrait for ProjectService {
    async fn create(
        &self,
        name: String,
        repos: Vec<RepoConfig>,
        ssh_keys: Vec<SshKeyConfig>,
    ) -> Result<Uuid> {
        let project_id = Uuid::new_v4();

        self.repository.create(
            project_id,
            name,
            &repos,
            &ssh_keys,
        ).await?;

        Ok(project_id)
    }

    async fn get(&self, project_id: &Uuid) -> ServerResult<Project> {
        self.repository.find_by_id(project_id).await
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
        project_id: &Uuid,
        name: Option<String>,
        repos: Option<Vec<RepoConfig>>,
        ssh_keys: Option<Vec<SshKeyConfig>>,
    ) -> ServerResult<Project> {
        self.repository.update(
            project_id,
            name,
            repos.as_ref(),
            ssh_keys.as_ref(),
        ).await
    }

    async fn delete(&self, project_id: &Uuid) -> ServerResult<()> {
        self.repository.delete(project_id).await
    }

    async fn get_repo(&self, project_id: &Uuid, repo_name: &str) -> ServerResult<Option<RepoConfig>> {
        let project = self.repository.find_by_id(project_id).await?;
        Ok(project.repos.into_iter().find(|r| r.name == repo_name))
    }

    async fn get_ssh_key(&self, project_id: &Uuid, key_name: &str) -> ServerResult<Option<SshKeyConfig>> {
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
    ) -> Result<Uuid>;

    async fn get(&self, project_id: &Uuid) -> ServerResult<Project>;

    async fn list(&self, params: ProjectListParams) -> Result<ProjectListResult>;

    async fn update(
        &self,
        project_id: &Uuid,
        name: Option<String>,
        repos: Option<Vec<RepoConfig>>,
        ssh_keys: Option<Vec<SshKeyConfig>>,
    ) -> ServerResult<Project>;

    async fn delete(&self, project_id: &Uuid) -> ServerResult<()>;

    async fn get_repo(&self, project_id: &Uuid, repo_name: &str) -> ServerResult<Option<RepoConfig>>;

    async fn get_ssh_key(&self, project_id: &Uuid, key_name: &str) -> ServerResult<Option<SshKeyConfig>>;
}