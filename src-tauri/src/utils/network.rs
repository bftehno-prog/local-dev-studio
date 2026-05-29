use std::{
    net::{IpAddr, TcpListener},
    process::Command,
};

pub(crate) fn network_url(port: u16) -> String {
    format!("http://{}:{}", local_ip_address(), port)
}

pub(crate) fn local_ip_address() -> String {
    if cfg!(windows) {
        local_ip_from_ipconfig().unwrap_or_else(|| "127.0.0.1".to_string())
    } else {
        "127.0.0.1".to_string()
    }
}

fn local_ip_from_ipconfig() -> Option<String> {
    let output = Command::new("ipconfig").output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if !line.contains("IPv4") {
            continue;
        }
        let candidate = line.split(':').nth(1)?.trim();
        if let Ok(IpAddr::V4(ip)) = candidate.parse::<IpAddr>() {
            if !ip.is_loopback() && !ip.is_link_local() && !ip.is_unspecified() {
                return Some(ip.to_string());
            }
        }
    }
    None
}

pub(crate) fn is_port_free(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}

pub(crate) fn find_free_port(start: u16, end: u16) -> Result<u16, String> {
    for port in start..=end {
        if is_port_free(port) {
            return Ok(port);
        }
    }
    Err(format!("No free port found in range {}-{}.", start, end))
}
