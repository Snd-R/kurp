use std::io::Result;
use std::ops::Deref;

use async_compression::Level;
use async_compression::tokio::write::{BrotliDecoder, BrotliEncoder, DeflateDecoder, DeflateEncoder, GzipDecoder, GzipEncoder};
use bytes::Bytes;
use Level::Fastest;
use tokio::io::AsyncWriteExt;

pub async fn decompress(bytes: Bytes, algorithm: Algorithm) -> Result<Bytes> {
    match algorithm {
        Algorithm::Brotli => {
            let mut decoder = BrotliDecoder::new(Vec::new());
            decoder.write_all(bytes.deref()).await?;
            decoder.shutdown().await?;
            Ok(Bytes::from(decoder.into_inner()))
        }
        Algorithm::Gzip => {
            let mut decoder = GzipDecoder::new(Vec::new());
            decoder.write_all(bytes.deref()).await?;
            decoder.shutdown().await?;
            Ok(Bytes::from(decoder.into_inner()))
        }
        Algorithm::Deflate => {
            let mut decoder = DeflateDecoder::new(Vec::new());
            decoder.write_all(bytes.deref()).await?;
            decoder.shutdown().await?;
            Ok(Bytes::from(decoder.into_inner()))
        }
    }
}

pub async fn compress(bytes: Bytes, algorithm: Algorithm) -> Result<Bytes> {
    match algorithm {
        Algorithm::Brotli => {
            let mut encoder = BrotliEncoder::with_quality(Vec::new(), Fastest);
            encoder.write_all(bytes.deref()).await?;
            encoder.shutdown().await?;
            Ok(Bytes::from(encoder.into_inner()))
        }
        Algorithm::Gzip => {
            let mut encoder = GzipEncoder::with_quality(Vec::new(), Fastest);
            encoder.write_all(bytes.deref()).await?;
            encoder.shutdown().await?;
            Ok(Bytes::from(encoder.into_inner()))
        }
        Algorithm::Deflate => {
            let mut encoder = DeflateEncoder::with_quality(Vec::new(), Fastest);
            encoder.write_all(bytes.deref()).await?;
            encoder.shutdown().await?;
            Ok(Bytes::from(encoder.into_inner()))
        }
    }
}

pub enum Algorithm {
    Brotli,
    Gzip,
    Deflate,
}