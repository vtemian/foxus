import { Typography } from "@/components/ui";

export type HeaderProps = {
  period: "today" | "week";
  onPeriodChange: (period: "today" | "week") => void;
};

export const Header = ({ period, onPeriodChange }: HeaderProps) => {
  return (
    <header className="flex items-center justify-between mb-4 pb-3 border-b border-gray-250">
      <Typography as="h1" variant="h1">
        Foxus
      </Typography>
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
    </header>
  );
};
