import type * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/utils/helpers";

const progressBarVariants = cva(["h-1 transition-all duration-300 ease-out"], {
  variants: {
    variant: {
      productive: "bg-productive-50",
      neutral: "bg-neutral-50",
      distracting: "bg-distracting-50",
    },
  },
  defaultVariants: {
    variant: "productive",
  },
});

export type ProgressBarProps = {
  value: number;
  max: number;
} & Omit<React.ComponentProps<"div">, "children"> &
  VariantProps<typeof progressBarVariants>;

export const ProgressBar = ({
  value,
  max,
  variant = "productive",
  className,
  ...props
}: ProgressBarProps) => {
  const percentage = max > 0 ? Math.min((value / max) * 100, 100) : 0;

  return (
    <div
      data-slot="progress-bar"
      data-variant={variant}
      role="progressbar"
      aria-valuenow={value}
      aria-valuemin={0}
      aria-valuemax={max}
      className={cn("h-1 flex-1 bg-gray-250 overflow-hidden", className)}
      {...props}
    >
      <div
        className={cn(progressBarVariants({ variant }))}
        style={{ width: `${percentage}%` }}
      />
    </div>
  );
};

export { progressBarVariants };
