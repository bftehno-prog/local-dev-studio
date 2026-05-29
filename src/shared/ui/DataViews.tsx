import type { LogEntry, PortInfo, ServerProcess, TemplateInfo } from "../../lib/types";
import type { TranslationKey } from "../../lib/i18n";
import type { TFunction } from "../../app/types";

export function Metric({ label, value }: { label: string; value: string | number }) {
  return <div className="metric"><span>{label}</span><strong>{value}</strong></div>;
}

export function Info({ label, value }: { label: string; value: string }) {
  return <div className="info-row"><span>{label}</span><code>{value}</code></div>;
}

export function Status({ status, t }: { status: string; t: TFunction }) {
  return <span className={`status ${status}`}>{t(`status.${status}` as TranslationKey)}</span>;
}

export function ServerTable({ servers, compact = false, t }: { servers: ServerProcess[]; compact?: boolean; t: TFunction }) {
  if (!servers.length) return <p className="muted">{t("empty.noServers")}</p>;
  return (
    <table>
      <thead><tr><th>{t("table.project")}</th><th>{t("table.type")}</th><th>{t("table.pid")}</th><th>{t("table.port")}</th>{!compact && <th>{t("table.url")}</th>}<th>{t("table.status")}</th>{!compact && <th>{t("table.memory")}</th>}</tr></thead>
      <tbody>
        {servers.map((server) => (
          <tr key={`${server.project_id}-${server.pid}`}>
            <td>{server.project_name}</td>
            <td>{server.project_type}</td>
            <td>{server.pid}</td>
            <td>{server.port}</td>
            {!compact && <td><code>{server.url}</code></td>}
            <td><Status status={server.status} t={t} /></td>
            {!compact && <td>{server.memory_usage_mb?.toFixed(1) || "-"} MB</td>}
          </tr>
        ))}
      </tbody>
    </table>
  );
}

export function PortList({ ports, t }: { ports: PortInfo[]; t: TFunction }) {
  if (!ports.length) return <p className="muted">{t("empty.noPorts")}</p>;
  return <div className="port-list">{ports.map((port) => <span key={port.port}>{port.port}{port.external ? ` ${t("ports.external")}` : ""}</span>)}</div>;
}

export function LogList({ logs, t }: { logs: LogEntry[]; t: TFunction }) {
  if (!logs.length) return <p className="muted">{t("empty.noLogs")}</p>;
  return <div className="logs">{logs.map((log) => <div className={`log ${log.level}`} key={log.id}><span>{log.created_at}</span><strong>{log.level}</strong><p>{log.message}</p></div>)}</div>;
}

export function templateName(template: TemplateInfo, t: TFunction) {
  if (!template.built_in) {
    return template.name;
  }
  return t(`template.${template.id}` as TranslationKey);
}

export function versionText(value: string | undefined, t: TFunction) {
  if (!value || value === "Not found") {
    return t("empty.notFound");
  }
  return value;
}

export function runtimeText(value: string | undefined, t: TFunction) {
  if (!value) {
    return t("empty.checking");
  }
  if (value === "Ready") {
    return "OK";
  }
  if (value === "Node.js not found") {
    return t("empty.notFound");
  }
  return value;
}
