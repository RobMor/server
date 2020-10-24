use std::convert::TryInto;
use std::error::Error;
use std::fmt;

use bytes::{Buf, BufMut, BytesMut};

use uuid::Uuid;

pub type Result<T> = std::result::Result<T, DataTypeError>;

// TODO resarch rust documentation best practices...
pub trait DataType: Sized {
    /// Read the data type from the source buffer. This will consume the data in
    /// the buffer in the process of reaing it. If `Err` is returned, the buffer
    /// is in an undefined state and should not be read from any longer.
    fn read_from(src: &mut BytesMut) -> Result<Self>;
    /// Write the data type to a destination buffer. Does not try to reserve
    /// space in the destination buffer, so there should be enough space in the
    /// buffer to accommodate `self.size()` bytes already.
    fn write_to(self, dst: &mut BytesMut);
    /// Returns the size in bytes of this instance of the data type.
    fn size(&self) -> usize;
}

pub trait SizedDataType: Sized {
    fn read_from_sized(src: &mut BytesMut, size: usize) -> Result<Self>;
    fn write_to(self, dst: &mut BytesMut);
    fn size(&self) -> usize;
}

#[derive(Debug)]
pub enum DataTypeError {
    OutOfBytes(String),
    Malformed(String, String),
    Context(Box<DataTypeError>, String),
}

impl fmt::Display for DataTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfBytes(s) => write!(f, "Ran out of bytes reading {}", s),
            Self::Malformed(s1, s2) => {
                write!(f, "Encountered malformed data while reading {}: {}", s1, s2)
            }
            Self::Context(_, s) => write!(f, "{}", s),
        }
    }
}

impl Error for DataTypeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let Self::Context(other, _) = self {
            Some(other)
        } else {
            None
        }
    }
}

impl DataTypeError {
    fn add_context(self, context: impl Into<String>) -> DataTypeError {
        DataTypeError::Context(Box::new(self), context.into())
    }
}

impl DataType for bool {
    fn read_from(src: &mut BytesMut) -> Result<bool> {
        if src.remaining() >= 1 {
            Ok(src.get_u8() == 0x01)
        } else {
            Err(DataTypeError::OutOfBytes("Boolean".to_string()))
        }
    }

    fn write_to(self, dst: &mut BytesMut) {
        if self {
            dst.put_u8(0);
        } else {
            dst.put_u8(1);
        }
    }

    fn size(&self) -> usize {
        1
    }
}

pub type Byte = i8;

impl DataType for Byte {
    fn read_from(src: &mut BytesMut) -> Result<Byte> {
        if src.remaining() >= 1 {
            Ok(src.get_i8())
        } else {
            Err(DataTypeError::OutOfBytes("Byte".to_string()))
        }
    }

    fn write_to(self, dst: &mut BytesMut) {
        dst.put_i8(self)
    }

    fn size(&self) -> usize {
        1
    }
}

pub type UnsignedByte = u8;

impl DataType for UnsignedByte {
    fn read_from(src: &mut BytesMut) -> Result<UnsignedByte> {
        if src.remaining() >= 1 {
            Ok(src.get_u8())
        } else {
            Err(DataTypeError::OutOfBytes("Unsigned Byte".to_string()))
        }
    }

    fn write_to(self, dst: &mut BytesMut) {
        dst.put_u8(self)
    }

    fn size(&self) -> usize {
        1
    }
}

pub type Short = i16;

impl DataType for Short {
    fn read_from(src: &mut BytesMut) -> Result<Short> {
        if src.remaining() >= 2 {
            Ok(src.get_i16())
        } else {
            Err(DataTypeError::OutOfBytes("Short".to_string()))
        }
    }

    fn write_to(self, dst: &mut BytesMut) {
        dst.put_i16(self)
    }

    fn size(&self) -> usize {
        2
    }
}

pub type UnsignedShort = u16;

impl DataType for UnsignedShort {
    fn read_from(src: &mut BytesMut) -> Result<UnsignedShort> {
        if src.remaining() >= 2 {
            Ok(src.get_u16())
        } else {
            Err(DataTypeError::OutOfBytes("Unsigned Short".to_string()))
        }
    }

    fn write_to(self, dst: &mut BytesMut) {
        dst.put_u16(self)
    }

    fn size(&self) -> usize {
        2
    }
}

pub type Int = i32;

impl DataType for Int {
    fn read_from(src: &mut BytesMut) -> Result<Int> {
        if src.remaining() >= 2 {
            Ok(src.get_i32())
        } else {
            Err(DataTypeError::OutOfBytes("Int".to_string()))
        }
    }

    fn write_to(self, dst: &mut BytesMut) {
        dst.put_i32(self)
    }

    fn size(&self) -> usize {
        4
    }
}

pub type Long = i64;

impl DataType for Long {
    fn read_from(src: &mut BytesMut) -> Result<Long> {
        if src.remaining() >= 2 {
            Ok(src.get_i64())
        } else {
            Err(DataTypeError::OutOfBytes("Long".to_string()))
        }
    }

    fn write_to(self, dst: &mut BytesMut) {
        dst.put_i64(self)
    }

    fn size(&self) -> usize {
        8
    }
}

pub type Float = f32;

impl DataType for Float {
    fn read_from(src: &mut BytesMut) -> Result<Float> {
        if src.remaining() >= 2 {
            Ok(src.get_f32())
        } else {
            Err(DataTypeError::OutOfBytes("Float".to_string()))
        }
    }

    fn write_to(self, dst: &mut BytesMut) {
        dst.put_f32(self)
    }

    fn size(&self) -> usize {
        4
    }
}

pub type Double = f64;

impl DataType for Double {
    fn read_from(src: &mut BytesMut) -> Result<Double> {
        if src.remaining() >= 2 {
            Ok(src.get_f64())
        } else {
            Err(DataTypeError::OutOfBytes("Double".to_string()))
        }
    }

    fn write_to(self, dst: &mut BytesMut) {
        dst.put_f64(self)
    }

    fn size(&self) -> usize {
        4
    }
}

pub const MAX_STRING_LENGTH: usize = 32767;

impl SizedDataType for String {
    fn read_from_sized(src: &mut BytesMut, size: usize) -> Result<String> {
        debug_assert!(size <= MAX_STRING_LENGTH);

        // Prefixed with its size in bytes
        let length = VarInt::read_from(src)
            .map_err(|e| e.add_context("While reading String length header"))?;

        let length: usize = length.value().try_into().map_err(|_| {
            DataTypeError::Malformed(
                "String".to_string(),
                "bad value for length prefix".to_string(),
            )
        })?;

        if length > size {
            return Err(DataTypeError::Malformed(
                "String".to_string(),
                format!("length header too large for string of max size {}", size),
            ));
        }

        if src.remaining() >= length {
            let data = src.split_to(length);

            Ok(String::from_utf8(data.as_ref().into()).map_err(|e| {
                DataTypeError::Malformed(
                    "String".to_string(),
                    format!("malformed UTF8 string: {}", e),
                )
            })?)
        } else {
            Err(DataTypeError::OutOfBytes("String".to_string()))
        }
    }

    fn write_to(self, dst: &mut BytesMut) {
        let length = VarInt::new(self.len() as i32);

        length.write_to(dst);
        dst.extend_from_slice(self.as_bytes());
    }

    fn size(&self) -> usize {
        let length_header = VarInt::new(self.len() as i32);
        length_header.size() + self.len()
    }
}

pub struct Chat {
    message: String,
}

impl Chat {
    pub fn new(message: String) -> Chat {
        Chat { message }
    }
}

impl DataType for Chat {
    fn read_from(src: &mut BytesMut) -> Result<Chat> {
        Ok(Chat {
            message: String::read_from_sized(src, 32767)?,
        })
    }

    fn write_to(self, dst: &mut BytesMut) {
        self.message.write_to(dst)
    }

    fn size(&self) -> usize {
        self.message.size()
    }
}

// TODO
pub struct Identifier {
    identifier: String,
}

impl Identifier {
    pub fn new(identifier: String) -> Identifier {
        Identifier { identifier }
    }
}

impl DataType for Identifier {
    fn read_from(src: &mut BytesMut) -> Result<Identifier> {
        Ok(Identifier {
            identifier: String::read_from_sized(src, 32767)?,
        })
    }

    fn write_to(self, dst: &mut BytesMut) {
        self.identifier.write_to(dst)
    }

    fn size(&self) -> usize {
        self.identifier.size()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct VarInt {
    value: i32,
}

impl VarInt {
    pub fn new(value: i32) -> VarInt {
        VarInt { value }
    }

    pub fn value(&self) -> i32 {
        self.value
    }

    /// Reads a var int from the input buffer, being careful to leave the buffer
    /// alone in the case of failure. This is useful when reading packet headers
    /// that might have a partially loaded VarInt.
    pub fn careful_read_from(src: &mut BytesMut) -> Result<VarInt> {
        let mut result = 0;

        // Get an iterator over the bytes in this stream.
        // Comes from the bytes as a slice so there is no advancing being done.
        for (i, byte) in src.as_ref().iter().enumerate() {
            // VarInts are never longer than 5 bytes
            if i + 1 > 5 {
                return Err(DataTypeError::Malformed(
                    "VarInt".to_string(),
                    "too many bytes".to_string(),
                ));
            }

            let value = byte & 0x7F;

            // Bytes arrive in least to most significant order
            result |= (value as i32) << (7 * (i));

            // The high bit of every byte tells us if there's another byte to
            // decode
            if byte & 0x80 == 0 {
                // Advance the buffer upon success
                src.advance(i + 1);
                return Ok(VarInt::new(result));
            }
        }

        Err(DataTypeError::OutOfBytes("VarInt".to_string()))
    }
}

impl DataType for VarInt {
    fn read_from(src: &mut BytesMut) -> Result<VarInt> {
        let mut num_read = 0;
        let mut result = 0;

        while !src.is_empty() {
            let byte = src.get_u8();

            num_read += 1;

            // VarInts are never longer than 5 bytes
            if num_read > 5 {
                return Err(DataTypeError::Malformed(
                    "VarInt".to_string(),
                    "too many bytes".to_string(),
                ));
            }

            let value = byte & 0x7F;

            // Bytes arrive in least to most significant order
            result |= (value as i32) << (7 * (num_read - 1));

            // The high bit of every byte tells us if there's another byte to
            // decode
            if byte & 0x80 == 0 {
                return Ok(VarInt::new(result));
            }
        }

        Err(DataTypeError::OutOfBytes("VarInt".to_string()))
    }

    fn write_to(self, dst: &mut BytesMut) {
        // Integer types don't allow for logical shifting (meaning shifting the
        // sign bit as well) which is why we cast to u32 here.
        let mut value = self.value as u32;

        // Execute loop at least once to handle the zero case.
        loop {
            let mut byte: u8 = (value & 0x7F) as u8;

            // Least significant to most significant order
            value >>= 7;

            // The high bit of the byte indicates whether there is another
            // byte to decode
            if value != 0 {
                byte |= 0x80;
            }

            dst.put_u8(byte);

            if value == 0 {
                break;
            }
        }
    }

    fn size(&self) -> usize {
        // TODO there might be a micro optimization here...
        let num_bits = (32 - self.value.leading_zeros()) as f32;
        std::cmp::max((num_bits / 7.0).ceil() as usize, 1)
    }
}

// TODO
#[derive(Debug)]
pub struct VarLong {
    pub value: i64,
}

impl VarLong {
    fn new(value: i64) -> VarLong {
        VarLong { value }
    }

    fn value(&self) -> i64 {
        self.value
    }
}

impl DataType for VarLong {
    fn read_from(src: &mut BytesMut) -> Result<VarLong> {
        let mut num_read = 0;
        let mut result = 0;

        while !src.is_empty() {
            let byte = src.get_u8();

            num_read += 1;

            // VarLongs are never longer than 5 bytes
            if num_read > 10 {
                return Err(DataTypeError::Malformed(
                    "VarLong".to_string(),
                    "too many bytes".to_string(),
                ));
            }

            let value = byte & 0x7F;

            // Bytes arrive in least to most significant order
            result |= (value as i64) << (7 * (num_read - 1));

            // The high bit of every byte tells us if there's another byte to
            // decode
            if byte & 0x80 == 0 {
                return Ok(VarLong::new(result));
            }
        }

        Err(DataTypeError::OutOfBytes("VarLong".to_string()))
    }

    fn write_to(self, dst: &mut BytesMut) {
        // Integer types don't allow for logical shifting (meaning shifting the
        // sign bit as well) which is why we cast to u64 here.
        let mut value = self.value as u64;

        // Execute loop at least once to handle the zero case.
        loop {
            let mut byte: u8 = (value & 0x7F) as u8;

            // Least significant to most significant order
            value >>= 7;

            // The high bit of the byte indicates whether there is another
            // byte to decode
            if value != 0 {
                byte |= 0x80;
            }

            dst.put_u8(byte);

            if value == 0 {
                break;
            }
        }
    }

    fn size(&self) -> usize {
        // TODO there might be a micro optimization here...
        let num_bits = (64 - self.value.leading_zeros()) as f32;
        std::cmp::max((num_bits / 7.0).ceil() as usize, 1)
    }
}

#[derive(Debug)]
pub struct Position {
    pub x: i32,
    pub z: i32,
    pub y: i16,
}

impl DataType for Position {
    fn read_from(src: &mut BytesMut) -> Result<Position> {
        // TODO this needs testing
        if src.remaining() >= 8 {
            let val = src.get_u64();

            // x: 26 MSBs
            // z: middle 26 bits (after x)
            // y: 12 LSBs
            let mut x = (val >> 38) as i32;
            let mut z = (val << 26 >> 38) as i32;
            let mut y = (val & 0xFFF) as i16;

            // The numbers are two's complement 26 and 16 bit integers so we
            // have to manually two's complement them.
            if x >= (1 << 25) {
                x -= 1 << 26
            }
            if z >= (1 << 25) {
                z -= 1 << 26
            }
            if y >= (1 << 11) {
                y -= 1 << 12
            }

            Ok(Position { x, z, y })
        } else {
            Err(DataTypeError::OutOfBytes("Position".to_string()))
        }
    }

    fn write_to(self, dst: &mut BytesMut) {
        // TODO this needs testing...
        let mut x = self.x;
        let mut y = self.y;
        let mut z = self.z;

        if x < 0 {
            x += 1 << 25
        }
        if y < 0 {
            y += 1 << 12
        }
        if z < 0 {
            z += 1 << 26
        }

        let val = (((x & 0x3FFFFFF) as u64) << 38)
            | (((z & 0x3FFFFFF) as u64) << 12)
            | ((y & 0xFFF) as u64);

        dst.put_u64(val)
    }

    fn size(&self) -> usize {
        8
    }
}

#[derive(Debug)]
pub struct Angle {
    /// The number of 1/256 steps of a full turn
    pub steps: u8,
}

impl DataType for Angle {
    fn read_from(src: &mut BytesMut) -> Result<Angle> {
        if src.remaining() >= 1 {
            Ok(Angle {
                steps: src.get_u8(),
            })
        } else {
            Err(DataTypeError::OutOfBytes("Angle".to_string()))
        }
    }

    fn write_to(self, dst: &mut BytesMut) {
        dst.put_u8(self.steps)
    }

    fn size(&self) -> usize {
        1
    }
}

impl DataType for Uuid {
    fn read_from(src: &mut BytesMut) -> Result<Uuid> {
        if src.remaining() >= 16 {
            Ok(Uuid::from_u128(src.get_u128()))
        } else {
            Err(DataTypeError::OutOfBytes("UUID".to_string()))
        }
    }

    fn write_to(self, dst: &mut BytesMut) {
        dst.put_u128(self.as_u128())
    }

    fn size(&self) -> usize {
        16
    }
}

impl<T: DataType> SizedDataType for Vec<T> {
    fn read_from_sized(src: &mut BytesMut, size: usize) -> Result<Vec<T>> {
        let array_size = VarInt::read_from(src)?.value() as usize;

        if array_size > size {
            return Err(DataTypeError::Malformed(
                "ByteArray".to_string(),
                format!(
                    "header length {} longer than max size of {}",
                    array_size, size
                ),
            ));
        }

        let mut vec = Vec::with_capacity(array_size);

        while vec.len() < array_size {
            match T::read_from(src) {
                Ok(v) => vec.push(v),
                Err(DataTypeError::OutOfBytes(s)) => {
                    return Err(DataTypeError::OutOfBytes(format!("Array of {}", s)))
                }
                Err(e) => {
                    return Err(DataTypeError::Context(
                        Box::new(e),
                        "Error parsing element of Array".to_string(),
                    ))
                }
            }
        }

        Ok(vec)
    }

    fn write_to(self, dst: &mut BytesMut) {
        let length = VarInt::new(self.len() as i32);

        length.write_to(dst);

        for v in self {
            v.write_to(dst);
        }
    }

    fn size(&self) -> usize {
        VarInt::new(self.len() as i32).size() + self.len()
    }
}

/// A much faster implementation for a vector of bytes but since we can't have
/// both this and the generic implementation I've opted for the ergonomics of
/// the generics...
// impl SizedDataType for Vec<u8> {
//     fn read_from_sized(src: &mut BytesMut, size: usize) -> Result<ByteArray> {
//         let array_size = VarInt::read_from(src)?.value() as usize;

//         if array_size > size {
//             return Err(DataTypeError::Malformed(
//                 "ByteArray".to_string(),
//                 format!(
//                     "header length {} longer than max size of {}",
//                     array_size, size
//                 ),
//             ));
//         }

//         if src.remaining() >= array_size {
//             Ok(src.split_to(array_size).as_ref().into())
//         } else {
//             Err(DataTypeError::OutOfBytes("ByteArray".to_string()))
//         }
//     }

//     fn write_to(self, dst: &mut BytesMut) {
//         let length = VarInt::new(self.len() as i32);

//         length.write_to(dst);
//         dst.extend_from_slice(&self);
//     }

//     fn size(&self) -> usize {
//         VarInt::new(self.len() as i32).size() + self.len()
//     }
// }

#[cfg(test)]
mod tests {
    use crate::protocol::data_types::*;

    #[test]
    fn var_int_basic_read() {
        // From the wiki.vg protocol page
        let mut bytes = BytesMut::with_capacity(23);
        bytes.extend_from_slice(&[0x00]); // 0
        bytes.extend_from_slice(&[0x01]); // 1
        bytes.extend_from_slice(&[0x02]); // 2
        bytes.extend_from_slice(&[0x7f]); // 127
        bytes.extend_from_slice(&[0x80, 0x01]); // 128
        bytes.extend_from_slice(&[0xff, 0x01]); // 255
        bytes.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0x07]); // 2147483647
        bytes.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0x0f]); // -1
        bytes.extend_from_slice(&[0x80, 0x80, 0x80, 0x80, 0x08]); // -2147483648

        assert_eq!(0, VarInt::read_from(&mut bytes).unwrap().value());
        assert_eq!(1, VarInt::read_from(&mut bytes).unwrap().value());
        assert_eq!(2, VarInt::read_from(&mut bytes).unwrap().value());
        assert_eq!(127, VarInt::read_from(&mut bytes).unwrap().value());
        assert_eq!(128, VarInt::read_from(&mut bytes).unwrap().value());
        assert_eq!(255, VarInt::read_from(&mut bytes).unwrap().value());
        assert_eq!(2147483647, VarInt::read_from(&mut bytes).unwrap().value());
        assert_eq!(-1, VarInt::read_from(&mut bytes).unwrap().value());
        assert_eq!(-2147483648, VarInt::read_from(&mut bytes).unwrap().value());
    }

    #[test]
    fn var_int_basic_write() {
        let mut bytes = BytesMut::with_capacity(5);

        VarInt::new(0).write_to(&mut bytes);
        assert_eq!(bytes.as_ref(), &[0x00]);

        bytes.clear();

        VarInt::new(1).write_to(&mut bytes);
        assert_eq!(bytes.as_ref(), &[0x01]);

        bytes.clear();

        VarInt::new(2).write_to(&mut bytes);
        assert_eq!(bytes.as_ref(), &[0x02]);

        bytes.clear();

        VarInt::new(127).write_to(&mut bytes);
        assert_eq!(bytes.as_ref(), &[0x7f]);

        bytes.clear();

        VarInt::new(128).write_to(&mut bytes);
        assert_eq!(bytes.as_ref(), &[0x80, 0x01]);

        bytes.clear();

        VarInt::new(255).write_to(&mut bytes);
        assert_eq!(bytes.as_ref(), &[0xff, 0x01]);

        bytes.clear();

        VarInt::new(2147483647).write_to(&mut bytes);
        assert_eq!(bytes.as_ref(), &[0xff, 0xff, 0xff, 0xff, 0x07]);

        bytes.clear();

        VarInt::new(-1).write_to(&mut bytes);
        assert_eq!(bytes.as_ref(), &[0xff, 0xff, 0xff, 0xff, 0x0f]);

        bytes.clear();

        VarInt::new(-2147483648).write_to(&mut bytes);
        assert_eq!(bytes.as_ref(), &[0x80, 0x80, 0x80, 0x80, 0x08]);
    }

    #[test]
    fn var_int_not_enough_bytes() {
        // Valid VarInts will end with a byte with a zero as MSB
        let mut bytes = BytesMut::with_capacity(1);
        bytes.extend_from_slice(&[0x80]);

        assert!(matches!(
            VarInt::read_from(&mut bytes),
            Err(DataTypeError::OutOfBytes(_))
        ));
    }

    #[test]
    fn var_int_longer_than_five() {
        // Valid VarInts never exceed five bytes, and the parser should fail
        // gracefully
        let mut bytes = BytesMut::with_capacity(6);
        bytes.extend_from_slice(&[0x80, 0x80, 0x80, 0x80, 0x80, 0x00]);

        assert!(matches!(
            VarInt::read_from(&mut bytes),
            Err(DataTypeError::Malformed(_, _))
        ));
    }

    #[test]
    fn var_int_careful_read() {
        // Valid VarInts will end with a byte with a zero as MSB
        let mut bytes = BytesMut::with_capacity(1);
        bytes.extend_from_slice(&[0x80]);

        assert!(matches!(
            VarInt::careful_read_from(&mut bytes),
            Err(DataTypeError::OutOfBytes(_))
        ));

        // Shouldn't touch the buffer on failure
        assert_eq!(bytes.len(), 1);
    }

    #[test]
    fn var_long_basic_read() {
        // From the wiki.vg protocol page
        let mut bytes = BytesMut::with_capacity(52);
        bytes.extend_from_slice(&[0x00]); // 0
        bytes.extend_from_slice(&[0x01]); // 1
        bytes.extend_from_slice(&[0x02]); // 2
        bytes.extend_from_slice(&[0x7f]); // 127
        bytes.extend_from_slice(&[0x80, 0x01]); // 128
        bytes.extend_from_slice(&[0xff, 0x01]); // 255
        bytes.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0x07]); // 2147483647
        bytes.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f]); // 9223372036854775807
        bytes.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01]); // -1
        bytes.extend_from_slice(&[0x80, 0x80, 0x80, 0x80, 0xf8, 0xff, 0xff, 0xff, 0xff, 0x01]); // -2147483648
        bytes.extend_from_slice(&[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01]); // -9223372036854775808

        assert_eq!(0, VarLong::read_from(&mut bytes).unwrap().value());
        assert_eq!(1, VarLong::read_from(&mut bytes).unwrap().value());
        assert_eq!(2, VarLong::read_from(&mut bytes).unwrap().value());
        assert_eq!(127, VarLong::read_from(&mut bytes).unwrap().value());
        assert_eq!(128, VarLong::read_from(&mut bytes).unwrap().value());
        assert_eq!(255, VarLong::read_from(&mut bytes).unwrap().value());
        assert_eq!(2147483647, VarLong::read_from(&mut bytes).unwrap().value());
        assert_eq!(
            9223372036854775807,
            VarLong::read_from(&mut bytes).unwrap().value()
        );
        assert_eq!(-1, VarLong::read_from(&mut bytes).unwrap().value());
        assert_eq!(-2147483648, VarLong::read_from(&mut bytes).unwrap().value());
        assert_eq!(
            -9223372036854775808,
            VarLong::read_from(&mut bytes).unwrap().value()
        );
    }

    #[test]
    fn var_long_basic_write() {
        let mut bytes = BytesMut::with_capacity(10);
        VarLong::new(0).write_to(&mut bytes);
        assert_eq!(bytes.as_ref(), &[0x00]);

        bytes.clear();

        VarLong::new(1).write_to(&mut bytes);
        assert_eq!(bytes.as_ref(), &[0x01]);

        bytes.clear();

        VarLong::new(2).write_to(&mut bytes);
        assert_eq!(bytes.as_ref(), &[0x02]);

        bytes.clear();

        VarLong::new(127).write_to(&mut bytes);
        assert_eq!(bytes.as_ref(), &[0x7f]);

        bytes.clear();

        VarLong::new(128).write_to(&mut bytes);
        assert_eq!(bytes.as_ref(), &[0x80, 0x01]);

        bytes.clear();

        VarLong::new(255).write_to(&mut bytes);
        assert_eq!(bytes.as_ref(), &[0xff, 0x01]);

        bytes.clear();

        VarLong::new(2147483647).write_to(&mut bytes);
        assert_eq!(bytes.as_ref(), &[0xff, 0xff, 0xff, 0xff, 0x07]);

        bytes.clear();

        VarLong::new(9223372036854775807).write_to(&mut bytes);
        assert_eq!(
            bytes.as_ref(),
            &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f]
        );

        bytes.clear();

        VarLong::new(-1).write_to(&mut bytes);
        assert_eq!(
            bytes.as_ref(),
            &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01]
        );

        bytes.clear();

        VarLong::new(-2147483648).write_to(&mut bytes);
        assert_eq!(
            bytes.as_ref(),
            &[0x80, 0x80, 0x80, 0x80, 0xf8, 0xff, 0xff, 0xff, 0xff, 0x01]
        );

        bytes.clear();

        VarLong::new(-9223372036854775808).write_to(&mut bytes);
        assert_eq!(
            bytes.as_ref(),
            &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01]
        );
    }

    #[test]
    fn var_long_not_enough_bytes() {
        // Valid VarInts will end with a byte with a zero as MSB
        let mut bytes = BytesMut::with_capacity(1);
        bytes.extend_from_slice(&[0x80]);

        assert!(matches!(
            VarLong::read_from(&mut bytes),
            Err(DataTypeError::OutOfBytes(_))
        ));
    }

    #[test]
    fn var_long_longer_than_ten() {
        // Valid VarInts never exceed five bytes, and the parser should fail
        // gracefully
        let mut bytes = BytesMut::with_capacity(6);
        bytes.extend_from_slice(&[
            0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x00,
        ]);

        assert!(matches!(
            VarLong::read_from(&mut bytes),
            Err(DataTypeError::Malformed(_, _))
        ));
    }
}
