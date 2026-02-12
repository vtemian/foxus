import { PieChart, Pie, Cell, ResponsiveContainer, Legend } from "recharts";
import { formatTime } from "@/utils/formatters";

export type ProductivityPieChartProps = {
  productiveSecs: number;
  neutralSecs: number;
  distractingSecs: number;
};

const COLORS = {
  productive: "#22c55e",
  neutral: "#f59e0b",
  distracting: "#ef4444",
};

type DataItem = {
  name: string;
  value: number;
  color: string;
};

type LegendPayloadItem = {
  value: string | number;
  color?: string;
};

type LegendContentProps = {
  payload?: readonly LegendPayloadItem[];
  data: DataItem[];
};

const LegendContent = ({ payload, data }: LegendContentProps) => {
  if (!payload) return null;
  return (
    <ul className="flex justify-center gap-4 text-xs">
      {payload.map((entry: LegendPayloadItem, index: number) => {
        const item = data.find((d: DataItem) => d.name === entry.value);
        return (
          <li key={`legend-${index}`} className="flex items-center gap-1.5">
            <span
              className="inline-block w-2.5 h-2.5 rounded-full"
              style={{ backgroundColor: entry.color }}
            />
            <span className="text-gray-400">
              {String(entry.value)}: {formatTime(item?.value ?? 0)}
            </span>
          </li>
        );
      })}
    </ul>
  );
};

export const ProductivityPieChart = ({
  productiveSecs,
  neutralSecs,
  distractingSecs,
}: ProductivityPieChartProps) => {
  const total = productiveSecs + neutralSecs + distractingSecs;

  // Don't render if no data
  if (total === 0) {
    return null;
  }

  const data: DataItem[] = [
    { name: "Productive", value: productiveSecs, color: COLORS.productive },
    { name: "Neutral", value: neutralSecs, color: COLORS.neutral },
    { name: "Distracting", value: distractingSecs, color: COLORS.distracting },
  ].filter((item) => item.value > 0);

  return (
    <div className="w-full h-[180px]">
      <ResponsiveContainer width="100%" height="100%">
        <PieChart>
          <Pie
            data={data}
            cx="50%"
            cy="45%"
            innerRadius={40}
            outerRadius={60}
            paddingAngle={2}
            dataKey="value"
            stroke="none"
          >
            {data.map((entry, index) => (
              <Cell key={`cell-${index}`} fill={entry.color} />
            ))}
          </Pie>
          <Legend
            content={(props) => (
              <LegendContent
                payload={props.payload as readonly LegendPayloadItem[]}
                data={data}
              />
            )}
            verticalAlign="bottom"
          />
        </PieChart>
      </ResponsiveContainer>
    </div>
  );
};
