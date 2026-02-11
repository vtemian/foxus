import { invoke } from "@tauri-apps/api/core";
import { useState, useEffect, useCallback, useRef } from "react";
import type { TauriStats, FocusState } from "@/types/api";

const REFRESH_INTERVAL = 5000; // 5 seconds

export type UseTauriReturn = {
  stats: TauriStats | null;
  focusState: FocusState | null;
  isLoading: boolean;
  error: Error | null;
  toggleFocus: () => Promise<void>;
  refresh: () => Promise<void>;
};

export const useTauri = (): UseTauriReturn => {
  const [stats, setStats] = useState<TauriStats | null>(null);
  const [focusState, setFocusState] = useState<FocusState | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);
  const refreshInProgress = useRef(false);

  const loadStats = useCallback(async () => {
    try {
      const data = await invoke<TauriStats>("get_today_stats");
      setStats(data);
      setError(null);
    } catch (e) {
      console.error("Failed to load stats:", e);
      setError(e instanceof Error ? e : new Error(String(e)));
    }
  }, []);

  const loadFocusState = useCallback(async () => {
    try {
      const state = await invoke<FocusState>("get_focus_state");
      setFocusState(state);
      setError(null);
    } catch (e) {
      console.error("Failed to load focus state:", e);
      setError(e instanceof Error ? e : new Error(String(e)));
    }
  }, []);

  const refresh = useCallback(async () => {
    if (refreshInProgress.current) return;
    refreshInProgress.current = true;
    try {
      await Promise.all([loadStats(), loadFocusState()]);
    } finally {
      refreshInProgress.current = false;
    }
  }, [loadStats, loadFocusState]);

  const toggleFocus = useCallback(async () => {
    if (!focusState) return;

    try {
      if (focusState.active) {
        await invoke("end_focus_session");
      } else {
        await invoke("start_focus_session", { budgetMinutes: 10 });
      }
      await refresh();
    } catch (e) {
      console.error("Failed to toggle focus:", e);
      setError(e instanceof Error ? e : new Error(String(e)));
    }
  }, [focusState, refresh]);

  // Initial load
  useEffect(() => {
    const initialize = async () => {
      await refresh();
      setIsLoading(false);
    };
    initialize();
  }, [refresh]);

  // Periodic refresh
  useEffect(() => {
    const interval = setInterval(refresh, REFRESH_INTERVAL);
    return () => clearInterval(interval);
  }, [refresh]);

  return { stats, focusState, isLoading, error, toggleFocus, refresh };
};
