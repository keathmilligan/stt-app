//! IPC server for client communication.

mod handlers;
mod server;

pub use server::{broadcast_event, run_server};
