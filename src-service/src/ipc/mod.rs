//! IPC server for client communication.

pub mod handlers;
mod server;

pub use server::{broadcast_event, run_server};
