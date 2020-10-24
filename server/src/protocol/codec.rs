use std::convert::TryInto;

use anyhow::{Context, Error};
use bytes::BytesMut;
use log::{info, trace};
use openssl::symm::{Cipher, Crypter, Mode};
use tokio_util::codec::{Decoder, Encoder};

use crate::protocol::data_types::{DataType, DataTypeError, VarInt};
use crate::protocol::packets::{ClientboundPacket, ServerboundPacket};

pub struct ServerboundDecoder {
    /// An OpenSSL cipher that will be Some when encryption is enabled.
    decrypter: Option<Crypter>,
    /// An internal buffer that buffers decrypted packet data for the decoder.
    buffer: BytesMut,
}

impl ServerboundDecoder {
    pub fn new() -> ServerboundDecoder {
        ServerboundDecoder {
            decrypter: None,
            buffer: BytesMut::new(),
        }
    }

    pub fn enable_encryption(&mut self, key: &[u8]) -> anyhow::Result<()> {
        info!("decoder enabling encryption");

        self.decrypter = Some(Crypter::new(
            Cipher::aes_128_cfb8(),
            Mode::Decrypt,
            key,
            Some(key), // Both sides use the shared secret as the Key and IV
        )?);

        Ok(())
    }
}

impl Decoder for ServerboundDecoder {
    type Item = ServerboundPacket;
    type Error = Error;

    fn decode(&mut self, mut src: &mut BytesMut) -> Result<Option<ServerboundPacket>, Error> {
        // When encryption is disabled it's faster to read from the source
        // buffer. When encryption is enabled we have to read from our internal
        // buffer (after decrypting into it).
        let mut read_from = if let Some(decrypter) = self.decrypter.as_mut() {
            let start = self.buffer.len();
            let new_data = src.split();

            // The OpenSSL api doesn't allow for decryption in place.
            // Unfortunately this means heap allocations for every packet.
            // TODO some Rust libraries allow in place decryption (how much do we care about
            // security?
            self.buffer.resize(new_data.len() + 16 + start, 0);

            let num_decrypted = decrypter.update(&new_data, &mut self.buffer[start..])?;

            self.buffer.truncate(start + num_decrypted);

            &mut self.buffer
        } else {
            &mut src
        };

        let packet_length = match VarInt::careful_read_from(&mut read_from) {
            Ok(v) => v.value() as usize,
            Err(DataTypeError::OutOfBytes(_)) => {
                src.reserve(5);
                return Ok(None);
            }
            Err(e) => return Err(e.into()),
        };

        trace!("packet length: {} bytes", packet_length);

        if packet_length <= read_from.len() {
            trace!("enough bytes in source buffer");

            let mut packet_data = read_from.split_to(packet_length);
            let packet_id = VarInt::read_from(&mut packet_data)?;

            trace!("packet ID: {:#04x}", packet_id.value());

            // Reserve space in the buffer for a max size VarInt.
            src.reserve(5);
            Ok(Some(ServerboundPacket::new(packet_id.value(), packet_data)))
        } else {
            trace!("not enough bytes in source buffer");

            // Reserve space for the rest of this packet.
            src.reserve(packet_length);
            Ok(None)
        }
    }
}

pub enum EncoderError {}

pub struct ClientboundEncoder {
    encrypter: Option<Crypter>,
}

impl ClientboundEncoder {
    pub fn new() -> ClientboundEncoder {
        ClientboundEncoder { encrypter: None }
    }

    pub fn enable_encryption(&mut self, key: &[u8]) -> anyhow::Result<()> {
        info!("encoder enabling encryption");

        self.encrypter = Some(Crypter::new(
            Cipher::aes_128_cfb8(),
            Mode::Encrypt,
            key,
            Some(key), // Both sides use the shared secret as the Key and IV
        )?);
        Ok(())
    }
}

impl Encoder<ClientboundPacket> for ClientboundEncoder {
    type Error = Error;

    fn encode(&mut self, item: ClientboundPacket, dst: &mut BytesMut) -> Result<(), Error> {
        // TODO reduce the number of allocations here...
        let packet_id = VarInt::new(item.packet_id());
        let data = item.data();

        let buffer_length = (packet_id.size() + data.len())
            .try_into()
            .context("Packet length exceeds size of 32 bit integer")?;
        let buffer_length = VarInt::new(buffer_length);

        let length = buffer_length.size() + buffer_length.value() as usize;
        dst.reserve(length + 16); // 16 is cipher block size

        // TODO
        if let Some(encrypter) = self.encrypter.as_mut() {
            let mut temp = BytesMut::with_capacity(length);
            buffer_length.write_to(&mut temp);
            packet_id.write_to(&mut temp);
            temp.extend_from_slice(data.as_ref());

            dst.resize(length, 0);
            encrypter.update(&temp, dst)?;
        } else {
            buffer_length.write_to(dst);
            packet_id.write_to(dst);
            dst.extend_from_slice(data.as_ref());
        }

        Ok(())
    }
}
