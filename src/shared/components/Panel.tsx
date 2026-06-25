import { Card } from "@heroui/react";
import type { ReactNode } from "react";

export function Panel({
  title,
  description,
  children,
  action,
  className = "",
}: {
  title: string;
  description?: string;
  children: ReactNode;
  action?: ReactNode;
  className?: string;
}) {
  return (
    <Card
      className={`glass-panel border-0 bg-transparent p-0 shadow-none ${className}`}
    >
      <Card.Header className="flex items-start justify-between gap-4 px-5 pt-5 pb-0">
        <div>
          <Card.Title className="text-base font-semibold text-white">
            {title}
          </Card.Title>
          {description && (
            <Card.Description className="mt-1 text-xs text-[var(--text-muted)]">
              {description}
            </Card.Description>
          )}
        </div>
        {action}
      </Card.Header>
      <Card.Content className="px-5 pb-5 pt-4">{children}</Card.Content>
    </Card>
  );
}
