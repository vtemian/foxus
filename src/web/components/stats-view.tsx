import { Card, CardHeader, CardBody, CardTitle, Typography } from "@/components/ui";
import { StatRow } from "./stat-row";
import { AppListItem } from "./app-list-item";
import type { TauriStats } from "@/types/api";

export type StatsViewProps = {
  stats: TauriStats | null;
};

export const StatsView = ({ stats }: StatsViewProps) => {
  const total =
    (stats?.productive_secs ?? 0) +
    (stats?.neutral_secs ?? 0) +
    (stats?.distracting_secs ?? 0);

  return (
    <>
      {/* Stats Bars */}
      <Card className="mb-4">
        <CardBody className="space-y-3">
          <StatRow
            label="Productive"
            variant="productive"
            value={stats?.productive_secs ?? 0}
            total={total}
          />
          <StatRow
            label="Neutral"
            variant="neutral"
            value={stats?.neutral_secs ?? 0}
            total={total}
          />
          <StatRow
            label="Distracting"
            variant="distracting"
            value={stats?.distracting_secs ?? 0}
            total={total}
          />
        </CardBody>
      </Card>

      {/* Top Apps */}
      <Card className="mb-4">
        <CardHeader>
          <CardTitle>Top Apps</CardTitle>
        </CardHeader>
        <CardBody>
          {stats?.top_apps && stats.top_apps.length > 0 ? (
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
    </>
  );
};
