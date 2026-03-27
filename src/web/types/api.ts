/**
 * Productivity level for an app or activity.
 * -1 = distracting, 0 = neutral, 1 = productive
 */
type ProductivityLevel = -1 | 0 | 1;

/**
 * A category for grouping activities.
 */
interface Category {
  id: number;
  name: string;
  productivity: ProductivityLevel;
}

/**
 * Match type for rules.
 */
type MatchType = "app" | "domain" | "title";

/**
 * A rule for categorizing activities based on patterns.
 */
interface Rule {
  id: number;
  pattern: string;
  match_type: MatchType;
  category_id: number;
  priority: number;
}

/**
 * A single app activity with duration and productivity score.
 */
interface AppActivity {
  name: string;
  duration_secs: number;
  productivity: ProductivityLevel;
}

/**
 * Response from get_today_stats Tauri command.
 */
interface TauriStats {
  productive_secs: number;
  neutral_secs: number;
  distracting_secs: number;
  top_apps: AppActivity[];
}

/**
 * Response from get_focus_state Tauri command.
 */
interface FocusState {
  active: boolean;
  budget_remaining: number;
  session_duration_secs: number | null;
}

/**
 * Daily stats for a single day within weekly stats.
 */
interface DailyStats {
  date: number; // Unix timestamp (start of day)
  productive_secs: number;
  neutral_secs: number;
  distracting_secs: number;
}

/**
 * Response from get_weekly_stats Tauri command.
 */
interface WeeklyStats {
  daily_stats: DailyStats[];
  total_productive_secs: number;
  total_neutral_secs: number;
  total_distracting_secs: number;
  top_apps: AppActivity[];
}

/**
 * Productivity variant for styling components.
 */
type ProductivityVariant = "productive" | "neutral" | "distracting";

/**
 * Convert numeric productivity to variant string.
 */
const productivityToVariant = (p: ProductivityLevel): ProductivityVariant => {
  if (p > 0) return "productive";
  if (p < 0) return "distracting";
  return "neutral";
};

export type {
  AppActivity,
  Category,
  DailyStats,
  FocusState,
  MatchType,
  ProductivityLevel,
  ProductivityVariant,
  Rule,
  TauriStats,
  WeeklyStats,
};
export { productivityToVariant };
