use std::{
    fs::read_to_string,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    process::Command,
    sync::Arc,
};

use axum::{
    body::Body,
    extract::{self, Query},
    http::{Response, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, get_service, post},
    Json, Router,
};
use mime_guess::from_ext;
use std::path::Path;
use tower_http::services::ServeFile;

use eyre::Result;

use serde_json::json;
use structopt::StructOpt;
use tokio::{fs::File, sync::Mutex, task};
use tokio_util::codec::{BytesCodec, FramedRead};
use tower_http::cors::CorsLayer;

use crate::{
    apis::{self},
    checkpointed_hashchain::CheckpointedHashchain,
    config::{Config, Context, EventsLatestStatus, NetworkManager, NodeManager, Peer, Wallet},
    genesis::Genesis,
    keys::{PrivateKey, PublicKey},
};
use statics::*;

enum ResourceType {
    Statics,
    Asset,
    CircuitsStatics,
    ZkStatics,
}

#[derive(Debug, StructOpt, PartialEq, Clone)]
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
            _ => Err("no match"),
        }
    }
}

#[derive(StructOpt, Debug)]
pub struct WalletOpt {
    #[structopt(long)]
    db: Option<PathBuf>,
    #[structopt(long)]
    config: PathBuf,
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
    test: bool,
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
        test,
    } = opt;

    let app_dir_path = std::env::var("APPDIR").unwrap_or_else(|_| "".to_string());
    let config_path = if test {
        config
    } else {
        PathBuf::from(format!("{}/usr/share/networks/Sepolia.json", app_dir_path))
    };

    let config = match mode {
        Mode::Windows => serde_json::from_slice::<Config>(
            &ConfigAsset::get("Localhost.json")
                .ok_or(eyre::eyre!("Asset not found!"))?
                .data,
        )?,
        _ => serde_json::from_str(&std::fs::read_to_string(&config_path)?)?,
    };

    let wallet_path = db.unwrap_or(wallet_path.clone());
    let send_mode = mode.clone();
    let withdrawal_mode = mode.clone();

    serve_wallet(
        ip,
        port,
        wallet_path,
        config.token_contracts.clone(),
        bootstrap_peers,
        peer2peer,
        mode,
        test,
        config.clone(),
        send_mode,
        withdrawal_mode,
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

async fn serve_wallet(
    ip: String,
    port: u16,
    wallet_path: PathBuf,
    token_contracts: NetworkManager,
    bootstrap_peers: Vec<Peer>,
    peer2peer: bool,
    mode: Mode,
    test: bool,
    config: Config,
    send_mode: Mode,
    withdrawal_mode: Mode,
) -> Result<()> {
    let app_dir_path = std::env::var("APPDIR").unwrap_or_else(|_| "".to_string());

    let (params_file, genesis_path, witness_gen_path, prover_path): (
        PathBuf,
        PathBuf,
        PathBuf,
        PathBuf,
    ) = if test {
        (
            "contracts/circuits/coin_withdraw_0001.zkey".into(),
            "owshen-genesis.dat".into(),
            "contracts/circuits/coin_withdraw_cpp/coin_withdraw".into(),
            "rapidsnark/package/bin/prover".into(),
        )
    } else {
        (
            format!("{}/usr/bin/coin_withdraw_0001.zkey", app_dir_path).into(),
            format!(
                "{}/usr/share/genesis/{}-owshen-genesis.dat",
                app_dir_path, config.name
            )
            .into(),
            format!("{}/usr/bin/coin_withdraw", app_dir_path).into(),
            format!("{}/usr/bin/prover", app_dir_path).into(),
        )
    };

    let send_witness_gen_path = witness_gen_path.clone();
    let send_prover_path = prover_path.clone();
    let send_params_file = params_file.clone();

    let genesis: Genesis = bincode::deserialize(&std::fs::read(genesis_path)?)?;

    let owshen_contract_deployment_block_number = config.owshen_contract_deployment_block_number;

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

    if peer2peer {
        let context_sync = context.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = async {
                    log::info!("Syncing with peers...");
                    let now = std::time::Instant::now();

                    let mut node_manager = context_sync.lock().await.node_manager.clone();
                    node_manager.sync_with_peers()?;

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
    let root_files_path = format!("{}/usr/share/owshen/client", app_dir_path);
    let static_files_path = format!("{}/usr/share/owshen/client/static", app_dir_path);

    let withdraw_wallet_path = wallet_path.clone();
    let coins_wallet_path = wallet_path.clone();
    let info_wallet_path = wallet_path.clone();
    let send_wallet_path = wallet_path.clone();
    let init_wallet_path = wallet_path.clone();
    let set_params_path_wallet_path = wallet_path.clone();

    let mut app = Router::new();
    if mode != Mode::Windows {
        app = app
            .route("/", get(move || serve_index(test)))
            .route(
                "/static/*file",
                get(|params: extract::Path<String>| async move {
                    let file_path =
                        PathBuf::from(static_files_path).join(params.trim_start_matches('/'));
                    serve_file(file_path).await
                }),
            )
            .route(
                "/manifest.json",
                get_service(ServeFile::new(format!("{}/manifest.json", root_files_path))),
            )
            .route(
                "/asset-manifest.json",
                get_service(ServeFile::new(format!(
                    "{}/asset-manifest.json",
                    root_files_path
                ))),
            )
            .route(
                "/robots.txt",
                get_service(ServeFile::new(format!("{}/robots.txt", root_files_path))),
            )
    } else {
        // For Windows mode, use embedded static file serving
        app = app
            .route("/", get(move || serve_embedded_index()))
            .route(
                "/static/*file",
                get(|params: extract::Path<String>| async move {
                    let file_name = params.as_str();
                    serve_embedded_file(file_name, ResourceType::Statics).await
                }),
            )
            .route(
                "/*file",
                get(|params: extract::Path<String>| async move {
                    let file_name = params.as_str();
                    serve_embedded_file(file_name, ResourceType::Asset).await
                }),
            )
            .route(
                "/witness/:filename",
                get(|params: extract::Path<String>| async move {
                    let file_name = params.as_str();
                    println!("file name witness {:?}", file_name);
                    serve_embedded_file(file_name, ResourceType::CircuitsStatics).await
                }),
            )
            .route(
                "/zk/:filename",
                get(|params: extract::Path<String>| async move {
                    let file_name = params.as_str();
                    println!("file name witness {:?}", file_name);
                    serve_embedded_file(file_name, ResourceType::ZkStatics).await
                }),
            )
    }

    app = app
        .route(
            "/coins",
            get(move || async move {
                handle_error(
                    async {
                        let priv_key = read_priv_key(coins_wallet_path)
                            .ok_or(eyre::Report::msg("Wallet is not initialized!"))?;
                        apis::coins(
                            context_coin,
                            priv_key,
                            owshen_contract_deployment_block_number,
                        )
                        .await
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
                                witness_gen_path,
                                prover_path,
                                Some(params_file),
                                withdrawal_mode,
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
                                send_witness_gen_path,
                                send_prover_path,
                                Some(send_params_file),
                                send_mode,
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
                        Ok(apis::info(
                            PublicKey::from(priv_key),
                            context_info,
                            token_contracts,
                            test,
                        )
                        .await?)
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
                        apis::set_network(Query(req), contest_set_network, test, config).await,
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
        .layer(CorsLayer::permissive());

    let ip_addr: IpAddr = ip.parse().expect("failed to parse ip");
    let addr = SocketAddr::new(ip_addr, port);

    if test {
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

fn handle_error<T: IntoResponse>(result: Result<T, eyre::Report>) -> impl IntoResponse {
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

async fn serve_index(test: bool) -> impl IntoResponse {
    let app_dir_path = std::env::var("APPDIR").unwrap_or_else(|_| "".to_string());
    let index_path = if test {
        "client/build/index.html".to_string()
    } else {
        format!("{}/usr/share/owshen/client/index.html", app_dir_path)
    };

    println!("index path {}", index_path);
    match read_to_string(index_path) {
        Ok(contents) => Html(contents),
        Err(_) => Html("<h1>Error: Unable to read the index file</h1>".to_string()),
    }
}

async fn serve_file(file_path: PathBuf) -> impl IntoResponse {
    if let Ok(file) = File::open(file_path).await {
        let stream = FramedRead::new(file, BytesCodec::new());

        Response::new(Body::wrap_stream(stream))
    } else {
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from("File not found"))
            .unwrap()
    }
}

async fn serve_embedded_index() -> impl IntoResponse {
    match Asset::get("index.html") {
        Some(content) => {
            let body = String::from_utf8(content.data.into_owned())
                .unwrap_or_else(|_| "Failed to parse the file content.".to_string());

            Html(body)
        }
        None => {
            Html("<h1>Error: Unable to find the index file in embedded assets</h1>".to_string())
        }
    }
}

async fn serve_embedded_file(file: &str, resource_type: ResourceType) -> impl IntoResponse {
    match resource_type {
        ResourceType::Statics => serve_from_embed(file, Statics::get),
        ResourceType::Asset => serve_from_embed(file, Asset::get),
        ResourceType::CircuitsStatics => serve_from_embed(file, CircuitsStatics::get),
        ResourceType::ZkStatics => serve_from_embed(file, ZkStatics::get),
    }
}

fn serve_from_embed<F>(file: &str, get_fn: F) -> Response<Body>
where
    F: Fn(&str) -> Option<rust_embed::EmbeddedFile>,
{
    match get_fn(file) {
        Some(content) => {
            let mime_type = match Path::new(file).extension().and_then(|ext| ext.to_str()) {
                Some(ext) => from_ext(ext).first_or_octet_stream().as_ref().to_string(),
                None => "application/octet_stream".to_string(),
            };

            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", mime_type)
                .body(Body::from(content.data))
                .unwrap();

            response
        }
        None => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from("File not found"))
            .unwrap(),
    }
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for event");
}
