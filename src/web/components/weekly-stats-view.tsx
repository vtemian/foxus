import { AppListItem } from "@/components/app-list-item";
import { StatRow } from "@/components/stat-row";
import { Card, CardBody, CardHeader, CardTitle, Typography } from "@/components/ui";
import type { DailyStats, WeeklyStats } from "@/types/api";
import { formatTime, MS_PER_SECOND } from "@/utils/formatters";

interface WeeklyStatsViewProps {
  stats: WeeklyStats | null;
}

const DAY_NAMES = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const PERCENTAGE_SCALE = 100;
const MIN_BAR_HEIGHT = 4;

const formatDayLabel = (timestamp: number): string => {
  const date = new Date(timestamp * MS_PER_SECOND);
  return DAY_NAMES[date.getDay()] ?? "???";
};

const DailyBar = ({ day, maxTotal }: { day: DailyStats; maxTotal: number }) => {
  const total = day.productive_secs + day.neutral_secs + day.distracting_secs;
  const height =
    maxTotal > 0 ? Math.max((total / maxTotal) * PERCENTAGE_SCALE, MIN_BAR_HEIGHT) : MIN_BAR_HEIGHT;

  const productiveHeight = total > 0 ? (day.productive_secs / total) * height : 0;
  const neutralHeight = total > 0 ? (day.neutral_secs / total) * height : 0;
  const distractingHeight = total > 0 ? (day.distracting_secs / total) * height : 0;

  return (
    <div className="flex flex-col items-center gap-1">
      <div className="relative w-8 h-24 flex flex-col justify-end">
        <div
          className="w-full bg-red-400 rounded-t-sm"
          style={{ height: `${String(distractingHeight)}%` }}
        />
        <div className="w-full bg-amber-400" style={{ height: `${String(neutralHeight)}%` }} />
        <div
          className="w-full bg-green-500 rounded-b-sm"
          style={{ height: `${String(productiveHeight)}%` }}
        />
      </div>
      <Typography variant="label" color="muted" className="text-xs">
        {formatDayLabel(day.date)}
      </Typography>
    </div>
  );
};

const WeeklyChart = ({ stats, maxDayTotal }: { stats: WeeklyStats; maxDayTotal: number }) => (
  <Card className="mb-4">
    <CardHeader>
      <CardTitle>7-Day Overview</CardTitle>
    </CardHeader>
    <CardBody>
      {stats.daily_stats.length > 0 ? (
        <div className="flex justify-between items-end px-2">
          {stats.daily_stats.map((day) => (
            <DailyBar key={day.date} day={day} maxTotal={maxDayTotal} />
          ))}
        </div>
      ) : (
        <Typography variant="body" color="muted" className="text-center py-4">
          No weekly data available
        </Typography>
      )}
    </CardBody>
  </Card>
);

const WeeklyTotals = ({ stats, total }: { stats: WeeklyStats; total: number }) => (
  <Card className="mb-4">
    <CardHeader>
      <CardTitle>Weekly Totals</CardTitle>
    </CardHeader>
    <CardBody className="space-y-3">
      <StatRow
        label="Productive"
        variant="productive"
        value={stats.total_productive_secs}
        total={total}
      />
      <StatRow label="Neutral" variant="neutral" value={stats.total_neutral_secs} total={total} />
      <StatRow
        label="Distracting"
        variant="distracting"
        value={stats.total_distracting_secs}
        total={total}
      />
      <div className="pt-2 border-t border-gray-200">
        <div className="flex justify-between">
          <Typography variant="label" color="secondary">
            Total Time
          </Typography>
          <Typography variant="time">{formatTime(total)}</Typography>
        </div>
      </div>
    </CardBody>
  </Card>
);

const WeeklyTopApps = ({ stats }: { stats: WeeklyStats }) => (
  <Card className="mb-4">
    <CardHeader>
      <CardTitle>Top Apps (This Week)</CardTitle>
    </CardHeader>
    <CardBody>
      {stats.top_apps.length > 0 ? (
        <ul className="space-y-0">
          {stats.top_apps.map((app) => (
            <AppListItem key={app.name} app={app} />
          ))}
        </ul>
      ) : (
        <Typography variant="body" color="muted" className="text-center py-4">
          No activity tracked yet
        </Typography>
      )}
    </CardBody>
  </Card>
);

const WeeklyStatsView = ({ stats }: WeeklyStatsViewProps) => {
  const total =
    (stats?.total_productive_secs ?? 0) +
    (stats?.total_neutral_secs ?? 0) +
    (stats?.total_distracting_secs ?? 0);

  const maxDayTotal =
    stats?.daily_stats?.reduce((max, day) => {
      const dayTotal = day.productive_secs + day.neutral_secs + day.distracting_secs;
      return Math.max(max, dayTotal);
    }, 0) ?? 0;

  if (!stats) return null;

  return (
    <>
      <WeeklyChart stats={stats} maxDayTotal={maxDayTotal} />
      <WeeklyTotals stats={stats} total={total} />
      <WeeklyTopApps stats={stats} />
    </>
  );
};

export type { WeeklyStatsViewProps };
export { WeeklyStatsView };
