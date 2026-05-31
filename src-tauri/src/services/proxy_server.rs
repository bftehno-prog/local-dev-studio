use std::{
    io::{Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use crate::state::ManagedProxy;

const HEADER_LIMIT: usize = 64 * 1024;

pub(crate) fn preview_url(project_id: &str, proxy_port: u16) -> String {
    format!("http://127.0.0.1:{}/preview/{}/", proxy_port, project_id)
}

pub(crate) fn spawn_proxy(
    project_id: String,
    proxy_port: u16,
    target_port: u16,
) -> Result<ManagedProxy, String> {
    let listener = TcpListener::bind(("127.0.0.1", proxy_port))
        .map_err(|error| format!("Could not start proxy on port {}: {}", proxy_port, error))?;
    let bound_port = listener
        .local_addr()
        .map_err(|error| error.to_string())?
        .port();
    listener
        .set_nonblocking(true)
        .map_err(|error| error.to_string())?;
    let stop = Arc::new(AtomicBool::new(false));
    let stop_thread = Arc::clone(&stop);
    let handle = thread::spawn(move || {
        while !stop_thread.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((client, _)) => {
                    let project_id = project_id.clone();
                    thread::spawn(move || {
                        let _ = handle_client(client, &project_id, target_port);
                    });
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(50));
                }
                Err(_) => break,
            }
        }
    });
    Ok(ManagedProxy {
        port: bound_port,
        target_port,
        stop,
        handle: Some(handle),
    })
}

fn handle_client(mut client: TcpStream, project_id: &str, target_port: u16) -> Result<(), String> {
    let mut header = Vec::new();
    let mut buffer = [0_u8; 2048];
    loop {
        let read = client
            .read(&mut buffer)
            .map_err(|error| error.to_string())?;
        if read == 0 {
            return Ok(());
        }
        header.extend_from_slice(&buffer[..read]);
        if header.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
        if header.len() > HEADER_LIMIT {
            return write_error(client, 431, "Request Header Fields Too Large");
        }
    }
    let request = rewrite_request(&header, project_id, target_port)?;
    let mut upstream =
        TcpStream::connect(("127.0.0.1", target_port)).map_err(|error| error.to_string())?;
    upstream
        .write_all(&request)
        .map_err(|error| error.to_string())?;
    relay_bidirectional(client, upstream);
    Ok(())
}

fn rewrite_request(raw: &[u8], project_id: &str, target_port: u16) -> Result<Vec<u8>, String> {
    let text = String::from_utf8_lossy(raw);
    let Some((request_line, rest)) = text.split_once("\r\n") else {
        return Err("Invalid HTTP request.".to_string());
    };
    let parts = request_line.split_whitespace().collect::<Vec<_>>();
    if parts.len() != 3 {
        return Err("Invalid HTTP request line.".to_string());
    }
    let prefix = format!("/preview/{}", project_id);
    let path = parts[1];
    if path != prefix && !path.starts_with(&(prefix.clone() + "/")) {
        return Err("Proxy path does not match this project.".to_string());
    }
    let stripped = path
        .strip_prefix(&prefix)
        .filter(|value| !value.is_empty())
        .unwrap_or("/");
    let rewritten_path = if stripped.starts_with('/') {
        stripped.to_string()
    } else {
        format!("/{}", stripped)
    };
    let mut rewritten = format!("{} {} {}\r\n", parts[0], rewritten_path, parts[2]);
    for line in rest.split("\r\n") {
        if line.is_empty() {
            rewritten.push_str("\r\n");
            break;
        }
        if line.to_ascii_lowercase().starts_with("host:") {
            rewritten.push_str(&format!("Host: 127.0.0.1:{}\r\n", target_port));
        } else {
            rewritten.push_str(line);
            rewritten.push_str("\r\n");
        }
    }
    Ok(rewritten.into_bytes())
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write},
        net::{Shutdown, TcpListener, TcpStream},
        thread,
    };

    use super::{rewrite_request, spawn_proxy};

    #[test]
    fn rewrites_preview_prefix_to_upstream_root() {
        let request = b"GET /preview/project_1/assets/app.js?x=1 HTTP/1.1\r\nHost: 127.0.0.1:4100\r\nConnection: keep-alive\r\n\r\n";
        let rewritten = String::from_utf8(rewrite_request(request, "project_1", 3105).unwrap())
            .expect("rewritten request should be utf8");

        assert!(rewritten.starts_with("GET /assets/app.js?x=1 HTTP/1.1\r\n"));
        assert!(rewritten.contains("Host: 127.0.0.1:3105\r\n"));
    }

    #[test]
    fn rejects_request_for_other_project() {
        let request = b"GET /preview/project_2/ HTTP/1.1\r\nHost: 127.0.0.1:4100\r\n\r\n";

        assert!(rewrite_request(request, "project_1", 3105).is_err());
    }

    #[test]
    fn forwards_preview_request_to_upstream_server() {
        let upstream = TcpListener::bind(("127.0.0.1", 0)).expect("upstream should bind");
        let upstream_port = upstream
            .local_addr()
            .expect("upstream should have address")
            .port();
        let upstream_thread = thread::spawn(move || {
            let (mut connection, _) = upstream.accept().expect("upstream should accept");
            let mut request = [0_u8; 512];
            let read = connection
                .read(&mut request)
                .expect("upstream should read request");
            let request = String::from_utf8_lossy(&request[..read]);
            assert!(request.starts_with("GET /index.html HTTP/1.1\r\n"));
            connection
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nOK")
                .expect("upstream should write response");
        });
        let mut proxy =
            spawn_proxy("project_1".to_string(), 0, upstream_port).expect("proxy should start");
        let mut client =
            TcpStream::connect(("127.0.0.1", proxy.port)).expect("client should connect to proxy");
        client
            .write_all(
                b"GET /preview/project_1/index.html HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
            )
            .expect("client should write request");
        client
            .shutdown(Shutdown::Write)
            .expect("client should finish writing");
        let mut response = String::new();
        client
            .read_to_string(&mut response)
            .expect("client should read response");

        proxy.stop();
        upstream_thread.join().expect("upstream should finish");
        assert!(response.contains("HTTP/1.1 200 OK"));
        assert!(response.ends_with("OK"));
    }
}

fn relay_bidirectional(client: TcpStream, upstream: TcpStream) {
    let Ok(mut client_reader) = client.try_clone() else {
        return;
    };
    let Ok(mut upstream_reader) = upstream.try_clone() else {
        return;
    };
    let mut client_writer = client;
    let mut upstream_writer = upstream;
    let upstream_to_client = thread::spawn(move || {
        let _ = std::io::copy(&mut upstream_reader, &mut client_writer);
        let _ = client_writer.shutdown(Shutdown::Both);
    });
    let client_to_upstream = thread::spawn(move || {
        let _ = std::io::copy(&mut client_reader, &mut upstream_writer);
        let _ = upstream_writer.shutdown(Shutdown::Write);
    });
    let _ = upstream_to_client.join();
    let _ = client_to_upstream.join();
}

fn write_error(mut client: TcpStream, status: u16, message: &str) -> Result<(), String> {
    let body = format!("{}\n", message);
    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status,
        message,
        body.len(),
        body
    );
    client
        .write_all(response.as_bytes())
        .map_err(|error| error.to_string())
}
