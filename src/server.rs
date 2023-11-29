use axum::{extract::connect_info::IntoMakeServiceWithConnectInfo, Router};
use hyper_util::rt::TokioIo;
use std::{io, net::SocketAddr};
use tokio::{net::TcpListener, signal, sync::watch};
use tower_service::Service;
use tracing::{debug, warn};

/// Serve the given router on the given TCP listener.
///
/// This supports graceful shutdown via SIGINT and SIGTERM.
///
/// # Errors
///
/// This function returns an error if accepting a connection fails.
pub async fn serve(
    tcp_listener: TcpListener,
    mut make_service: IntoMakeServiceWithConnectInfo<Router, SocketAddr>,
) -> io::Result<()> {
    let (tx, rx) = watch::channel(());
    loop {
        let (sock, addr) = tokio::select! {
            res = tcp_listener.accept() => res?,
            () = shutdown() => {
                debug!("shutdown signal received, not accepting new connections");
                break;
            }
        };
        debug!("connection {addr} accepted");
        let svc = make_service.call(addr).await.unwrap_or_else(|e| match e {});
        let rx = rx.clone();
        tokio::spawn(async move {
            let sock = TokioIo::new(sock);
            let hyper_svc = hyper::service::service_fn(move |req| svc.clone().call(req));
            // [`hyper_util::server::conn::auto::Builder`] supports both HTTP/1 and HTTP/2
            // but doesn't support graceful, so we unfortunately have to pick
            // one.
            let conn = hyper::server::conn::http1::Builder::new()
                .serve_connection(sock, hyper_svc)
                .with_upgrades();
            let mut conn = std::pin::pin!(conn);
            loop {
                tokio::select! {
                    res = conn.as_mut() => {
                        if let Err(e) = res {
                            debug!("failed to serve connection: {e:#}");
                        }
                        break;
                    }
                    () = shutdown() => {
                        debug!("shutdown signal received, starting graceful shutdown");
                        conn.as_mut().graceful_shutdown();
                    }
                }
            }
            debug!("connection {addr} closed");
            drop(rx);
        });
    }
    drop(rx);
    drop(tcp_listener);
    debug!("waiting for {} tasks to finish", tx.receiver_count());
    tx.closed().await;
    Ok(())
}

async fn shutdown() {
    let ctrlc = async { signal::ctrl_c().await.expect("failed to install ^C handler") };
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        () = ctrlc => warn!("Initiating graceful shutdown"),
        () = terminate => warn!("Initiating graceful shutdown"),
    }
}
