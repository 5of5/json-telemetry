//! Hosted shell: `work --serve ADDR`. One static binary hosts the whole
//! catalog over HTTP/1.1 with the standard library only (minimal-dependency
//! doctrine; wasm32 untouched since this module is `cfg(feature = "cli")`).
//!
//! Routes (all JSON, `Connection: close`):
//!   GET  /health    → {"ok":true,"catalog":560,"version":…}
//!   GET  /commands  → aria-work-commands-v1 (what Aria compiles against)
//!   GET  /dispatch  → aria-dispatch-v1 (PCVC registry descriptor)
//!   POST /work      → body: work-v1 command {work|ops, in}  → callback
//!   POST /harness   → body: pcvc-aria-telemetry-request-v1  → bound result
//!
//! The node is stateless, so each connection runs on its own thread with no
//! shared mutable state; sequential and parallel bytes are identical
//! (`dump` proves it every run).

use crate::{commands_json, dispatch_json, execute_work, harness_lane, RunOpts, WorkRequest};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

/// Largest request body accepted (the same order as PCVC's evidence caps).
const MAX_BODY: usize = 8 * 1024 * 1024;

/// Bind and serve forever. Returns only on bind failure.
pub fn serve(addr: &str, opts: &RunOpts) -> io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    // Startup line to stdout (not stderr — harness lanes keep stderr silent).
    println!(
        "{}",
        serde_json::json!({"serving": listener.local_addr()?.to_string(), "catalog": crate::catalog().len()})
    );
    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        let opts = opts.clone();
        std::thread::spawn(move || {
            let _ = handle(stream, &opts);
        });
    }
    Ok(())
}

fn handle(mut stream: TcpStream, opts: &RunOpts) -> io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let mut parts = line.split_whitespace();
    let (method, path) = (parts.next().unwrap_or(""), parts.next().unwrap_or("/"));
    let mut len = 0usize;
    loop {
        let mut h = String::new();
        if reader.read_line(&mut h)? == 0 || h == "\r\n" || h == "\n" {
            break;
        }
        if let Some(v) = h.to_ascii_lowercase().strip_prefix("content-length:") {
            len = v.trim().parse().unwrap_or(0);
        }
    }
    if len > MAX_BODY {
        return respond(&mut stream, 413, br#"{"error":"body too large"}"#);
    }
    let mut body = vec![0u8; len];
    if len > 0 {
        reader.read_exact(&mut body)?;
    }
    let (code, out): (u16, Vec<u8>) = match (method, path.split('?').next().unwrap_or(path)) {
        ("GET", "/health") => (
            200,
            serde_json::to_vec(&serde_json::json!({
                "ok": true,
                "catalog": crate::catalog().len(),
                "version": env!("CARGO_PKG_VERSION"),
                "stateless": true,
            }))
            .unwrap_or_default(),
        ),
        ("GET", "/commands") => (200, serde_json::to_vec(&commands_json()).unwrap_or_default()),
        ("GET", "/dispatch") => (200, serde_json::to_vec(&dispatch_json()).unwrap_or_default()),
        ("POST", "/work") => match serde_json::from_slice::<WorkRequest>(&body) {
            Ok(req) => match execute_work(&req, opts) {
                Ok(v) => (200, serde_json::to_vec(&v).unwrap_or_default()),
                Err(e) => (422, err_json(&e.to_string())),
            },
            Err(e) => (400, err_json(&e.to_string())),
        },
        ("POST", "/harness") => {
            let (exit, bytes) = harness_lane(&body);
            (if exit == 0 { 200 } else { 400 }, bytes)
        }
        _ => (404, err_json("unknown route")),
    };
    respond(&mut stream, code, &out)
}

fn err_json(msg: &str) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({"error": msg})).unwrap_or_default()
}

fn respond(stream: &mut TcpStream, code: u16, body: &[u8]) -> io::Result<()> {
    let reason = match code {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        413 => "Payload Too Large",
        422 => "Unprocessable Entity",
        _ => "Error",
    };
    write!(
        stream,
        "HTTP/1.1 {code} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    stream.write_all(body)?;
    stream.flush()
}
