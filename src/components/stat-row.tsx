import { Typography, ProgressBar } from "@/components/ui";
import { formatTime } from "@/utils/formatters";
import type { ProductivityVariant } from "@/types/api";

export type StatRowProps = {
  label: string;
  variant: ProductivityVariant;
  value: number;
  total: number;
};

export const StatRow = ({ label, variant, value, total }: StatRowProps) => {
  return (
    <div className="flex items-center gap-3">
      <Typography variant="label" color="secondary" className="w-24">
        {label}
      </Typography>
      <ProgressBar value={value} max={total} variant={variant} />
      <Typography variant="time" className="w-16 text-right">
        {formatTime(value)}
      </Typography>
    </div>
  );
};
