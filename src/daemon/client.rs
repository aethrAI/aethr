use crate::Result;
use crate::utils::config;
use std::os::unix::net::UnixStream;
use std::io::{Read, Write};

pub fn send_request(payload: &str) -> Result<String> {
    let socket = config::get_daemon_socket_path();
    let mut stream = UnixStream::connect(&socket)?;
    stream.write_all(payload.as_bytes())?;
    stream.shutdown(std::net::Shutdown::Write)?;
    let mut buf = String::new();
    stream.read_to_string(&mut buf)?;
    Ok(buf)
}
