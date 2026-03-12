//! gRPC streamer library – re-exports the generated client and the
//! reconnect-ready [`Streamer`] wrapper.

pub mod proto {
    tonic::include_proto!("yellowstone");
}

pub mod streamer;

pub use streamer::Streamer;
