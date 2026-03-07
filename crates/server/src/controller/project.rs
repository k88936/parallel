use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    Extension,
};
use tower_http::request_id::RequestId;
use uuid::Uuid;

use parallel_common::*;

use crate::api_error::{ApiResult, ErrorResponse};
use crate::error_codes::ErrorCode;
use crate::errors::ServerError;
use crate::service::project_service::ProjectListParams;
use crate::state::AppState;

pub async fn create_project(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<CreateProjectRequest>,
) -> ApiResult<Json<CreateProjectResponse>> {
    let correlation_id = request_id
        .header_value()
        .to_str()
        .ok()
        .and_then(|s| Uuid::parse_str(s).ok());

    tracing::info!(
        correlation_id = ?correlation_id,
        name = %payload.name,
        "Creating project"
    );

    let project_id = state
        .project_service
        .create(payload.name, payload.repos, payload.ssh_keys)
        .await
        .map_err(|e| {
            tracing::error!(
                correlation_id = ?correlation_id,
                error = %e,
                "Failed to create project"
            );
            ErrorResponse::new(ErrorCode::InternalError, "Failed to create project")
                .with_details(e.to_string())
                .with_correlation_id(correlation_id.unwrap_or_default())
        })?;

    tracing::info!(
        correlation_id = ?correlation_id,
        project_id = %project_id,
        "Project created successfully"
    );

    Ok(Json(CreateProjectResponse { project_id }))
}

pub async fn list_projects(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Query(query): Query<ListProjectsQuery>,
) -> ApiResult<Json<ProjectListResponse>> {
    let correlation_id = request_id
        .header_value()
        .to_str()
        .ok()
        .and_then(|s| Uuid::parse_str(s).ok());

    let params = ProjectListParams {
        search: query.search,
        sort_direction: query.sort_direction,
        limit: query.limit.map(|l| l as u64),
    };

    let result = state.project_service.list(params).await.map_err(|e| {
        tracing::error!(
            correlation_id = ?correlation_id,
            error = %e,
            "Failed to list projects"
        );
        ErrorResponse::from(ServerError::DatabaseError(e.to_string()))
            .with_correlation_id(correlation_id.unwrap_or_default())
    })?;

    Ok(Json(ProjectListResponse {
        projects: result.projects,
        total: result.total,
        has_more: result.has_more,
    }))
}

pub async fn get_project(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Path(project_id): Path<Uuid>,
) -> ApiResult<Json<Project>> {
    let correlation_id = request_id
        .header_value()
        .to_str()
        .ok()
        .and_then(|s| Uuid::parse_str(s).ok());

    let project = state.project_service.get(&project_id).await.map_err(|e| {
        tracing::error!(
            correlation_id = ?correlation_id,
            project_id = %project_id,
            error = %e,
            "Failed to get project"
        );

        let error_response = match e {
            ServerError::ProjectNotFound(id) => ErrorResponse::new(
                ErrorCode::InternalError,
                format!("Project with ID {} not found", id),
            )
            .with_metadata("project_id", serde_json::json!(id)),
            other => ErrorResponse::from(other),
        };

        error_response.with_correlation_id(correlation_id.unwrap_or_default())
    })?;

    Ok(Json(project))
}

pub async fn update_project(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Path(project_id): Path<Uuid>,
    Json(payload): Json<UpdateProjectRequest>,
) -> ApiResult<Json<Project>> {
    let correlation_id = request_id
        .header_value()
        .to_str()
        .ok()
        .and_then(|s| Uuid::parse_str(s).ok());

    let project = state
        .project_service
        .update(&project_id, payload.name, payload.repos, payload.ssh_keys)
        .await
        .map_err(|e| {
            tracing::error!(
                correlation_id = ?correlation_id,
                project_id = %project_id,
                error = %e,
                "Failed to update project"
            );
            ErrorResponse::from(ServerError::DatabaseError(e.to_string()))
                .with_correlation_id(correlation_id.unwrap_or_default())
        })?;

    Ok(Json(project))
}

pub async fn delete_project(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Path(project_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    let correlation_id = request_id
        .header_value()
        .to_str()
        .ok()
        .and_then(|s| Uuid::parse_str(s).ok());

    state.project_service.delete(&project_id).await.map_err(|e| {
        tracing::error!(
            correlation_id = ?correlation_id,
            project_id = %project_id,
            error = %e,
            "Failed to delete project"
        );
        ErrorResponse::from(ServerError::DatabaseError(e.to_string()))
            .with_correlation_id(correlation_id.unwrap_or_default())
    })?;

    tracing::info!(
        correlation_id = ?correlation_id,
        project_id = %project_id,
        "Project deleted successfully"
    );

    Ok(StatusCode::NO_CONTENT)
}
