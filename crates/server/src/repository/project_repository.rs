use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use sea_orm::*;
use uuid::Uuid;

use parallel_protocol::{Project, RepoConfig, SshKeyConfig};

use crate::db::entity::projects;
use crate::errors::{ServerError, ServerResult};

pub struct ProjectRepository {
    db: DatabaseConnection,
}

impl ProjectRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

fn model_to_project(p: projects::Model) -> Project {
    let repos: Vec<RepoConfig> = serde_json::from_str(&p.repos_json).unwrap_or_default();
    let ssh_keys: Vec<SshKeyConfig> = serde_json::from_str(&p.ssh_keys_json).unwrap_or_default();
    
    Project {
        id: p.id,
        name: p.name,
        repos,
        ssh_keys,
        created_at: p.created_at,
        updated_at: p.updated_at,
    }
}

#[async_trait]
pub trait ProjectRepositoryTrait: Send + Sync {
    async fn create(
        &self,
        id: Uuid,
        name: String,
        repos: &Vec<RepoConfig>,
        ssh_keys: &Vec<SshKeyConfig>,
    ) -> Result<()>;

    async fn find_by_id(&self, project_id: &Uuid) -> ServerResult<Project>;

    async fn find_many(
        &self,
        search: Option<&str>,
        sort_direction: &str,
        limit: u64,
    ) -> Result<Vec<Project>>;

    async fn count_all(&self) -> Result<u64>;

    async fn update(
        &self,
        project_id: &Uuid,
        name: Option<String>,
        repos: Option<&Vec<RepoConfig>>,
        ssh_keys: Option<&Vec<SshKeyConfig>>,
    ) -> ServerResult<Project>;

    async fn delete(&self, project_id: &Uuid) -> ServerResult<()>;
}

#[async_trait]
impl ProjectRepositoryTrait for ProjectRepository {
    async fn create(
        &self,
        id: Uuid,
        name: String,
        repos: &Vec<RepoConfig>,
        ssh_keys: &Vec<SshKeyConfig>,
    ) -> Result<()> {
        let now = Utc::now();

        let project = projects::ActiveModel {
            id: Set(id),
            name: Set(name),
            repos_json: Set(serde_json::to_string(repos)?),
            ssh_keys_json: Set(serde_json::to_string(ssh_keys)?),
            created_at: Set(now),
            updated_at: Set(now),
        };

        projects::Entity::insert(project).exec(&self.db).await?;
        Ok(())
    }

    async fn find_by_id(&self, project_id: &Uuid) -> ServerResult<Project> {
        let project = projects::Entity::find_by_id(*project_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| ServerError::ProjectNotFound(*project_id))?;

        Ok(model_to_project(project))
    }

    async fn find_many(
        &self,
        search: Option<&str>,
        sort_direction: &str,
        limit: u64,
    ) -> Result<Vec<Project>> {
        let mut query = projects::Entity::find();

        if let Some(s) = search {
            let pattern = format!("%{}%", s);
            query = query.filter(projects::Column::Name.like(&pattern));
        }

        if sort_direction == "desc" {
            query = query.order_by_desc(projects::Column::CreatedAt);
        } else {
            query = query.order_by_asc(projects::Column::CreatedAt);
        }

        let db_projects = query.limit(limit).all(&self.db).await?;

        Ok(db_projects
            .into_iter()
            .map(model_to_project)
            .collect())
    }

    async fn count_all(&self) -> Result<u64> {
        Ok(projects::Entity::find().count(&self.db).await?)
    }

    async fn update(
        &self,
        project_id: &Uuid,
        name: Option<String>,
        repos: Option<&Vec<RepoConfig>>,
        ssh_keys: Option<&Vec<SshKeyConfig>>,
    ) -> ServerResult<Project> {
        let now = Utc::now();
        let project = projects::Entity::find_by_id(*project_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| ServerError::ProjectNotFound(*project_id))?;

        let mut project: projects::ActiveModel = project.into();
        if let Some(n) = name {
            project.name = Set(n);
        }
        if let Some(r) = repos {
            project.repos_json = Set(serde_json::to_string(r)?);
        }
        if let Some(k) = ssh_keys {
            project.ssh_keys_json = Set(serde_json::to_string(k)?);
        }
        project.updated_at = Set(now);
        
        let updated = project.update(&self.db).await?;
        Ok(model_to_project(updated))
    }

    async fn delete(&self, project_id: &Uuid) -> ServerResult<()> {
        let project = projects::Entity::find_by_id(*project_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| ServerError::ProjectNotFound(*project_id))?;

        let project: projects::ActiveModel = project.into();
        project.delete(&self.db).await?;
        Ok(())
    }
}
