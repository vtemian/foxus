import { useEffect, useState } from "react";
import "@/static/css/globals.css";

import { FocusView } from "@/components/focus-view";
import { Header } from "@/components/header";
import { SettingsView } from "@/components/settings-view";
import { StatsView } from "@/components/stats-view";
import { Button, Typography } from "@/components/ui";
import { WeeklyStatsView } from "@/components/weekly-stats-view";
import { useTauri } from "@/hooks/use-tauri";
import type { FocusState, TauriStats, WeeklyStats } from "@/types/api";

const LoadingScreen = () => (
  <div className="min-h-screen bg-gray-100 noise-overlay">
    <div className="max-w-[400px] mx-auto p-4">
      <Typography variant="body" color="muted" className="text-center py-8">
        Loading...
      </Typography>
    </div>
  </div>
);

const ErrorScreen = () => (
  <div className="min-h-screen bg-gray-100 noise-overlay">
    <div className="max-w-[400px] mx-auto p-4">
      <Typography variant="body" color="distracting" className="text-center py-8">
        Failed to connect to Foxus backend
      </Typography>
    </div>
  </div>
);

const MainContent = ({
  stats,
  weeklyStats,
  focusState,
  period,
  toggleFocus,
}: {
  stats: TauriStats | null;
  weeklyStats: WeeklyStats | null;
  focusState: FocusState | null;
  period: "today" | "week";
  toggleFocus: () => Promise<void>;
}) => {
  const isFocusActive = focusState?.active ?? false;

  return (
    <>
      {!isFocusActive && period === "today" && <StatsView stats={stats} />}
      {!isFocusActive && period === "week" && <WeeklyStatsView stats={weeklyStats} />}
      {isFocusActive && <FocusView budgetRemaining={focusState?.budget_remaining ?? 0} />}

      <footer className="mt-4">
        <Button
          variant={isFocusActive ? "focus" : "default"}
          onClick={() => void toggleFocus()}
          aria-pressed={isFocusActive}
        >
          {isFocusActive ? "End Focus Session" : "Start Focus Session"}
        </Button>
      </footer>
    </>
  );
};

const App = () => {
  const { stats, weeklyStats, focusState, isLoading, error, toggleFocus, loadWeeklyStats } =
    useTauri();
  const [period, setPeriod] = useState<"today" | "week">("today");
  const [showSettings, setShowSettings] = useState(false);

  useEffect(() => {
    if (period === "week") {
      loadWeeklyStats();
    }
  }, [period, loadWeeklyStats]);

  if (isLoading) return <LoadingScreen />;
  if (error) return <ErrorScreen />;

  return (
    <div className="min-h-screen bg-gray-100 noise-overlay">
      <div className="max-w-[400px] mx-auto p-4">
        <Header
          period={period}
          onPeriodChange={setPeriod}
          onSettingsClick={() => setShowSettings(!showSettings)}
          showSettings={showSettings}
        />

        {showSettings ? (
          <SettingsView onClose={() => setShowSettings(false)} />
        ) : (
          <MainContent
            stats={stats}
            weeklyStats={weeklyStats}
            focusState={focusState}
            period={period}
            toggleFocus={toggleFocus}
          />
        )}
      </div>
    </div>
  );
};

export { App };
