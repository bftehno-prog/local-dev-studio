import type { ProjectStatus } from "../../lib/types";

export function StatusBadge({ status }: { status: ProjectStatus | "unknown" | string }) {
  return <span className={`status ${status}`}>{status}</span>;
}
