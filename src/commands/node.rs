use std::net::SocketAddr;
use std::{path::PathBuf, sync::Arc};

use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{extract, Json, Router};
use ethers::providers::{Http, Middleware, Provider};
use structopt::StructOpt;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

use crate::apis;
use crate::config::{Config, Network, NodeContext, NodeManager, Peer, NODE_UPDATE_INTERVAL};

#[derive(StructOpt, Debug, Clone)]
pub struct NodeOpt {
    #[structopt(long)]
    endpoint: String,
    #[structopt(long)]
    config: PathBuf,

    #[structopt(long, default_value = "127.0.0.1:8888")]
    external: SocketAddr,
    #[structopt(long, default_value = "0.0.0.0:8888")]
    interface: SocketAddr,

    #[structopt(long, parse(try_from_str))]
    bootstrap_peers: Vec<Peer>,
    #[structopt(long)]
    peer2peer: bool,

    #[structopt(long)]
    relayer: Option<String>,
}

pub async fn node(opt: NodeOpt) -> Result<(), eyre::Report> {
    let NodeOpt {
        endpoint,
        config,
        external,
        interface,
        bootstrap_peers,
        peer2peer,
        relayer,
    } = opt;

    let config: Config = std::fs::read_to_string(&config).map(|s| {
        let c: Config = serde_json::from_str(&s).expect("Invalid config file!");
        c
    })?;

    let provider = Provider::<Http>::try_from(endpoint.clone())?;
    let context = Arc::new(Mutex::new(NodeContext {
        node_manager: NodeManager {
            external_addr: Some(external.clone()),
            network: Some(Network {
                provider: Arc::new(provider),
                config,
            }),
            peers: bootstrap_peers,
            elected_peer: None,
            is_peer2peer: peer2peer,

            is_client: false,
        },

        spent_events: vec![],
        sent_events: vec![],
        currnet_block_number: 0,
        mempool: vec![],
    }));

    let context_sync = context.clone();
    tokio::spawn(async move {
        loop {
            if let Err(e) = async {
                log::info!("Syncing with peers...");
                let now = std::time::Instant::now();

                let mut node_manager = context_sync.lock().await.node_manager.clone();
                node_manager.sync_with_peers().await?;
                context_sync.lock().await.node_manager = node_manager;

                log::info!("Syncing with peers took: {:?}", now.elapsed());
                Ok::<(), eyre::Report>(())
            }
            .await
            {
                log::error!("Error occurred while syncing with peers: {}", e);
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    });

    let context_status = context.clone();
    let context_events = context.clone();
    let context_get_peers = context.clone();
    let context_handshake = context.clone();
    let context_get_mempool = context.clone();
    let context_post_tx = context.clone();

    let app = Router::new()
        .route(
            "/status",
            get(move || async move { handle_error(apis::status(context_status).await) }),
        )
        .route(
            "/events",
            get(
                move |extract::Query(req): extract::Query<apis::GetEventsRequest>| async move {
                    handle_error(apis::events(Query(req), context_events).await)
                },
            ),
        )
        .route(
            "/get-peers",
            get(move || async move { handle_error(apis::get_peers(context_get_peers).await) }),
        )
        .route(
            "/handshake",
            get(
                move |extract::Query(req): extract::Query<apis::GetHandShakeRequest>| async move {
                    handle_error(apis::handshake(Json(req), context_handshake.clone()).await)
                },
            ),
        )
        .route(
            "/",
            get(move || async { Json(serde_json::json!({"ok": true})) }),
        )
        .route(
            "/transact",
            post(
                move |extract::Json(req): extract::Json<apis::PostTransactRequest>| async move {
                    handle_error(async { apis::transact(context_post_tx, req).await }.await)
                },
            ),
        )
        .route(
            "/mempool",
            get(
                move |extract::Query(req): extract::Query<apis::GetMempoolRequest>| async move {
                    handle_error(async { apis::mempool(context_get_mempool, req).await }.await)
                },
            ),
        )
        .layer(CorsLayer::permissive());

    let backend = async {
        log::info!("Server started at: {:?}", interface);
        axum::Server::bind(&interface)
            .serve(app.into_make_service())
            .await?;
        Ok::<(), eyre::Error>(())
    };

    #[allow(unreachable_code)]
    let sync_job = async {
        loop {
            log::info!("Updating events...");
            update_events(context.clone()).await?;

            log::info!("Sleeping for {} seconds...", NODE_UPDATE_INTERVAL);
            tokio::time::sleep(tokio::time::Duration::from_secs(NODE_UPDATE_INTERVAL)).await;

            if let Some(priv_key) = &relayer {
                relay_txs(priv_key.clone(), context.clone()).await.unwrap(); // TODO: handle exceptions of the loop
            }
        }
        Ok::<(), eyre::Error>(())
    };

    tokio::try_join!(backend, sync_job)?;

    Ok(())
}

async fn relay_txs(priv_key: String, context: Arc<Mutex<NodeContext>>) -> Result<(), eyre::Report> {
    let txs = context.lock().await.mempool.clone();
    for _tx in txs.iter() {
        log::info!("Should relay tx with priv-key: {}", priv_key);
        // TODO
    }
    Ok(())
}

async fn update_events(context: Arc<Mutex<NodeContext>>) -> Result<(), eyre::Report> {
    let mut ctx = context.lock().await;
    let network = ctx.node_manager.get_provider_network().clone();

    if let Some(network) = network {
        if ctx.node_manager.is_peer2peer {
            let from_spent = ctx.spent_events.len();
            let from_sent = ctx.sent_events.len();
            let (spent_events, sent_events, peer_current_block_number) = ctx
                .node_manager
                .clone()
                .get_events_from_elected_peer(from_spent, from_sent)
                .await?;

            if peer_current_block_number >= ctx.currnet_block_number {
                ctx.spent_events.extend(spent_events.clone());
                ctx.sent_events.extend(sent_events.clone());
                ctx.currnet_block_number = peer_current_block_number;

                log::info!(
                    "new events: {} spent, {} sent",
                    spent_events.len(),
                    sent_events.len()
                );
            } else {
                log::info!("No new events");
            }
        } else {
            let curr = std::cmp::max(
                ctx.currnet_block_number,
                ctx.node_manager
                    .network
                    .as_ref()
                    .unwrap()
                    .config
                    .owshen_contract_deployment_block_number
                    .as_u64(),
            );
            let node_manager = ctx.node_manager.clone();
            drop(ctx);

            let curr_block_number = network.provider.get_block_number().await?.as_u64();

            let spent_events = node_manager.get_spend_events(curr, curr_block_number).await;

            let sent_events = node_manager.get_sent_events(curr, curr_block_number).await;

            log::info!(
                "New events: {} spent, {} sent",
                spent_events.len(),
                sent_events.len()
            );
            ctx = context.lock().await;
            ctx.spent_events.extend(spent_events);
            ctx.sent_events.extend(sent_events);
            ctx.currnet_block_number = curr_block_number;
        }
    } else {
        log::error!("Provider is not set");
    }

    Ok(())
}

fn handle_error<T: IntoResponse>(result: Result<T, eyre::Report>) -> impl IntoResponse {
    match result {
        Ok(a) => a.into_response(),
        Err(e) => {
            log::error!("{}", e);
            let error_message = format!("Internal server error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_message)).into_response()
        }
    }
}
