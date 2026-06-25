import { useEffect, useState } from "react";
import { api } from "@/shared/api/tauri";
import { Button } from "@/shared/components/Button";
import { Panel } from "@/shared/components/Panel";
import { StatusBadge } from "@/shared/components/StatusBadge";
import { useSettingsStore } from "@/shared/stores/settingsStore";
import { SCHEDULER_TASKS, type SchedulerStatus } from "@/shared/types";

export function SchedulerPage() {
  const settings = useSettingsStore((s) => s.settings);
  const [status, setStatus] = useState<SchedulerStatus | null>(null);

  const refresh = async () => {
    try {
      setStatus(await api.schedulerGetStatus());
    } catch {
      // ignore
    }
  };

  useEffect(() => {
    void refresh();
    const timer = setInterval(() => {
      if (status?.running) void refresh();
    }, 2000);
    return () => clearInterval(timer);
  }, [status?.running]);

  const start = async () => {
    await api.schedulerStart();
    await refresh();
  };

  const stop = async () => {
    await api.schedulerStop();
    await refresh();
  };

  const advance = async () => {
    await api.schedulerAdvance();
    await refresh();
  };

  const tasks = settings?.schedulerTasks ?? SCHEDULER_TASKS.map((t) => t.id);

  return (
    <div className="space-y-6">
      <Panel title="Task Scheduler">
        <p className="mb-6 text-sm leading-relaxed text-[#9aa3b5]">
          Chain automation tasks: card farm → idle → giveaways → redeem on win.
          Configure the task order in Settings.
        </p>

        <div className="mb-4 flex items-center gap-3">
          <StatusBadge
            status={status?.running ? "ok" : "idle"}
            label={status?.running ? "Running" : "Idle"}
          />
          {status?.currentTask && (
            <span className="text-xs text-[#93c5fd]">
              Current: {status.currentTask}
            </span>
          )}
          {status?.lastError && (
            <span className="text-xs text-red-400">{status.lastError}</span>
          )}
        </div>

        <div className="flex flex-wrap gap-2">
          <Button variant="primary" onClick={() => void start()}>
            Start chain
          </Button>
          <Button variant="danger" onClick={() => void stop()}>
            Stop
          </Button>
          <Button onClick={() => void advance()}>Advance task</Button>
        </div>
      </Panel>

      <Panel title="Task chain">
        <ol className="space-y-2">
          {tasks.map((taskId, index) => {
            const task = SCHEDULER_TASKS.find((t) => t.id === taskId);
            const done = status?.completedTasks.includes(taskId);
            const current = status?.currentTask === taskId;
            return (
              <li
                key={taskId}
                className={`flex items-center gap-3 rounded-lg border px-4 py-3 text-sm ${
                  current
                    ? "border-[#2d6b4f] bg-[#1a2e28] text-[#6ee7a8]"
                    : done
                      ? "border-[#2a3140] bg-[#12151c] text-[#8b95a8] line-through"
                      : "border-[#2a3140] bg-[#12151c] text-[#c5cdd9]"
                }`}
              >
                <span className="text-xs text-[#5c6578]">{index + 1}.</span>
                {task?.label ?? taskId}
                {current && (
                  <StatusBadge status="ok" label="active" />
                )}
              </li>
            );
          })}
        </ol>
      </Panel>
    </div>
  );
}
