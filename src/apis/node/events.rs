use std::sync::Arc;

use axum::{extract::Query, Json};
use bindings::owshen::{SentFilter, SpendFilter};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::config::NodeContext;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetEventsRequest {
    pub from_spend: u64,
    pub from_sent: u64,
    pub length: u64,
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
    let from_spend = req.from_spend;
    let from_sent = req.from_sent;
    let length = req.length;

    if length > 256 {
        return Err(eyre::eyre!("Length must be less than 256"));
    }

    let context = context_events.lock().await;
    let spend_events = context.spent_events[min(context.spent_events.len(), from_spend as usize)
        ..min(context.spent_events.len(), (from_spend + length) as usize)]
        .to_vec();
    let sent_events = context.sent_events[min(context.sent_events.len(), from_sent as usize)
        ..min(context.sent_events.len(), (from_sent + length) as usize)]
        .to_vec();

    Ok(Json(GetEventsResponse {
        spend_events,
        sent_events,
    }))
}

fn min(a: usize, b: usize) -> usize {
    if a < b {
        a
    } else {
        b
    }
}
