use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use log::{error, info};
use tokio::net::TcpListener;
use tokio::stream::StreamExt;

mod connection;
mod protocol;

use connection::ConnectionHandler;

#[tokio::main]
async fn main() {
    simple_logger::init().unwrap();

    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 25565);
    let mut listener = TcpListener::bind(address)
        .await
        .map_err(|e| format!("Could not bind to {}: {}", address, e))
        .unwrap();

    let server = async move {
        let mut incoming = listener.incoming();

        while let Some(result) = incoming.next().await {
            match result {
                Ok(socket) => {
                    let peer_addr = socket.peer_addr().unwrap();
                    info!("Accepted connection from {}", peer_addr);

                    // Spawn a new task for each connection
                    tokio::spawn(async move {
                        let connection_handler = ConnectionHandler::new(socket);

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
