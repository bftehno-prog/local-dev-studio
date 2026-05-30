use crate::{
    models::{PortInfo, ServerProcess, Settings},
    utils::network::{is_port_free, network_url as build_network_url},
};

pub(crate) fn network_url(port: u16) -> String {
    build_network_url(port)
}

pub(crate) fn list_ports(settings: &Settings, servers: &[ServerProcess]) -> Vec<PortInfo> {
    let mut result = Vec::new();
    for port in settings.port_start..=settings.port_end.min(settings.port_start + 120) {
        if let Some(server) = servers.iter().find(|server| server.port == port) {
            result.push(PortInfo {
                port,
                available: false,
                pid: Some(server.pid),
                project_id: Some(server.project_id.clone()),
                project_name: Some(server.project_name.clone()),
                external: false,
            });
        } else {
            let available = is_port_free(port);
            result.push(PortInfo {
                port,
                available,
                pid: None,
                project_id: None,
                project_name: None,
                external: !available,
            });
        }
    }
    result
}
