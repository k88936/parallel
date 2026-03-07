use axum::{
    extract::Request,
    http::{header, HeaderMap},
    middleware::Next,
    response::Response,
};
use tower_http::request_id::{MakeRequestId, RequestId};
use uuid::Uuid;

#[derive(Clone, Copy)]
pub struct CorrelationIdGenerator;

impl MakeRequestId for CorrelationIdGenerator {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let id = Uuid::new_v4().to_string().parse().ok()?;
        Some(RequestId::new(id))
    }
}

pub async fn add_correlation_header(
    request: Request,
    next: Next,
) -> Response {
    let request_id = request.extensions().get::<RequestId>().map(|id| id.header_value().clone());
    
    let mut response = next.run(request).await;
    
    if let Some(id) = request_id {
        response.headers_mut().insert(
            header::HeaderName::from_static("x-correlation-id"),
            id,
        );
    }
    
    response
}

pub fn get_correlation_id(headers: &HeaderMap) -> Option<Uuid> {
    headers
        .get("x-correlation-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
}
