import { AppListItem } from "@/components/app-list-item";
import { ProductivityPieChart } from "@/components/productivity-pie-chart";
import { StatRow } from "@/components/stat-row";
import { Card, CardBody, CardHeader, CardTitle, Typography } from "@/components/ui";
import type { TauriStats } from "@/types/api";

interface StatsViewProps {
  stats: TauriStats | null;
}

const StatsBreakdown = ({ stats, total }: { stats: TauriStats; total: number }) => (
  <Card className="mb-4">
    <CardBody className="space-y-3">
      <StatRow
        label="Productive"
        variant="productive"
        value={stats.productive_secs}
        total={total}
      />
      <StatRow label="Neutral" variant="neutral" value={stats.neutral_secs} total={total} />
      <StatRow
        label="Distracting"
        variant="distracting"
        value={stats.distracting_secs}
        total={total}
      />
    </CardBody>
  </Card>
);

const StatsTopApps = ({ stats }: { stats: TauriStats }) => (
  <Card className="mb-4">
    <CardHeader>
      <CardTitle>Top Apps</CardTitle>
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

const StatsView = ({ stats }: StatsViewProps) => {
  const total =
    (stats?.productive_secs ?? 0) + (stats?.neutral_secs ?? 0) + (stats?.distracting_secs ?? 0);

  return (
    <>
      <Card className="mb-4">
        <CardBody>
          <ProductivityPieChart
            productiveSecs={stats?.productive_secs ?? 0}
            neutralSecs={stats?.neutral_secs ?? 0}
            distractingSecs={stats?.distracting_secs ?? 0}
          />
        </CardBody>
      </Card>

      {stats && <StatsBreakdown stats={stats} total={total} />}
      {stats && <StatsTopApps stats={stats} />}
    </>
  );
};

export type { StatsViewProps };
export { StatsView };
