use axum::http::StatusCode;
use axum::{response::IntoResponse, Json};
use serde_json::json;

pub fn handle_error<T: IntoResponse>(result: Result<T, anyhow::Error>) -> impl IntoResponse {
    match result {
        Ok(response) => response.into_response(),
        Err(e) => {
            log::error!("{}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": true,
                    "message": e.to_string()
                })),
            )
                .into_response()
        }
    }
}
