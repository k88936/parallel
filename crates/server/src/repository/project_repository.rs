use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::sqlite::SqliteConnection;
use uuid::Uuid;

use parallel_common::{Project, RepoConfig, SshKeyConfig};

use super::task_repository::DbPool;
use crate::db::entity::{NewProject, Project as DbProject};
use crate::db::schema::projects as projects_schema;
use crate::errors::{ServerError, ServerResult};

pub struct ProjectRepository {
    pool: DbPool,
}

impl ProjectRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    fn get_conn(&self) -> ServerResult<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>> {
        self.pool.get().map_err(|e| ServerError::InternalError(e.to_string()))
    }
}

fn db_project_to_project(p: DbProject) -> Project {
    let repos: Vec<RepoConfig> = serde_json::from_str(&p.repos_json).unwrap_or_default();
    let ssh_keys: Vec<SshKeyConfig> = serde_json::from_str(&p.ssh_keys_json).unwrap_or_default();
    
    Project {
        id: p.get_uuid(),
        name: p.name,
        repos,
        ssh_keys,
        created_at: chrono::DateTime::from_naive_utc_and_offset(p.created_at, Utc),
        updated_at: chrono::DateTime::from_naive_utc_and_offset(p.updated_at, Utc),
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
        let now = Utc::now().naive_utc();

        let new_project = NewProject {
            id: id.to_string(),
            name,
            repos_json: serde_json::to_string(repos)?,
            ssh_keys_json: serde_json::to_string(ssh_keys)?,
            created_at: now,
            updated_at: now,
        };

        let mut conn = self.get_conn()?;
        diesel::insert_into(projects_schema::table)
            .values(&new_project)
            .execute(&mut conn)?;

        Ok(())
    }

    async fn find_by_id(&self, project_id: &Uuid) -> ServerResult<Project> {
        let mut conn = self.get_conn()?;
        let project = projects_schema::table
            .filter(projects_schema::id.eq(project_id.to_string()))
            .first::<DbProject>(&mut conn)
            .map_err(|_| ServerError::ProjectNotFound(*project_id))?;

        Ok(db_project_to_project(project))
    }

    async fn find_many(
        &self,
        search: Option<&str>,
        sort_direction: &str,
        limit: u64,
    ) -> Result<Vec<Project>> {
        let mut conn = self.get_conn()?;
        let pattern = search.map(|s| format!("%{}%", s));
        let mut query = projects_schema::table.into_boxed();

        if let Some(ref pat) = pattern {
            query = query.filter(projects_schema::name.like(pat));
        }

        if sort_direction == "desc" {
            query = query.order_by(projects_schema::created_at.desc());
        } else {
            query = query.order_by(projects_schema::created_at.asc());
        }

        let db_projects = query
            .limit(limit as i64)
            .load::<DbProject>(&mut conn)?;

        Ok(db_projects
            .into_iter()
            .map(db_project_to_project)
            .collect())
    }

    async fn count_all(&self) -> Result<u64> {
        let mut conn = self.get_conn()?;
        let count = projects_schema::table
            .count()
            .get_result::<i64>(&mut conn)? as u64;

        Ok(count)
    }

    async fn update(
        &self,
        project_id: &Uuid,
        name: Option<String>,
        repos: Option<&Vec<RepoConfig>>,
        ssh_keys: Option<&Vec<SshKeyConfig>>,
    ) -> ServerResult<Project> {
        let now = Utc::now().naive_utc();

        let mut conn = self.get_conn()?;
        
        let project = projects_schema::table
            .filter(projects_schema::id.eq(project_id.to_string()))
            .first::<DbProject>(&mut conn)
            .map_err(|_| ServerError::ProjectNotFound(*project_id))?;

        let new_name = name.unwrap_or(project.name);
        let new_repos_json = match repos {
            Some(r) => serde_json::to_string(r)?,
            None => project.repos_json,
        };
        let new_ssh_keys_json = match ssh_keys {
            Some(k) => serde_json::to_string(k)?,
            None => project.ssh_keys_json,
        };

        diesel::update(projects_schema::table)
            .filter(projects_schema::id.eq(project_id.to_string()))
            .set((
                projects_schema::name.eq(new_name),
                projects_schema::repos_json.eq(new_repos_json),
                projects_schema::ssh_keys_json.eq(new_ssh_keys_json),
                projects_schema::updated_at.eq(now),
            ))
            .execute(&mut conn)?;

        let project = projects_schema::table
            .filter(projects_schema::id.eq(project_id.to_string()))
            .first::<DbProject>(&mut conn)?;

        Ok(db_project_to_project(project))
    }

    async fn delete(&self, project_id: &Uuid) -> ServerResult<()> {
        let mut conn = self.get_conn()?;
        let rows_affected = diesel::delete(
            projects_schema::table.filter(projects_schema::id.eq(project_id.to_string()))
        )
        .execute(&mut conn)?;

        if rows_affected == 0 {
            return Err(ServerError::ProjectNotFound(*project_id));
        }

        Ok(())
    }
}
