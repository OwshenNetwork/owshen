use axum::{response::IntoResponse, Json};
use reqwest::StatusCode;
use serde_json::json;

pub fn handle_error<T: IntoResponse>(result: Result<T, eyre::Report>) -> impl IntoResponse {
    match result {
        Ok(a) => a.into_response(),
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
