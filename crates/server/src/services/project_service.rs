use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use sea_orm::*;
use uuid::Uuid;

use parallel_protocol::{Project, RepoConfig, SshKeyConfig};

use crate::db::entity::projects;
use crate::errors::{ServerError, ServerResult};
use crate::services::traits::{ProjectListParams, ProjectListResult, ProjectServiceTrait};

pub struct ProjectService {
    db: DatabaseConnection,
}

impl ProjectService {
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
impl ProjectServiceTrait for ProjectService {
    async fn create(
        &self,
        name: String,
        repos: Vec<RepoConfig>,
        ssh_keys: Vec<SshKeyConfig>,
    ) -> Result<Uuid> {
        let project_id = Uuid::new_v4();
        let now = Utc::now();

        let project = projects::ActiveModel {
            id: Set(project_id),
            name: Set(name),
            repos_json: Set(serde_json::to_string(&repos)?),
            ssh_keys_json: Set(serde_json::to_string(&ssh_keys)?),
            created_at: Set(now),
            updated_at: Set(now),
        };

        projects::Entity::insert(project).exec(&self.db).await?;

        Ok(project_id)
    }

    async fn get(&self, project_id: &Uuid) -> ServerResult<Project> {
        let project = projects::Entity::find_by_id(*project_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| ServerError::ProjectNotFound(*project_id))?;

        Ok(model_to_project(project))
    }

    async fn list(&self, params: ProjectListParams) -> Result<ProjectListResult> {
        let limit = params.limit.unwrap_or(50);
        let fetch_limit = limit + 1;

        let mut query = projects::Entity::find();

        if let Some(ref search) = params.search {
            let pattern = format!("%{}%", search);
            query = query.filter(projects::Column::Name.like(&pattern));
        }

        let sort_direction = params.sort_direction.as_deref().unwrap_or("desc");
        if sort_direction == "desc" {
            query = query.order_by_desc(projects::Column::CreatedAt);
        } else {
            query = query.order_by_asc(projects::Column::CreatedAt);
        }

        let db_projects = query.limit(fetch_limit).all(&self.db).await?;

        let has_more = db_projects.len() > limit as usize;
        let projects: Vec<Project> = db_projects
            .into_iter()
            .take(limit as usize)
            .map(model_to_project)
            .collect();

        let total = projects::Entity::find().count(&self.db).await?;

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
            project.repos_json = Set(serde_json::to_string(&r)?);
        }
        if let Some(k) = ssh_keys {
            project.ssh_keys_json = Set(serde_json::to_string(&k)?);
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

    async fn get_repo(&self, project_id: &Uuid, repo_name: &str) -> ServerResult<Option<RepoConfig>> {
        let project = self.get(project_id).await?;
        Ok(project.repos.into_iter().find(|r| r.name == repo_name))
    }

    async fn get_ssh_key(&self, project_id: &Uuid, key_name: &str) -> ServerResult<Option<SshKeyConfig>> {
        let project = self.get(project_id).await?;
        Ok(project.ssh_keys.into_iter().find(|k| k.name == key_name))
    }
}
