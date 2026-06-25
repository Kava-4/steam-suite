const styles = {
  ok: "bg-[var(--bg-interactive)] text-[var(--text-body)]",
  warn: "bg-[var(--bg-interactive)] text-[#fbbf24]",
  error: "bg-[var(--bg-interactive)] text-[#f87171]",
  idle: "bg-[var(--bg-inset)] text-[var(--text-muted)]",
  accent:
    "border border-[var(--accent-border)] bg-[var(--bg-inset)] text-[var(--accent)]",
};

type Status = keyof typeof styles;

export function StatusBadge({
  status,
  label,
}: {
  status: Status;
  label: string;
}) {
  return (
    <span
      className={`inline-flex items-center rounded-[var(--radius-sm)] px-2 py-0.5 text-[10px] font-semibold uppercase tracking-[0.05em] ${styles[status]}`}
    >
      {label}
    </span>
  );
}
