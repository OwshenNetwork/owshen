use std::net::{IpAddr, SocketAddr};
use std::{path::PathBuf, sync::Arc};

use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{extract, Json, Router};
use ethers::providers::{Http, Middleware, Provider};
use structopt::StructOpt;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

use crate::apis;
use crate::config::{
    Config, Network, NodeContext, NodeManager, Peer, GOERLI_ENDPOINT, NODE_UPDATE_INTERVAL,
};

#[derive(StructOpt, Debug, Clone)]
pub struct NodeOpt {
    #[structopt(long, default_value = GOERLI_ENDPOINT)]
    endpoint: String,
    #[structopt(long)]
    config: Option<PathBuf>,

    #[structopt(long, default_value = "127.0.0.1")]
    ip: String,
    #[structopt(long, default_value = "8888")]
    port: u16,

    #[structopt(long, parse(try_from_str))]
    bootstrap_peers: Vec<Peer>,
    #[structopt(long)]
    peer2peer: bool,
}

pub async fn node(opt: NodeOpt, config_path: PathBuf) {
    let NodeOpt {
        endpoint,
        config,
        ip,
        port,
        bootstrap_peers,
        peer2peer,
    } = opt;

    let config_path = config.unwrap_or(config_path.clone());
    let config: Option<Config> = std::fs::read_to_string(&config_path)
        .map(|s| {
            let c: Config = serde_json::from_str(&s).expect("Invalid config file!");
            c
        })
        .ok();

    // validate ip and port
    let _ = ip.parse::<IpAddr>().expect("Invalid ip address");
    let _ = SocketAddr::new(ip.parse().unwrap(), port);

    let provider = Provider::<Http>::try_from(endpoint.clone()).unwrap();
    let context = Arc::new(Mutex::new(NodeContext {
        node_manager: NodeManager {
            ip: Some(ip.clone()),
            port: Some(port.clone()),
            network: Some(Network {
                provider: Arc::new(provider),
                config: config.unwrap_or_default(),
            }),
            peers: bootstrap_peers,
            elected_peer: None,
            is_peer2peer: peer2peer,

            is_client: false,
        },

        spent_events: vec![],
        sent_events: vec![],
        currnet_block_number: 0,
    }));

    let context_sync = context.clone();
    tokio::spawn(async move {
        loop {
            log::info!("Syncing with peers...");
            let now = std::time::Instant::now();

            let mut node_manager = context_sync.lock().await.node_manager.clone();
            node_manager.sync_with_peers();

            context_sync.lock().await.node_manager = node_manager;

            log::info!("Syncing with peers took: {:?}", now.elapsed());
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    });

    let context_status = context.clone();
    let context_events = context.clone();
    let context_get_peers = context.clone();
    let context_handshake = context.clone();

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
        .layer(CorsLayer::permissive());

    let ip_addr: IpAddr = ip.parse().expect("failed to parse ip");
    let addr = SocketAddr::new(ip_addr, port);

    let backend = async {
        log::info!("Server started at: {:?}", addr);
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await?;
        Ok::<(), eyre::Error>(())
    };

    #[allow(unreachable_code)]
    let sync_job = async {
        loop {
            log::info!("Updating events...");
            update_events(context.clone()).await;

            log::info!("Sleeping for {} seconds...", NODE_UPDATE_INTERVAL);
            tokio::time::sleep(tokio::time::Duration::from_secs(NODE_UPDATE_INTERVAL)).await;
        }
        Ok::<(), eyre::Error>(())
    };

    _ = tokio::try_join!(backend, sync_job);
}

async fn update_events(context: Arc<Mutex<NodeContext>>) {
    let mut context = context.lock().await;
    let network = context.node_manager.get_provider_network().clone();

    if let Some(network) = network {
        if context.node_manager.is_peer2peer {
            let from_spent: u64 = context.spent_events.len().try_into().unwrap();
            let from_sent: u64 = context.sent_events.len().try_into().unwrap();
            let (spent_events, sent_events, peer_current_block_number) = context
                .node_manager
                .clone()
                .get_events_from_elected_peer(from_spent, from_sent);

            if peer_current_block_number >= context.currnet_block_number {
                context.spent_events.extend(spent_events.clone());
                context.sent_events.extend(sent_events.clone());
                context.currnet_block_number = peer_current_block_number;

                log::info!(
                    "new events: {} spent, {} sent",
                    spent_events.len(),
                    sent_events.len()
                );
            } else {
                log::info!("No new events");
            }
        } else {
            let curr_block_number = network.provider.get_block_number().await;
            if let Err(e) = curr_block_number {
                log::error!("Failed to get current block number: {}", e);
                return;
            }

            let curr_block_number = curr_block_number.unwrap().as_u64();
            let curr = context.currnet_block_number;

            let spent_events = context
                .node_manager
                .get_spend_events(curr, curr_block_number)
                .await;
            let sent_events = context
                .node_manager
                .get_sent_events(curr, curr_block_number)
                .await;

            log::info!(
                "new events: {} spent, {} sent",
                spent_events.len(),
                sent_events.len()
            );
            context.spent_events.extend(spent_events);
            context.sent_events.extend(sent_events);
            context.currnet_block_number = curr_block_number;
        }
    } else {
        log::error!("Provider is not set");
    }
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
