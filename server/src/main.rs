use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use log::{error, info};
use openssl::pkey::Private;
use openssl::rsa;
use simple_logger::SimpleLogger;
use tokio::net::TcpListener;
use tokio::stream::StreamExt;

#[macro_use]
extern crate mcserver_macros;

mod api;
mod protocol;

use protocol::connection::ConnectionHandler;

#[tokio::main]
async fn main() {
    SimpleLogger::new().init().unwrap();

    let rsa_key = Arc::new(rsa::Rsa::generate(1024).expect("Could not generate server key"));

    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 25565);
    let mut listener = TcpListener::bind(address)
        .await
        .map_err(|e| format!("Could not bind to {}: {}", address, e))
        .unwrap();

    let server = async move {
        while let Some(result) = listener.next().await {
            match result {
                Ok(socket) => {
                    let peer_addr = socket.peer_addr().unwrap();
                    info!("Accepted connection from {}", peer_addr);

                    let key_copy = rsa_key.clone();
                    // Spawn a new task for each connection
                    tokio::spawn(async move {
                        let connection_handler = ConnectionHandler::new(key_copy, socket);

                        let result = connection_handler.execute().await;

                        match result {
                            Ok(_) => info!("{} - connection closed with no problems", peer_addr),
                            Err(err) => error!(
                                "{} - connection closed with error: {}",
                                peer_addr,
                                err.to_string()
                            ),
                        }
                    });
                }
                Err(err) => error!("Accept error: {}", err),
            }
        }
    };

    info!("Server listening on {}", address);

    server.await;
}
