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
use eyre::Result;

use serde_json::json;
use structopt::StructOpt;
use tokio::{fs::File, sync::Mutex, task};
use tokio_util::codec::{BytesCodec, FramedRead};
use tower_http::{cors::CorsLayer, services::ServeFile};

use crate::{
    apis::{self},
    config::{Config, Context, EventsLatestStatus, NetworkManager, NodeManager, Peer, Wallet},
    fmt::FMT,
    genesis::Genesis,
    keys::{PrivateKey, PublicKey},
};

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
    #[structopt(long, help = "Enable test mode")]
    test: bool,
}

pub async fn wallet(opt: WalletOpt, config_path: PathBuf, wallet_path: PathBuf) {
    let WalletOpt {
        db,
        config,
        ip,
        port,
        bootstrap_peers,
        peer2peer,
        test,
    } = opt;

    let app_dir_path = std::env::var("APPDIR").unwrap_or_else(|_| "".to_string());
    let config_path = if test {
        config.unwrap_or_else(|| config_path.clone())
    } else {
        PathBuf::from(format!("{}/usr/share/networks/Sepolia.json", app_dir_path))
    };

    let wallet_path = db.unwrap_or(wallet_path.clone());

    let config = std::fs::read_to_string(&config_path)
        .map(|s| {
            let c: Config = serde_json::from_str(&s).expect("Invalid config file!");
            c
        })
        .ok();

    if let Some(config) = &config {
        let _ = serve_wallet(
            ip,
            port,
            wallet_path,
            config.token_contracts.clone(),
            bootstrap_peers,
            peer2peer,
            test,
            config.clone(),
        )
        .await;
    } else {
        log::error!("Owshen is not initialized!");
    }
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
    test: bool,
    config: Config,
) -> Result<()> {
    let app_dir_path = std::env::var("APPDIR").unwrap_or_else(|_| "".to_string());

    let params_file: Option<PathBuf> = Some(
        if test {
            "contracts/circuits/coin_withdraw_0001.zkey".to_string()
        } else {
            format!("{}/usr/bin/coin_withdraw_0001.zkey", app_dir_path)
        }
        .into(),
    );
    let send_params_file = params_file.clone();
    let genesis_path = if test {
        "owshen-genesis.dat".to_string()
    } else {
        format!(
            "{}/usr/share/genesis/{}-owshen-genesis.dat",
            app_dir_path, config.name
        )
    };
    let witness_gen_path = if test {
        "contracts/circuits/coin_withdraw_cpp/coin_withdraw".into()
    } else {
        format!("{}/usr/bin/coin_withdraw", app_dir_path).to_string()
    };

    let prover_path = if test {
        "rapidsnark/package/bin/prover".to_string()
    } else {
        format!("{}/usr/bin/prover", app_dir_path).to_string()
    };

    let send_witness_gen_path = witness_gen_path.clone();
    let send_prover_path = prover_path.clone();
    let genesis: Option<Genesis> = if let Ok(f) = std::fs::read(genesis_path) {
        bincode::deserialize(&f).ok()
    } else {
        None
    };

    let owshen_contract_deployment_block_number = config.owshen_contract_deployment_block_number;

    let context = Arc::new(Mutex::new(Context {
        coins: vec![],
        genesis: genesis.unwrap(),
        fmt: FMT::new(),
        events_latest_status: EventsLatestStatus {
            last_sent_event: 0,
            last_spent_event: 0,
        },
        node_manager: NodeManager {
            ip: None,
            port: None,

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
                log::info!("Syncing with peers...");
                let now = std::time::Instant::now();

                let mut node_manager = context_sync.lock().await.node_manager.clone();
                node_manager.sync_with_peers();

                context_sync.lock().await.node_manager = node_manager;

                log::info!("Syncing with peers took: {:?}", now.elapsed());
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

    let app = Router::new()
        .route("/", get(move || serve_index(test)))
        .route(
            "/static/*file",
            get(|params: extract::Path<String>| async move {
                let file_path = PathBuf::from(static_files_path).join(params.as_str());
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
                                params_file,
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
                                send_params_file,
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
                    handle_error(apis::set_network(Query(req), contest_set_network, test).await)
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

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for event");
}
