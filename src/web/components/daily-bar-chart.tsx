import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  ResponsiveContainer,
  Tooltip,
} from "recharts";
import type { DailyStats } from "@/types/api";
import { formatTime } from "@/utils/formatters";

export type DailyBarChartProps = {
  dailyStats: DailyStats[];
};

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

const formatDayLabel = (timestamp: number): string => {
  const date = new Date(timestamp * 1000);
  return date.toLocaleDateString("en-US", { weekday: "short" });
};

type ChartDataItem = {
  day: string;
  productive: number;
  neutral: number;
  distracting: number;
};

type TooltipPayloadItem = {
  value?: number;
  name?: string;
  color?: string;
};

type CustomTooltipProps = {
  active?: boolean;
  payload?: TooltipPayloadItem[];
  label?: string;
};

const CustomTooltip = ({ active, payload, label }: CustomTooltipProps) => {
  if (!active || !payload || payload.length === 0) {
    return null;
  }

  return (
    <div className="bg-gray-200 border border-gray-250 p-2 text-xs">
      <p className="font-mono text-gray-400 mb-1">{label}</p>
      {payload.map((entry, index) => (
        <p key={index} style={{ color: entry.color }}>
          {entry.name}: {formatTime(entry.value ?? 0)}
        </p>
      ))}
    </div>
  );
};

export const DailyBarChart = ({ dailyStats }: DailyBarChartProps) => {
  if (!dailyStats || dailyStats.length === 0) {
    return null;
  }

  const data: ChartDataItem[] = dailyStats.map((day) => ({
    day: formatDayLabel(day.date),
    productive: day.productive_secs,
    neutral: day.neutral_secs,
    distracting: day.distracting_secs,
  }));

  return (
    <div className="w-full h-[180px]">
      <ResponsiveContainer width="100%" height="100%">
        <BarChart data={data} margin={{ top: 10, right: 10, left: -20, bottom: 0 }}>
          <XAxis
            dataKey="day"
            tick={AXIS_STYLE}
            axisLine={{ stroke: "#374151" }}
            tickLine={false}
          />
          <YAxis
            tick={AXIS_STYLE}
            axisLine={false}
            tickLine={false}
            tickFormatter={(value) => {
              const hours = Math.floor(value / 3600);
              return hours > 0 ? `${hours}h` : "";
            }}
          />
          <Tooltip content={<CustomTooltip />} cursor={{ fill: "#1f2937" }} />
          <Bar
            dataKey="productive"
            stackId="a"
            fill={COLORS.productive}
            name="Productive"
            radius={[0, 0, 0, 0]}
          />
          <Bar
            dataKey="neutral"
            stackId="a"
            fill={COLORS.neutral}
            name="Neutral"
            radius={[0, 0, 0, 0]}
          />
          <Bar
            dataKey="distracting"
            stackId="a"
            fill={COLORS.distracting}
            name="Distracting"
            radius={[4, 4, 0, 0]}
          />
        </BarChart>
      </ResponsiveContainer>
    </div>
  );
};
