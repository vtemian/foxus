import type * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/utils/helpers";

const badgeVariants = cva(["inline-block rounded-none"], {
  variants: {
    variant: {
      productive: "bg-productive-50",
      neutral: "bg-neutral-50",
      distracting: "bg-distracting-50",
    },
    size: {
      dot: "w-1.5 h-1.5",
      sm: "w-2 h-2",
      md: "w-3 h-3",
    },
  },
  defaultVariants: {
    variant: "productive",
    size: "dot",
  },
});

export type BadgeProps = Omit<React.ComponentProps<"span">, "children"> &
  VariantProps<typeof badgeVariants>;

export const Badge = ({
  variant = "productive",
  size = "dot",
  className,
  ...props
}: BadgeProps) => {
  return (
    <span
      data-slot="badge"
      data-variant={variant}
      aria-hidden="true"
      className={cn(badgeVariants({ variant, size, className }))}
      {...props}
    />
  );
};

export { badgeVariants };
