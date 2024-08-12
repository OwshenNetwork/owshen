use crate::blockchain::Blockchain;

use crate::services::api_services::api_routes;
use crate::services::Context;

use anyhow::Result;
use jsonrpsee::server::{RpcModule, Server};
use jsonrpsee::types::ErrorCode;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use tower_http_cors::cors::CorsLayer as CorsLayerAxum;

use super::{ContextKvStore, ContextSigner};

pub async fn api_server<S: ContextSigner + 'static, K: ContextKvStore + 'static>(
    ctx: Arc<Mutex<Context<S, K>>>,
    port: u16,
) -> Result<()> {
    let app = api_routes(ctx);
    let app_with_middleware = app.layer(CorsLayerAxum::permissive());

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
    log::info!("Running API server on: {}", addr);

    axum::Server::bind(&addr)
        .serve(app_with_middleware.into_make_service())
        .await?;

    Ok(())
}

fn anyhow_to_rpc_error(e: anyhow::Error) -> ErrorCode {
    log::error!("RPC Error: {}", e);
    ErrorCode::InternalError
}

pub async fn rpc_server<S: ContextSigner + 'static, K: ContextKvStore + 'static>(
    ctx: Arc<Mutex<Context<S, K>>>,
    port: u16,
) -> Result<()> {
    let chain_id = ctx.lock().await.chain.config().chain_id;

    let cors = CorsLayer::new()
        .allow_methods(tower_http::cors::Any)
        .allow_origin(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);
    let middleware = tower::ServiceBuilder::new().layer(cors);
    let rpc_addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port));
    let server = Server::builder()
        .set_http_middleware(middleware)
        .build(rpc_addr)
        .await?;
    let mut module = RpcModule::new(ctx);

    module.register_async_method("net_version", move |params, ctx, _| async move {
        log::info!("net_version! {:?}", params);
        crate::services::rpc_services::net_version(ctx, params)
            .await
            .map_err(anyhow_to_rpc_error)
    })?;
    module.register_async_method("eth_call", move |params, ctx, _| async move {
        log::info!("eth_call! {:?}", params);
        crate::services::rpc_services::eth_call(ctx, params)
            .await
            .map_err(anyhow_to_rpc_error)
    })?;
    module.register_async_method("eth_blockNumber", move |params, ctx, _| async move {
        log::info!("eth_blockNumber! {:?}", params);
        crate::services::rpc_services::eth_block_number(ctx, params)
            .await
            .map_err(anyhow_to_rpc_error)
    })?;

    module.register_async_method("eth_getBalance", move |params, ctx, _| async move {
        log::info!("eth_getBalance! {:?}", params);
        crate::services::rpc_services::eth_get_balance(ctx, params)
            .await
            .map_err(anyhow_to_rpc_error)
    })?;

    module.register_async_method("eth_sendTransaction", move |params, ctx, _| async move {
        log::info!("eth_sendTransaction! {:?}", params);
        crate::services::rpc_services::todo(ctx, params)
            .await
            .map_err(anyhow_to_rpc_error)
    })?;
    module.register_async_method("eth_sendRawTransaction", move |params, ctx, _| async move {
        log::info!("eth_sendRawTransaction! {:?}", params);
        crate::services::rpc_services::eth_send_raw_transaction(ctx, params)
            .await
            .map_err(anyhow_to_rpc_error)
    })?;
    module.register_async_method("eth_estimateGas", move |params, ctx, _| async move {
        log::info!("eth_estimateGas! {:?}", params);
        crate::services::rpc_services::eth_estimate_gas(ctx, params)
            .await
            .map_err(anyhow_to_rpc_error)
    })?;
    module.register_async_method("eth_gasPrice", move |params, ctx, _| async move {
        log::info!("eth_gasPrice! {:?}", params);
        crate::services::rpc_services::eth_get_gas_price(ctx, params)
            .await
            .map_err(anyhow_to_rpc_error)
    })?;
    module.register_async_method(
        "eth_getTransactionCount",
        move |params, ctx, _| async move {
            log::info!("eth_getTransactionCount! {:?}", params);
            crate::services::rpc_services::eth_get_transaction_count(ctx, params)
                .await
                .map_err(anyhow_to_rpc_error)
        },
    )?;
    module.register_async_method(
        "eth_getTransactionReceipt",
        move |params, ctx, _| async move {
            log::info!("eth_getTransactionReceipt! {:?}", params);
            crate::services::rpc_services::eth_get_transaction_receipt(ctx, params)
                .await
                .map_err(anyhow_to_rpc_error)
        },
    )?;
    module.register_async_method("eth_getBlockByNumber", move |params, ctx, _| async move {
        log::info!("eth_getBlockByNumber! {:?}", params);
        crate::services::rpc_services::eth_get_block_by_number(ctx, params)
            .await
            .map_err(anyhow_to_rpc_error)
    })?;
    module.register_async_method("eth_chainId", move |params, ctx, _| async move {
        log::info!("eth_chainId! {:?}", params);
        crate::services::rpc_services::eth_chain_id(ctx, params)
            .await
            .map_err(anyhow_to_rpc_error)
    })?;
    module.register_async_method("eth_requestAccounts", move |params, ctx, _| async move {
        log::info!("eth_requestAccounts! {:?}", params);
        crate::services::rpc_services::eth_request_accounts(ctx, params)
            .await
            .map_err(anyhow_to_rpc_error)
    })?;
    module.register_async_method("eth_feeHistory", move |params, ctx, _| async move {
        log::info!("eth_feeHistory! {:?}", params);
        crate::services::rpc_services::eth_fee_history(ctx, params)
            .await
            .map_err(anyhow_to_rpc_error)
    })?;
    module.register_async_method(
        "eth_getTransactionByHash",
        move |params, ctx, _| async move {
            log::info!("eth_getTransactionByHash! {:?}", params);
            crate::services::rpc_services::eth_get_transaction_by_hash(ctx, params)
                .await
                .map_err(anyhow_to_rpc_error)
        },
    )?;
    module.register_async_method("eth_get_code", move |params, ctx, _| async move {
        log::info!("eth_getTransactionByHash! {:?}", params);
        crate::services::rpc_services::eth_get_code(ctx, params)
            .await
            .map_err(anyhow_to_rpc_error)
    })?;

    for method_name in [
        "debug_getBadBlocks",
        "debug_getRawBlock",
        "debug_getRawHeader",
        "debug_getRawReceipts",
        "debug_getRawTransaction",
        "engine_exchangeCapabilities",
        "engine_exchangeTransitionConfigurationV1",
        "engine_forkchoiceUpdatedV1",
        "engine_forkchoiceUpdatedV2",
        "engine_forkchoiceUpdatedV3",
        "engine_getPayloadBodiesByHashV1",
        "engine_getPayloadBodiesByHashV2",
        "engine_getPayloadBodiesByRangeV1",
        "engine_getPayloadBodiesByRangeV2",
        "engine_getPayloadV1",
        "engine_getPayloadV2",
        "engine_getPayloadV3",
        "engine_getPayloadV4",
        "engine_newPayloadV1",
        "engine_newPayloadV2",
        "engine_newPayloadV3",
        "engine_newPayloadV4",
        "eth_accounts",
        "eth_blobBaseFee",
        "eth_coinbase",
        "eth_createAccessList",
        "eth_getBlockByHash",
        "eth_getBlockReceipts",
        "eth_getBlockTransactionCountByHash",
        "eth_getBlockTransactionCountByNumber",
        "eth_getCode",
        "eth_getFilterChanges",
        "eth_getFilterLogs",
        "eth_getLogs",
        "eth_getProof",
        "eth_getStorageAt",
        "eth_getTransactionByBlockHashAndIndex",
        "eth_getTransactionByBlockNumberAndIndex",
        "eth_getUncleCountByBlockHash",
        "eth_getUncleCountByBlockNumber",
        "eth_maxPriorityFeePerGas",
        "eth_newBlockFilter",
        "eth_newFilter",
        "eth_newPendingTransactionFilter",
        "eth_sign",
        "eth_signTransaction",
        "eth_syncing",
        "eth_uninstallFilter",
    ] {
        module.register_async_method(method_name, move |params, ctx, _| async move {
            log::info!("{}! {:?}", method_name, params);
            crate::services::rpc_services::todo(ctx, params)
                .await
                .map_err(anyhow_to_rpc_error)
        })?;
    }

    let addr = server.local_addr()?;
    log::info!("Running RPC server on: {} (Chain-id: {})", addr, chain_id);
    let handle = server.start(module);
    handle.stopped().await;

    Ok(())
}
