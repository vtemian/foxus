import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useRef, useState } from "react";
import type { FocusState, TauriStats, WeeklyStats } from "@/types/api";

const REFRESH_INTERVAL_MS = 5000;
const DEFAULT_BUDGET_MINUTES = 10;

interface UseTauriReturn {
  stats: TauriStats | null;
  weeklyStats: WeeklyStats | null;
  focusState: FocusState | null;
  isLoading: boolean;
  error: Error | null;
  toggleFocus: () => Promise<void>;
  loadWeeklyStats: () => Promise<void>;
  refresh: () => Promise<void>;
}

const toError = (e: unknown): Error => (e instanceof Error ? e : new Error(String(e)));

const useLoadStats = (setError: (e: Error) => void) => {
  const [stats, setStats] = useState<TauriStats | null>(null);

  const loadStats = useCallback(async () => {
    try {
      setStats(await invoke<TauriStats>("get_today_stats"));
    } catch (e: unknown) {
      console.error("Failed to load stats:", e);
      setError(toError(e));
    }
  }, [setError]);

  return { stats, loadStats };
};

const useLoadWeeklyStats = (setError: (e: Error) => void) => {
  const [weeklyStats, setWeeklyStats] = useState<WeeklyStats | null>(null);

  const loadWeeklyStats = useCallback(async () => {
    try {
      setWeeklyStats(await invoke<WeeklyStats>("get_weekly_stats"));
    } catch (e: unknown) {
      console.error("Failed to load weekly stats:", e);
      setError(toError(e));
    }
  }, [setError]);

  return { weeklyStats, loadWeeklyStats };
};

const useLoadFocusState = (setError: (e: Error) => void) => {
  const [focusState, setFocusState] = useState<FocusState | null>(null);

  const loadFocusState = useCallback(async () => {
    try {
      setFocusState(await invoke<FocusState>("get_focus_state"));
    } catch (e: unknown) {
      console.error("Failed to load focus state:", e);
      setError(toError(e));
    }
  }, [setError]);

  return { focusState, loadFocusState };
};

const useRefresh = (loadStats: () => Promise<void>, loadFocusState: () => Promise<void>) => {
  const refreshInProgress = useRef(false);

  const refresh = useCallback(async () => {
    if (refreshInProgress.current) return;
    refreshInProgress.current = true;
    try {
      await Promise.all([loadStats(), loadFocusState()]);
    } finally {
      refreshInProgress.current = false;
    }
  }, [loadStats, loadFocusState]);

  return refresh;
};

const useInitAndPoll = (refresh: () => Promise<void>, setIsLoading: (v: boolean) => void) => {
  useEffect(() => {
    const initialize = async () => {
      await refresh();
      setIsLoading(false);
    };
    initialize();
  }, [refresh, setIsLoading]);

  useEffect(() => {
    const interval = setInterval(() => void refresh(), REFRESH_INTERVAL_MS);
    return () => clearInterval(interval);
  }, [refresh]);
};

const useTauri = (): UseTauriReturn => {
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const stableSetError = useCallback((e: Error) => setError(e), []);
  const { stats, loadStats } = useLoadStats(stableSetError);
  const { weeklyStats, loadWeeklyStats } = useLoadWeeklyStats(stableSetError);
  const { focusState, loadFocusState } = useLoadFocusState(stableSetError);
  const refresh = useRefresh(loadStats, loadFocusState);

  const toggleFocus = useCallback(async () => {
    if (!focusState) return;
    try {
      const cmd = focusState.active ? "end_focus_session" : "start_focus_session";
      const args = focusState.active ? undefined : { budgetMinutes: DEFAULT_BUDGET_MINUTES };
      await invoke(cmd, args);
      await refresh();
    } catch (e: unknown) {
      console.error("Failed to toggle focus:", e);
      setError(toError(e));
    }
  }, [focusState, refresh]);

  useInitAndPoll(refresh, setIsLoading);

  return {
    stats,
    weeklyStats,
    focusState,
    isLoading,
    error,
    toggleFocus,
    loadWeeklyStats,
    refresh,
  };
};

export type { UseTauriReturn };
export { useTauri };
