//! Control LAN optativo. Nunca sirve archivos del usuario ni acepta rutas de audio.
mod assets;
mod http;
mod mirror;
pub mod protocol;
mod server;
pub use server::{RemoteServer, RemoteStatus};
