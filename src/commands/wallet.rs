use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    process::Command,
    sync::Arc,
};

use axum::{
    body::Body,
    extract::{self, Query},
    http::{Response, StatusCode},
    routing::{get, post},
    Router,
};
use mime_guess::from_ext;
use serde::{Deserialize, Serialize};
use std::path::Path;

use eyre::Result;

use structopt::StructOpt;
use tokio::{sync::Mutex, task};
use tower_http::cors::CorsLayer;

use crate::{
    apis,
    checkpointed_hashchain::CheckpointedHashchain,
    config::{Config, Context, EventsLatestStatus, NodeManager, Peer, Wallet},
    genesis::Genesis,
    keys::{PrivateKey, PublicKey},
};

#[derive(Debug, StructOpt, PartialEq, Clone, Serialize, Deserialize, Copy)]
pub enum Mode {
    Test,
    AppImage,
    Windows,
}

impl std::str::FromStr for Mode {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "test" => Ok(Mode::Test),
            "appimage" => Ok(Mode::AppImage),
            "windows" => Ok(Mode::Windows),
            _ => Err("No matching mod found"),
        }
    }
}

use super::utils::handle_error;

#[derive(StructOpt, Debug)]
pub struct WalletOpt {
    #[structopt(long)]
    db: Option<PathBuf>,
    #[structopt(long)]
    config: Option<PathBuf>,
    #[structopt(long, default_value = "127.0.0.1")]
    ip: String,
    #[structopt(long, default_value = "9000")]
    port: u16,
    #[structopt(long, parse(try_from_str))]
    bootstrap_peers: Vec<Peer>,
    #[structopt(long)]
    peer2peer: bool,
    #[structopt(long, help = "Select mode: test, appimage, windows")]
    mode: Mode,
    #[structopt(long)]
    dev: bool,
}

pub async fn wallet(opt: WalletOpt, wallet_path: PathBuf) -> Result<(), eyre::Report> {
    let WalletOpt {
        db,
        config,
        ip,
        port,
        bootstrap_peers,
        peer2peer,
        mode,
        dev,
    } = opt;

    let forced_config: Option<Config> = if let Some(p) = config {
        Some(serde_json::from_str(&std::fs::read_to_string(&p)?)?)
    } else {
        None
    };

    let wallet_path = db.unwrap_or(wallet_path.clone());

    serve_wallet(
        ip,
        port,
        wallet_path,
        bootstrap_peers,
        peer2peer,
        mode,
        dev,
        forced_config,
    )
    .await?;

    Ok(())
}

fn read_priv_key(wallet_path: PathBuf) -> Option<PrivateKey> {
    let wallet = std::fs::read_to_string(wallet_path)
        .map(|s| {
            let w: Wallet = serde_json::from_str(&s).expect("Invalid wallet file!");
            w
        })
        .ok();

    wallet.map(|w| w.entropy.clone().into())
}

lazy_static! {
    pub static ref PROVER_FILE: PathBuf = "assets/bin/prover".into();
    pub static ref WITNESS_GEN_FILE: PathBuf = "assets/bin/coin_withdraw".into();
    pub static ref PARAMS_FILE: PathBuf = "assets/zk/coin_withdraw_0001.zkey".into();
    pub static ref GENESIS_FILE: PathBuf = "assets/bin/owshen-genesis.dat".into();
}

async fn serve_wallet(
    ip: String,
    port: u16,
    wallet_path: PathBuf,
    bootstrap_peers: Vec<Peer>,
    peer2peer: bool,
    mode: Mode,
    dev: bool,
    forced_config: Option<Config>,
) -> Result<()> {
    let genesis: Genesis = bincode::deserialize(&std::fs::read(GENESIS_FILE.clone())?)?;

    let context = Arc::new(Mutex::new(Context {
        coins: vec![],
        genesis,
        chc: CheckpointedHashchain::new(),
        events_latest_status: EventsLatestStatus {
            last_sent_event: 0,
            last_spent_event: 0,
        },
        node_manager: NodeManager {
            external_addr: None,

            network: None,
            peers: bootstrap_peers,
            elected_peer: None,
            is_peer2peer: peer2peer,

            is_client: true,
        },
        syncing: Arc::new(std::sync::Mutex::new(None)),
        syncing_task: None,
    }));

    if let Some(conf) = forced_config.clone() {
        context.lock().await.switch_network(conf)?;
    }

    if peer2peer {
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
    }

    let context_coin = context.clone();
    let context_withdraw = context.clone();
    let context_send = context.clone();
    let context_info = context.clone();
    let contest_set_network = context.clone();

    let withdraw_wallet_path = wallet_path.clone();
    let coins_wallet_path = wallet_path.clone();
    let info_wallet_path = wallet_path.clone();
    let send_wallet_path = wallet_path.clone();
    let init_wallet_path = wallet_path.clone();
    let set_params_path_wallet_path = wallet_path.clone();

    let mut app = Router::new();

    app = app
        .route(
            "/",
            get(|| async move { serve_file("index.html").unwrap() }),
        )
        .route(
            "/coins",
            get(move || async move {
                handle_error(
                    async {
                        let priv_key = read_priv_key(coins_wallet_path)
                            .ok_or(eyre::Report::msg("Wallet is not initialized!"))?;
                        apis::coins(context_coin, priv_key).await
                    }
                    .await,
                )
            }),
        )
        .route(
            "/withdraw",
            get(
                move |extract::Query(req): extract::Query<apis::GetWithdrawRequest>| async move {
                    handle_error(
                        async {
                            let priv_key = read_priv_key(withdraw_wallet_path)
                                .ok_or(eyre::Report::msg("Wallet is not initialized!"))?;
                            apis::withdraw(
                                Query(req),
                                context_withdraw,
                                priv_key,
                                WITNESS_GEN_FILE.clone(),
                                PROVER_FILE.clone(),
                                Some(PARAMS_FILE.clone()),
                                mode,
                            )
                            .await
                        }
                        .await,
                    )
                },
            ),
        )
        .route(
            "/send",
            get(
                move |extract::Query(req): extract::Query<apis::GetSendRequest>| async move {
                    handle_error(
                        async {
                            let priv_key = read_priv_key(send_wallet_path)
                                .ok_or(eyre::Report::msg("Wallet is not initialized!"))?;
                            apis::send(
                                Query(req),
                                context_send,
                                priv_key,
                                WITNESS_GEN_FILE.clone(),
                                PROVER_FILE.clone(),
                                Some(PARAMS_FILE.clone()),
                                mode,
                            )
                            .await
                        }
                        .await,
                    )
                },
            ),
        )
        .route(
            "/stealth",
            get(
                |extract::Query(req): extract::Query<apis::GetStealthRequest>| async move {
                    handle_error(apis::stealth(Query(req)).await)
                },
            ),
        )
        .route(
            "/info",
            get(move || async move {
                handle_error(
                    async {
                        let priv_key = read_priv_key(info_wallet_path)
                            .ok_or(eyre::Report::msg("Wallet is not initialized!"))?;
                        Ok(apis::info(PublicKey::from(priv_key), context_info, dev, mode).await?)
                    }
                    .await,
                )
            }),
        )
        .route(
            "/set-network",
            post(
                move |extract::Query(req): extract::Query<apis::SetNetworkRequest>| async move {
                    handle_error(
                        apis::set_network(Query(req), contest_set_network, forced_config).await,
                    )
                },
            ),
        )
        .route(
            "/set-params-path",
            post(
                move |req: extract::Query<apis::SetParamsPathRequest>| async move {
                    handle_error(apis::set_params_path(req, set_params_path_wallet_path).await)
                },
            ),
        )
        .route(
            "/init",
            post(
                move |req: extract::Json<apis::PostInitRequest>| async move {
                    handle_error(apis::init(init_wallet_path, req).await)
                },
            ),
        )
        .route(
            "/*file",
            get(|params: extract::Path<String>| async move {
                let file_name = params.as_str();
                serve_file(file_name).unwrap()
            }),
        )
        .layer(CorsLayer::permissive());

    let ip_addr: IpAddr = ip.parse().expect("failed to parse ip");
    let addr = SocketAddr::new(ip_addr, port);

    if dev {
        let frontend = async {
            task::spawn_blocking(move || {
                let _output = Command::new("npm")
                    .arg("start")
                    .env(
                        "REACT_APP_OWSHEN_ENDPOINT",
                        format!("http://127.0.0.1:{}", 9000),
                    )
                    .current_dir("client")
                    .spawn()
                    .expect("failed to execute process");
            });
            Ok::<(), eyre::Error>(())
        };
        let backend = async {
            axum::Server::bind(&addr)
                .serve(app.into_make_service())
                .await?;
            Ok::<(), eyre::Error>(())
        };

        tokio::try_join!(backend, frontend)?;
        Ok(())
    } else {
        let server = axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .with_graceful_shutdown(shutdown_signal());

        // Attempt to open the web browser
        if webbrowser::open(&format!("http://{}", addr)).is_err() {
            println!(
                "Failed to open web browser. Please navigate to http://{} manually",
                addr
            );
        }

        server.await.map_err(eyre::Report::new)?;
        Ok(())
    }
}

fn serve_file(path: &str) -> Result<Response<Body>, eyre::Report> {
    if let Ok(content) = std::fs::read(Path::new("assets").join(path)) {
        let mime_type = match Path::new(path).extension().and_then(|ext| ext.to_str()) {
            Some(ext) => from_ext(ext).first_or_octet_stream().as_ref().to_string(),
            None => "application/octet_stream".to_string(),
        };

        let response = Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", mime_type)
            .body(Body::from(content))?;

        Ok(response)
    } else {
        Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from("File not found"))?)
    }
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for event");
}
