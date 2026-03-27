import { Bar, BarChart, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import type { DailyStats } from "@/types/api";
import { CHART_HEIGHT } from "@/utils/constants";
import { formatTime, MS_PER_SECOND, SECONDS_PER_HOUR } from "@/utils/formatters";

interface DailyBarChartProps {
  dailyStats: DailyStats[];
}

const COLORS = {
  productive: "#22c55e",
  neutral: "#f59e0b",
  distracting: "#ef4444",
};

const AXIS_STYLE = {
  fill: "#6b7280",
  fontSize: 10,
  fontFamily: "monospace",
};

const TOP_BAR_RADIUS = 4;
const CHART_LEFT_MARGIN = -20;
const CHART_TOP_MARGIN = 10;
const CHART_RIGHT_MARGIN = 10;

const formatDayLabel = (timestamp: number): string => {
  const date = new Date(timestamp * MS_PER_SECOND);
  return date.toLocaleDateString("en-US", { weekday: "short" });
};

interface ChartDataItem {
  day: string;
  productive: number;
  neutral: number;
  distracting: number;
}

interface TooltipPayloadItem {
  value?: number;
  name?: string;
  color?: string;
}

interface CustomTooltipProps {
  active?: boolean;
  payload?: TooltipPayloadItem[];
  label?: string;
}

const CustomTooltip = ({ active, payload, label }: CustomTooltipProps) => {
  if (!active || !payload || payload.length === 0) {
    return null;
  }

  return (
    <div className="bg-gray-200 border border-gray-250 p-2 text-xs">
      <p className="font-mono text-gray-400 mb-1">{label}</p>
      {payload.map((entry, index) => (
        <p key={String(index)} style={{ color: entry.color }}>
          {entry.name}: {formatTime(entry.value ?? 0)}
        </p>
      ))}
    </div>
  );
};

const formatYAxisTick = (value: number): string => {
  const hours = Math.floor(value / SECONDS_PER_HOUR);
  return hours > 0 ? `${String(hours)}h` : "";
};

const buildChartData = (dailyStats: DailyStats[]): ChartDataItem[] =>
  dailyStats.map((day) => ({
    day: formatDayLabel(day.date),
    productive: day.productive_secs,
    neutral: day.neutral_secs,
    distracting: day.distracting_secs,
  }));

const ChartContent = ({ data }: { data: ChartDataItem[] }) => (
  <ResponsiveContainer width="100%" height="100%">
    <BarChart
      data={data}
      margin={{
        top: CHART_TOP_MARGIN,
        right: CHART_RIGHT_MARGIN,
        left: CHART_LEFT_MARGIN,
        bottom: 0,
      }}
    >
      <XAxis dataKey="day" tick={AXIS_STYLE} axisLine={{ stroke: "#374151" }} tickLine={false} />
      <YAxis tick={AXIS_STYLE} axisLine={false} tickLine={false} tickFormatter={formatYAxisTick} />
      <Tooltip content={<CustomTooltip />} cursor={{ fill: "#1f2937" }} />
      <Bar dataKey="productive" stackId="a" fill={COLORS.productive} name="Productive" />
      <Bar dataKey="neutral" stackId="a" fill={COLORS.neutral} name="Neutral" />
      <Bar
        dataKey="distracting"
        stackId="a"
        fill={COLORS.distracting}
        name="Distracting"
        radius={[TOP_BAR_RADIUS, TOP_BAR_RADIUS, 0, 0]}
      />
    </BarChart>
  </ResponsiveContainer>
);

const DailyBarChart = ({ dailyStats }: DailyBarChartProps) => {
  if (!dailyStats || dailyStats.length === 0) {
    return null;
  }

  const data = buildChartData(dailyStats);

  return (
    <div className="w-full" style={{ height: `${String(CHART_HEIGHT)}px` }}>
      <ChartContent data={data} />
    </div>
  );
};

export type { DailyBarChartProps };
export { DailyBarChart };
