import { useState } from "react";
import "@/static/css/globals.css";

import { useTauri } from "@/hooks/use-tauri";
import { Button, Typography } from "@/components/ui";
import { Header } from "@/components/header";
import { StatsView } from "@/components/stats-view";
import { FocusView } from "@/components/focus-view";

export default function App() {
  const { stats, focusState, isLoading, error, toggleFocus } = useTauri();
  const [period, setPeriod] = useState<"today" | "week">("today");

  const isFocusActive = focusState?.active ?? false;

  if (isLoading) {
    return (
      <div className="min-h-screen bg-gray-100 noise-overlay">
        <div className="max-w-[400px] mx-auto p-4">
          <Typography variant="body" color="muted" className="text-center py-8">
            Loading...
          </Typography>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="min-h-screen bg-gray-100 noise-overlay">
        <div className="max-w-[400px] mx-auto p-4">
          <Typography variant="body" color="distracting" className="text-center py-8">
            Failed to connect to Foxus backend
          </Typography>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-100 noise-overlay">
      <div className="max-w-[400px] mx-auto p-4">
        <Header period={period} onPeriodChange={setPeriod} />

        {!isFocusActive && <StatsView stats={stats} />}
        {isFocusActive && (
          <FocusView budgetRemaining={focusState?.budget_remaining ?? 0} />
        )}

        <footer className="mt-4">
          <Button
            variant={isFocusActive ? "focus" : "default"}
            onClick={toggleFocus}
            aria-pressed={isFocusActive}
          >
            {isFocusActive ? "End Focus Session" : "Start Focus Session"}
          </Button>
        </footer>
      </div>
    </div>
  );
}
