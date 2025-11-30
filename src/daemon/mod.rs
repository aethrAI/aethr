// Daemon uses Unix sockets - only available on Unix systems
#[cfg(unix)]
pub mod server;
#[cfg(unix)]
pub mod client;