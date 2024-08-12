pub mod deposit;
pub mod explorer;
pub mod test;
pub mod withdraw;
pub mod withdrawals;

use super::{Context, ContextKvStore, ContextSigner};
use crate::client::explorer::{explorer_handler, ExplorerContext};
use crate::client::style::{css_handler, StyleContext};
use crate::client::BlockDetails::{block_details_handler, BlockDetailsContext};
use crate::{db, utils};
use axum::extract::Path;

use axum::response::{Html, IntoResponse, Response};
use axum::{
    extract,
    routing::{get, post},
    Extension, Json, Router,
};
use deposit::{deposit_handler, DepositRequest};
use explorer::{
    get_transaction_by_hash_handler, get_transactions_by_block_handler, GetBlockByHashRequest,
    GetTransactionByHashRequest, GetTransactionsByBlockRequest,
};
use hyper::StatusCode;

use std::sync::Arc;
use tokio::sync::Mutex;

use utils::handle_error;
use withdraw::{withdraw_handler, WithdrawRequest};
use withdrawals::{withdrawals_handler, WithdrawalsRequest};

pub async fn css_handler_endpoint() -> impl IntoResponse {
    let style_ctx = StyleContext {};

    match css_handler(&style_ctx).await {
        Ok(response) => response,
        Err(err) => {
            log::error!("Error handling CSS request: {}", err);
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Internal Server Error".to_string())
                .unwrap()
        }
    }
}

fn explorer_routes<S, K>(ctx: Arc<Mutex<Context<S, K>>>) -> Router
where
    S: ContextSigner + 'static,
    K: ContextKvStore + 'static,
{
    Router::new()
        .route(
            "/by-block",
            post({
                let ctx: Arc<Mutex<Context<S, K>>> = ctx.clone();
                move |Json(req): Json<GetTransactionsByBlockRequest>| async move {
                    handle_error(Ok(get_transactions_by_block_handler(
                        ctx.clone(),
                        extract::Json(req),
                    )
                    .await))
                }
            }),
        )
        .route(
            "/by-tx",
            get({
                let ctx = ctx.clone();
                move |Json(req): Json<GetTransactionByHashRequest>| async move {
                    handle_error(Ok(get_transaction_by_hash_handler(
                        ctx.clone(),
                        extract::Json(req),
                    )
                    .await))
                }
            }),
        )
}

pub fn api_routes<S, K>(ctx: Arc<Mutex<Context<S, K>>>) -> Router
where
    S: ContextSigner + 'static,
    K: ContextKvStore + 'static,
{
    Router::new()
        .route(
            "/",
            get(|| async { Html(include_str!("assets/index.html")) }),
        )
        .route(
            "/deposit",
            post({
                let ctx = ctx.clone();
                move |Json(req): Json<DepositRequest>| async move {
                    handle_error(deposit_handler(ctx.clone(), extract::Json(req)).await)
                }
            }),
        )
        .route(
            "/withdraw",
            get({
                let ctx = ctx.clone();
                move |Json(req): Json<WithdrawRequest>| async move {
                    handle_error(withdraw_handler(ctx.clone(), extract::Json(req)).await)
                }
            }),
        )
        .route(
            "/Withdrawals",
            get({
                let ctx = ctx.clone();
                move |Json(req): Json<WithdrawalsRequest>| async move {
                    handle_error(withdrawals_handler(ctx.clone(), extract::Json(req)).await)
                }
            }),
        )
        .nest("/explorer", explorer_routes(ctx.clone()))
        .route(
            "/explorer",
            get({
                let ctx = ctx.clone();
                move || async move { handle_error(explorer_handler(ctx).await) }
            }),
        )
        .route("/style.css", get(css_handler_endpoint))
        .route(
            "/explorer/:id",
            get({
                let ctx = ctx.clone();
                move |Path(id): Path<String>| async move {
                    let block_details_ctx = BlockDetailsContext {
                        name: String::from("Yoctochain Explorer"),
                        id: id.clone(),
                    };

                    handle_error(block_details_handler(ctx, block_details_ctx).await)
                }
            }),
        )
        .layer(Extension(ctx))
}
