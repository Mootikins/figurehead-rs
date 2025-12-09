//! Simple HTTP server for serving the web examples
//!
//! Run with: cargo run --manifest-path examples/web-editor/Cargo.toml
//! Or: just web-server

use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;

fn main() -> std::io::Result<()> {
    let port = std::env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr)?;

    println!("Figurehead web server running on http://localhost:{}", port);
    println!("Available pages:");
    println!("  http://localhost:{}/index.html - Interactive editor", port);
    println!("  http://localhost:{}/editor.html - Live editor", port);
    println!("Press Ctrl+C to stop");

    for stream in listener.incoming() {
        let mut stream = stream?;
        let mut buffer = [0; 4096];
        let bytes_read = stream.read(&mut buffer)?;

        if bytes_read == 0 {
            continue;
        }

        let request = String::from_utf8_lossy(&buffer[..bytes_read]);
        let path = request
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .unwrap_or("/");

        let (status, content_type, body) = match path {
            "/" => serve_file("editor.html", "text/html"),
            "/index.html" => serve_file("index.html", "text/html"),
            "/editor.html" => serve_file("editor.html", "text/html"),
            path if path.starts_with("/pkg/") => {
                let file_path = path.trim_start_matches('/');
                let content_type = if path.ends_with(".wasm") {
                    "application/wasm"
                } else if path.ends_with(".js") {
                    "application/javascript"
                } else if path.ends_with(".ts") {
                    "application/typescript"
                } else {
                    "application/octet-stream"
                };
                serve_file(file_path, content_type)
            }
            _ => (
                "404 Not Found".to_string(),
                "text/plain".to_string(),
                format!("404 Not Found: {}", path).into_bytes(),
            ),
        };

        write_response(&mut stream, &status, &content_type, &body)?;
        stream.flush()?;
    }

    Ok(())
}

fn serve_file(path: &str, content_type: &str) -> (String, String, Vec<u8>) {
    let base_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let file_path = base_path.join(path);

    if content_type == "application/wasm" || content_type == "application/octet-stream" {
        // Binary file
        match fs::read(&file_path) {
            Ok(bytes) => ("200 OK".to_string(), content_type.to_string(), bytes),
            Err(_) => (
                "404 Not Found".to_string(),
                "text/plain".to_string(),
                format!("File not found: {}", path).into_bytes(),
            ),
        }
    } else {
        // Text file
        match fs::read_to_string(&file_path) {
            Ok(content) => ("200 OK".to_string(), content_type.to_string(), content.into_bytes()),
            Err(_) => (
                "404 Not Found".to_string(),
                "text/plain".to_string(),
                format!("File not found: {}", path).into_bytes(),
            ),
        }
    }
}

fn write_response(stream: &mut std::net::TcpStream, status: &str, content_type: &str, body: &[u8]) -> std::io::Result<()> {
    let header = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
        status,
        content_type,
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(body)?;
    Ok(())
}
