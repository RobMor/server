use anyhow::{anyhow, Result};
use futures::{SinkExt, StreamExt};
use lazy_static::lazy_static;
use log::debug;
use openssl::pkey::Private;
use openssl::rsa;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::protocol::api;
use crate::protocol::codec::{ClientboundEncoder, ServerboundDecoder};
use crate::protocol::packets::{
    handshake, login, play, status, ClientboundPacket, IntoPacket, ServerboundPacket,
};

lazy_static! {
    static ref RSA_KEY: rsa::Rsa<Private> =
        rsa::Rsa::generate(1024).expect("Could not generate server key");
}

enum State {
    Handshaking,
    Status,
    Login,
    Encrypt,
    Play,
}

pub struct ConnectionHandler {
    // Login Information
    username: Option<String>,
    verify_token: Option<[u8; 4]>,

    current_state: State,

    reader: FramedRead<OwnedReadHalf, ServerboundDecoder>,
    writer: FramedWrite<OwnedWriteHalf, ClientboundEncoder>,
}

impl ConnectionHandler {
    pub fn new(socket: TcpStream) -> ConnectionHandler {
        let (socket_read, socket_write) = socket.into_split();

        ConnectionHandler {
            username: None,
            verify_token: None,

            current_state: State::Handshaking,
            reader: FramedRead::new(socket_read, ServerboundDecoder::new()),
            writer: FramedWrite::new(socket_write, ClientboundEncoder::new()),
        }
    }

    pub async fn execute(mut self) -> Result<()> {
        // The framed reader will close the stream when the connection is
        // closed.
        while let Some(msg) = self.reader.next().await {
            match msg {
                Ok(packet) => self.handle_packet(packet).await?,
                Err(err) => return Err(err),
            }
            self.writer.flush().await?;
        }

        // TODO pass off control of the connection to a play handler?

        Ok(())
    }

    async fn send(&mut self, packet: ClientboundPacket) -> Result<()> {
        self.writer.send(packet).await?;

        Ok(())
    }

    async fn handle_packet(&mut self, packet: ServerboundPacket) -> Result<()> {
        match self.current_state {
            State::Handshaking => match packet.packet_id() {
                0x00 => self.handle_handshake(packet.parse()?).await?,
                id => return Err(anyhow!("Unrecognized handshake packet id {}", id)),
            },
            State::Status => match packet.packet_id() {
                0x00 => self.handle_status_request(packet.parse()?).await?,
                0x01 => self.handle_status_ping(packet.parse()?).await?,
                id => return Err(anyhow!("Unrecognized status packet id {}", id)),
            },
            State::Login => match packet.packet_id() {
                0x00 => self.handle_login_start(packet.parse()?).await?,
                id => return Err(anyhow!("Unrecognized login packet id {}", id)),
            },
            State::Encrypt => match packet.packet_id() {
                0x01 => self.handle_login_encryption_response(packet.parse()?).await?,
                id => return Err(anyhow!("Unrecognized login packet id {}", id)),
            },
            State::Play => match packet.packet_id() {
                _ => unimplemented!(),
            },
        }

        Ok(())
    }

    async fn handle_handshake(&mut self, handshake: handshake::Handshake) -> Result<()> {
        debug!("handling handshake packet");

        self.current_state = match handshake.next_state() {
            handshake::NextState::Status => State::Status,
            handshake::NextState::Login => State::Login,
        };

        Ok(())
    }

    async fn handle_status_request(&mut self, _status: status::Request) -> Result<()> {
        debug!("handling status request packet");

        let response = status::Response::new(1337, 69, "Hello World".to_string(), None);
        self.send(response.into_packet()).await?;

        Ok(())
    }

    async fn handle_status_ping(&mut self, ping: status::Ping) -> Result<()> {
        debug!("handling status ping packet");

        let pong = status::Pong::new(ping);
        self.send(pong.into_packet()).await?;

        Ok(())
    }

    async fn handle_login_start(&mut self, start: login::Start) -> Result<()> {
        debug!("handling login start packet");

        let verify_token = rand::random();
        let public_key = RSA_KEY.public_key_to_der()?;

        let encryption_request = login::EncryptionRequest::new(public_key, verify_token);

        self.username = Some(start.username());
        self.verify_token = Some(verify_token);

        self.current_state = State::Encrypt;
        self.send(encryption_request.into_packet()).await?;

        Ok(())
    }

    async fn handle_login_encryption_response(
        &mut self,
        response: login::EncryptionResponse,
    ) -> Result<()> {
        debug!("handling login encryption response packet");

        let (encryped_shared_secret, encryped_verify_token) = response.into_parts();

        let mut shared_secret_decrypted = [0u8; 128];
        let num_bytes = RSA_KEY.private_decrypt(
            &encryped_shared_secret,
            &mut shared_secret_decrypted,
            rsa::Padding::PKCS1,
        )?;

        if num_bytes < 16 {
            return Err(anyhow!("Decryption of shared secret failed"));
        }

        let mut verify_token_decrypted = [0u8; 128];
        let num_bytes = RSA_KEY.private_decrypt(
            &encryped_verify_token,
            &mut verify_token_decrypted,
            rsa::Padding::PKCS1,
        )?;

        if num_bytes < 4 {
            return Err(anyhow!("Decryption of verify token failed"));
        }

        if self.verify_token.as_ref().unwrap() != &verify_token_decrypted[..4] {
            return Err(anyhow!("Verify token does not match"));
        }

        self.reader
            .decoder_mut()
            .enable_encryption(&shared_secret_decrypted[..16])?;

        self.writer
            .encoder_mut()
            .enable_encryption(&shared_secret_decrypted[..16])?;

        let uuid = api::authenticate(
            self.username.as_ref().unwrap(),
            &shared_secret_decrypted[..16],
            &RSA_KEY.public_key_to_der()?,
        )
        .await?;

        let success = login::Success::new(&uuid, self.username.as_ref().unwrap());

        self.send(success.into_packet()).await?;

        self.current_state = State::Play;

        // TODO move this elsewhere?
        // A play handler?

        let join_game = play::JoinGame::new(
            0,
            0,
            0,
            0,
            "flat".to_string(),
            4,
            false,
            true,
        );

        self.send(join_game.into_packet()).await?;

        Ok(())
    }
}
