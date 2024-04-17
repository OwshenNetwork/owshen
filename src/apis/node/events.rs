use std::sync::Arc;

use axum::{extract::Query, Json};
use bindings::owshen::{SentFilter, SpendFilter};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::config::NodeContext;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetEventsRequest {
    pub from_spend: usize,
    pub from_sent: usize,
    pub length: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetEventsResponse {
    pub spend_events: Vec<SpendFilter>,
    pub sent_events: Vec<SentFilter>,
}

pub async fn events(
    Query(req): Query<GetEventsRequest>,
    context_events: Arc<Mutex<NodeContext>>,
) -> Result<Json<GetEventsResponse>, eyre::Report> {
    if req.length > 256 {
        return Err(eyre::eyre!("Length must be less than 256"));
    }

    let context = context_events.lock().await;
    let spend_events = context
        .spent_events
        .iter()
        .skip(req.from_spend)
        .take(req.length)
        .cloned()
        .collect();
    let sent_events = context
        .sent_events
        .iter()
        .skip(req.from_sent)
        .take(req.length)
        .cloned()
        .collect();

    Ok(Json(GetEventsResponse {
        spend_events,
        sent_events,
    }))
}
