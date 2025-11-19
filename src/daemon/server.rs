use crate::Result;
use crate::utils::config;
use std::os::unix::net::UnixListener;
use std::io::{Read, Write};
use std::thread;
use std::fs;

/// Simple blocking daemon runner (foreground) for development.
/// In production run as systemd user service and use the async version.

pub fn run_blocking() -> Result<()> {
    let socket = config::get_daemon_socket_path();
    if socket.exists() {
        let _ = fs::remove_file(&socket);
    }
    let listener = UnixListener::bind(&socket)?;
    println!("Daemon listening on {}", socket.display());

    for stream in listener.incoming() {
        match stream {
            Ok(mut s) => {
                let mut reader = s.try_clone()?;
                thread::spawn(move || {
                    let mut data = Vec::new();
                    if let Ok(_) = reader.read_to_end(&mut data) {
                        // naive: expect a UTF-8 JSON string; in future, use length-prefixed framing or tokio async.
                        if let Ok(req) = String::from_utf8(data) {
                            // For now, echo back to client
                            let resp = format!("Daemon received: {}", req);
                            let _ = s.write_all(resp.as_bytes());
                        }
                    }
                });
            }
            Err(e) => {
                eprintln!("Daemon accept error: {}", e);
            }
        }
    }

    Ok(())
}
