use crate::certs::controller::{error, CertificateInterface, ImplCertificateInterface};
use crate::config::process::{ConfigInterface, ImplConfigInterface, Parameters};
use crate::httpservices::server::process;
use clap::Parser;
use custom_logger as log;
use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use rustls::ServerConfig;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

mod api;
mod certs;
mod command;
mod config;
mod error;
mod httpservices;

/// cli struct
#[derive(Parser, Debug)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(author = env!("CARGO_PKG_AUTHORS"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "A generic script executor for AI wokrflows", long_about = None)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// config file to use
    #[arg(short, long, value_name = "config")]
    pub config: String,
}

// used for lookup in read mode only
static MAP_LOOKUP: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);

fn main() {
    let args = Cli::parse();
    // we dont want to panic so ignore errors
    let _ = fs::create_dir_all("logs");
    let _ = fs::create_dir_all("staging");
    let _ = fs::create_dir_all("results");

    log::Logging::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .expect("should initiliase logging");

    let config = args.config;
    let impl_config = ImplConfigInterface {};
    let params = impl_config.read(config);
    if params.is_err() {
        log::error!("{}", params.err().unwrap().to_string());
        std::process::exit(1);
    }
    let parameters = params.as_ref().unwrap();
    // set up for (read) reference in auth handler
    let mut hm: HashMap<String, String> = HashMap::new();
    hm.insert(
        "certs_dir".to_string(),
        parameters.certs_dir.as_ref().unwrap().to_string(),
    );
    for (k, v) in parameters.agents.iter() {
        hm.insert(k.to_string(), v.to_string());
    }
    *MAP_LOOKUP.lock().unwrap() = Some(hm.clone());

    // use logging only for the inference and web service
    // setup logging
    let level = match parameters.log_level.as_str() {
        "info" => log::LevelFilter::Info,
        "debug" => log::LevelFilter::Debug,
        "trace" => log::LevelFilter::Trace,
        &_ => log::LevelFilter::Info,
    };

    // setup logging
    // avoid log panic
    let _ = log::Logging::new().with_level(level).init();

    // clean up semaphore
    if Path::new("semaphore.pid").exists() {
        let _ = fs::remove_file("semaphore.pid");
    }
    // Serve over HTTPS, with proper error handling.
    if let Err(e) = run_server(parameters.to_owned()) {
        log::error!("{}", e);
        std::process::exit(1);
    }
}

#[tokio::main]
async fn run_server(params: Parameters) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), params.port.parse()?);
    let certs_dir = params.certs_dir.unwrap_or("".to_string()).to_string();
    log::debug!("certs directory {}", certs_dir);
    let impl_certs = ImplCertificateInterface::new(params.cert_mode, Some(certs_dir));
    // Load public certificate.
    let certs = impl_certs.get_public_cert().await?;
    // Load private key.
    let key = impl_certs.get_private_cert().await?;
    log::info!("starting {} on https://{}", params.name, addr);
    // Create a TCP listener via tokio.
    let incoming = TcpListener::bind(&addr).await?;
    // Build TLS configuration.
    let mut server_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| error(e.to_string()))?;
    server_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec(), b"http/1.0".to_vec()];
    let tls_acceptor = TlsAcceptor::from(Arc::new(server_config));
    let service = service_fn(process);

    loop {
        let (tcp_stream, _remote_addr) = incoming.accept().await?;
        let tls_acceptor = tls_acceptor.clone();
        tokio::spawn(async move {
            let tls_stream = match tls_acceptor.accept(tcp_stream).await {
                Ok(tls_stream) => tls_stream,
                Err(err) => {
                    log::error!("failed to perform tls handshake: {err:#}");
                    return;
                }
            };
            if let Err(err) = Builder::new(TokioExecutor::new())
                .serve_connection(TokioIo::new(tls_stream), service)
                .await
            {
                log::error!("failed to serve connection: {err:#}");
            }
        });
    }
}
