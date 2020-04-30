//! A thin wrapper of [prost](https://docs.rs/prost/0.6.1/prost/)

/// A labcodec message.
pub trait Message: prost::Message + Default {}
impl<T: prost::Message + Default> Message for T {}

/// A message encoding error.
pub type EncodeError = prost::EncodeError;
/// A message decoding error.
pub type DecodeError = prost::DecodeError;

/// Encodes the message to a `Vec<u8>`.
pub fn encode<M: Message>(message: &M, buf: &mut Vec<u8>) -> Result<(), EncodeError> {
    buf.reserve(message.encoded_len());
    message.encode(buf)?;
    Ok(())
}

/// Decodes an message from the buffer.
pub fn decode<M: Message>(buf: &[u8]) -> Result<M, DecodeError> {
    M::decode(buf)
}

#[cfg(test)]
mod tests {
    mod fixture {
        // The generated rust file:
        // labs6824/target/debug/build/labcodec-hashhashhashhash/out/fixture.rs
        //
        // It looks like:
        //
        // ```no_run
        // /// A simple protobuf message.
        // #[derive(Clone, PartialEq, Message)]
        // pub struct Msg {
        //     #[prost(enumeration="msg::Type", tag="1")]
        //     pub type_: i32,
        //     #[prost(uint64, tag="2")]
        //     pub id: u64,
        //     #[prost(string, tag="3")]
        //     pub name: String,
        //     #[prost(bytes, repeated, tag="4")]
        //     pub paylad: ::std::vec::Vec<Vec<u8>>,
        // }
        // pub mod msg {
        //     #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Enumeration)]
        //     pub enum Type {
        //         Unknown = 0,
        //         Put = 1,
        //         Get = 2,
        //         Del = 3,
        //     }
        // }
        // ```
        include!(concat!(env!("OUT_DIR"), "/fixture.rs"));
    }

    use super::{decode, encode};

    #[test]
    fn test_basic_encode_decode() {
        let msg = fixture::Msg {
            r#type: fixture::msg::Type::Put as _,
            id: 42,
            name: "the answer".to_owned(),
            paylad: vec![vec![7; 3]; 2],
        };
        let mut buf = vec![];
        encode(&msg, &mut buf).unwrap();
        let msg1 = decode(&buf).unwrap();
        assert_eq!(msg, msg1);
    }

    #[test]
    fn test_default() {
        let msg = fixture::Msg::default();
        let msg1 = decode(&[]).unwrap();
        assert_eq!(msg, msg1);
    }
}
