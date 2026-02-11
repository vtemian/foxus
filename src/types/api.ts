/**
 * Productivity level for an app or activity.
 * -1 = distracting, 0 = neutral, 1 = productive
 */
export type ProductivityLevel = -1 | 0 | 1;

/**
 * A single app activity with duration and productivity score.
 */
export type AppActivity = {
  name: string;
  duration_secs: number;
  productivity: ProductivityLevel;
};

/**
 * Response from get_today_stats Tauri command.
 */
export type TauriStats = {
  productive_secs: number;
  neutral_secs: number;
  distracting_secs: number;
  top_apps: AppActivity[];
};

/**
 * Response from get_focus_state Tauri command.
 */
export type FocusState = {
  active: boolean;
  budget_remaining: number;
};

/**
 * Productivity variant for styling components.
 */
export type ProductivityVariant = "productive" | "neutral" | "distracting";

/**
 * Convert numeric productivity to variant string.
 */
export const productivityToVariant = (p: ProductivityLevel): ProductivityVariant => {
  if (p > 0) return "productive";
  if (p < 0) return "distracting";
  return "neutral";
};
