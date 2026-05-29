import type { ReactNode } from "react";

export function EmptyState({ children, action }: { children: ReactNode; action?: ReactNode }) {
  return (
    <div className="empty-preview">
      <p>{children}</p>
      {action}
    </div>
  );
}
