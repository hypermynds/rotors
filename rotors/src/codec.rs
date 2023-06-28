use std::marker::PhantomData;

use bytes::{Buf, BufMut};

/// A [`Codec`] that implements `application/grpc+cbor`
#[derive(Debug)]
pub struct CborCodec<T, U>(PhantomData<(T, U)>);

impl<T, U> Default for CborCodec<T, U> {
    #[inline]
    fn default() -> Self {
        CborCodec(PhantomData)
    }
}

impl<T, U> Clone for CborCodec<T, U> {
    #[inline]
    fn clone(&self) -> Self {
        CborCodec(PhantomData)
    }
}

impl<T, U> Copy for CborCodec<T, U> {}

impl<T, U> tonic::codec::Codec for CborCodec<T, U>
where
    T: Send + 'static + serde::Serialize,
    U: Send + 'static + for<'de> serde::Deserialize<'de>,
{
    type Encode = T;
    type Decode = U;

    type Encoder = CborEncoder<T>;
    type Decoder = CborDecoder<U>;

    fn encoder(&mut self) -> Self::Encoder {
        CborEncoder(PhantomData)
    }

    fn decoder(&mut self) -> Self::Decoder {
        CborDecoder(PhantomData)
    }
}

#[derive(Debug)]
pub struct CborEncoder<T>(PhantomData<T>);

impl<T> Clone for CborEncoder<T> {
    #[inline]
    fn clone(&self) -> Self {
        CborEncoder(PhantomData)
    }
}

impl<T> Copy for CborEncoder<T> {}

impl<T> tonic::codec::Encoder for CborEncoder<T>
where
    T: serde::Serialize,
{
    type Item = T;
    type Error = tonic::Status;

    fn encode(
        &mut self,
        item: Self::Item,
        dst: &mut tonic::codec::EncodeBuf<'_>,
    ) -> Result<(), Self::Error> {
        ciborium::into_writer(&item, dst.writer()).map_err(|err| match err {
            ciborium::ser::Error::Io(err) => tonic::Status::from(err),
            err => tonic::Status::new(tonic::Code::Internal, err.to_string()),
        })
    }
}

#[derive(Debug)]
pub struct CborDecoder<T>(PhantomData<T>);

impl<T> Clone for CborDecoder<T> {
    #[inline]
    fn clone(&self) -> Self {
        CborDecoder(PhantomData)
    }
}

impl<T> Copy for CborDecoder<T> {}

impl<T> tonic::codec::Decoder for CborDecoder<T>
where
    T: for<'de> serde::Deserialize<'de>,
{
    type Item = T;
    type Error = tonic::Status;

    fn decode(
        &mut self,
        src: &mut tonic::codec::DecodeBuf<'_>,
    ) -> Result<Option<Self::Item>, Self::Error> {
        ciborium::from_reader(src.reader()).map_err(|err| match err {
            ciborium::de::Error::Io(err) => tonic::Status::from(err),
            err => tonic::Status::new(tonic::Code::Internal, err.to_string()),
        })
    }
}
