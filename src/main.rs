mod config;
use {
    config::Config,
    bs58,
    futures::{sink::SinkExt, stream::StreamExt},
    log::{info, error, warn},
    std::{collections::HashMap},
    tokio::{signal},
    tonic::{
        transport::ClientTlsConfig,
        service::Interceptor,
        Status,
    },
    yellowstone_grpc_client::GeyserGrpcClient,
    yellowstone_grpc_proto::{
        geyser::SubscribeUpdate,
        prelude::{
            subscribe_update::UpdateOneof,
            CommitmentLevel,
            SubscribeRequest,
            SubscribeRequestFilterTransactions,
        },
    },
    env_logger::Env,
};



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let cfg = Config::from_file("config.toml").unwrap();
    println!("Endpoint: {}", cfg.endpoint);

    setup_logging(&cfg);

    info!("Starting to monitor account: {}", cfg.pump_fun_fee_account);

    let mut client = setup_client(&cfg).await?;
    info!("Connected to gRPC endpoint");
    let (subscribe_tx, subscribe_rx) = client.subscribe().await?;
    
    send_subscription_request(subscribe_tx, &cfg).await?;
    info!("Subscription request sent. Listening for updates...");
    
    tokio::select! {
        _ = process_updates(subscribe_rx) => {},
        _ = signal::ctrl_c() => {
            info!("Received Ctrl-C, shutting down...");
        }
    }
    
    info!("Stream closed");
    Ok(())
}

/// Initialize the logging system
fn setup_logging(cfg: &Config) {
    let env = Env::default().filter_or("RUST_LOG", &cfg.rust_log_level);
    env_logger::Builder::from_env(env).init();
}

/// Create and connect to the gRPC client
async fn setup_client(cfg: &Config) -> Result<GeyserGrpcClient<impl Interceptor>, Box<dyn std::error::Error>> {
    
    info!("Connecting to gRPC endpoint: {}", cfg.endpoint);
    
    // Build the gRPC client with TLS config
    let client = GeyserGrpcClient::build_from_shared(cfg.endpoint.to_string())?
        .x_token(Some(cfg.auth_token.to_string()))?
        .tls_config(ClientTlsConfig::new().with_native_roots())?
        .connect()
        .await?;
    
    Ok(client)
}

/// Send the subscription request with transaction filters
async fn send_subscription_request<T>(
    mut tx: T,
    cfg: &Config
) -> Result<(), Box<dyn std::error::Error>> 
where 
    T: SinkExt<SubscribeRequest> + Unpin,
    <T as futures::Sink<SubscribeRequest>>::Error: std::error::Error + 'static,
{
    // Create account filter with the target accounts
    let mut accounts_filter = HashMap::new();
    accounts_filter.insert(
        "account_monitor".to_string(),
        SubscribeRequestFilterTransactions {
            account_include: vec![],
            account_exclude: vec![],
            account_required: vec![
                cfg.pump_fun_fee_account.to_string(), 
                cfg.pump_fun_program.to_string()
            ],
            vote: Some(false),
            failed: Some(false),
            signature: None,
        },
    );
    
    // Send subscription request
    tx.send(SubscribeRequest {
        transactions: accounts_filter,
        commitment: Some(CommitmentLevel::Processed as i32),
        ..Default::default()
    }).await?;
    
    Ok(())
}

/// Process updates from the stream
async fn process_updates<S>(
    mut stream: S,
) -> Result<(), Box<dyn std::error::Error>> 
where 
    S: StreamExt<Item = Result<SubscribeUpdate, Status>> + Unpin,
{
    while let Some(message) = stream.next().await {
        match message {
            Ok(msg) => handle_message(msg),
            Err(e) => {
                error!("Error receiving message: {:?}", e);
                break;
            }
        }
    }
    
    Ok(())
}

/// Handle an individual message from the stream
fn handle_message(msg: SubscribeUpdate) {
    //info!("Full Message: {:?}", msg);
    match msg.update_oneof {
        Some(UpdateOneof::Transaction(transaction_update)) => {
            if let Some(tx_info) = &transaction_update.transaction {
                let signature = &tx_info.signature;
                let tx_id = bs58::encode(signature).into_string();
                info!("Transaction update received! ID: {}", tx_id);
            } else {
                warn!("Transaction update received but no transaction info available");
            }
        },
        Some(other) => {
            info!("Other update received: {:?}", other);
        },
        None => {
            warn!("Empty update received");
        }
    }
}