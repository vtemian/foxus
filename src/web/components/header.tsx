import { Typography } from "@/components/ui";

export type HeaderProps = {
  period: "today" | "week";
  onPeriodChange: (period: "today" | "week") => void;
  onSettingsClick: () => void;
  showSettings: boolean;
};

export const Header = ({ period, onPeriodChange, onSettingsClick, showSettings }: HeaderProps) => {
  return (
    <header className="flex items-center justify-between mb-4 pb-3 border-b border-gray-250">
      <Typography as="h1" variant="h1">
        Foxus
      </Typography>
      <div className="flex items-center gap-2">
        {!showSettings && (
          <>
            <label className="sr-only" htmlFor="period-select">
              Time period
            </label>
            <select
              id="period-select"
              value={period}
              onChange={(e) => onPeriodChange(e.target.value as "today" | "week")}
              className="font-mono text-xs bg-gray-150 border border-gray-250 px-2 py-1 uppercase tracking-wide text-gray-600 cursor-pointer hover:border-gray-300 focus:outline-none focus:border-accent-50"
            >
              <option value="today">Today</option>
              <option value="week">This Week</option>
            </select>
          </>
        )}
        <button
          onClick={onSettingsClick}
          className="font-mono text-xs text-gray-400 hover:text-gray-600 px-2 py-1"
          aria-label={showSettings ? "Close settings" : "Open settings"}
          title={showSettings ? "Close settings" : "Settings"}
        >
          {showSettings ? "[<]" : "[=]"}
        </button>
      </div>
    </header>
  );
};
