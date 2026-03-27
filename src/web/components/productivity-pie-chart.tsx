import { Cell, Legend, Pie, PieChart, ResponsiveContainer } from "recharts";
import { CHART_HEIGHT } from "@/utils/constants";
import { formatTime } from "@/utils/formatters";

interface ProductivityPieChartProps {
  productiveSecs: number;
  neutralSecs: number;
  distractingSecs: number;
}

const COLORS = {
  productive: "#22c55e",
  neutral: "#f59e0b",
  distracting: "#ef4444",
};

interface DataItem {
  name: string;
  value: number;
  color: string;
}

interface LegendPayloadItem {
  value?: string | number;
  color?: string;
}

interface LegendContentProps {
  payload?: readonly LegendPayloadItem[];
  data: DataItem[];
}

const PIE_INNER_RADIUS = 40;
const PIE_OUTER_RADIUS = 60;
const PIE_PADDING_ANGLE = 2;

const LegendContent = ({ payload, data }: LegendContentProps) => {
  if (!payload) return null;
  return (
    <ul className="flex justify-center gap-4 text-xs">
      {payload.map((entry: LegendPayloadItem, index: number) => {
        const item = data.find((d: DataItem) => d.name === entry.value);
        return (
          <li key={`legend-${String(index)}`} className="flex items-center gap-1.5">
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

const renderLegend = (props: { payload?: readonly LegendPayloadItem[] }, data: DataItem[]) => (
  <LegendContent payload={props.payload} data={data} />
);

const buildChartData = (
  productiveSecs: number,
  neutralSecs: number,
  distractingSecs: number,
): DataItem[] =>
  [
    { name: "Productive", value: productiveSecs, color: COLORS.productive },
    { name: "Neutral", value: neutralSecs, color: COLORS.neutral },
    { name: "Distracting", value: distractingSecs, color: COLORS.distracting },
  ].filter((item) => item.value > 0);

const ProductivityPieChart = ({
  productiveSecs,
  neutralSecs,
  distractingSecs,
}: ProductivityPieChartProps) => {
  const total = productiveSecs + neutralSecs + distractingSecs;
  if (total === 0) return null;

  const data = buildChartData(productiveSecs, neutralSecs, distractingSecs);

  return (
    <div className="w-full" style={{ height: `${String(CHART_HEIGHT)}px` }}>
      <ResponsiveContainer width="100%" height="100%">
        <PieChart>
          <Pie
            data={data}
            cx="50%"
            cy="45%"
            innerRadius={PIE_INNER_RADIUS}
            outerRadius={PIE_OUTER_RADIUS}
            paddingAngle={PIE_PADDING_ANGLE}
            dataKey="value"
            stroke="none"
          >
            {data.map((entry, index) => (
              <Cell key={`cell-${String(index)}`} fill={entry.color} />
            ))}
          </Pie>
          <Legend content={(props) => renderLegend(props, data)} verticalAlign="bottom" />
        </PieChart>
      </ResponsiveContainer>
    </div>
  );
};

export type { ProductivityPieChartProps };
export { ProductivityPieChart };
