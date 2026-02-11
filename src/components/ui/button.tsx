import type * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/utils/helpers";

const buttonVariants = cva(
  [
    "inline-flex items-center justify-center",
    "w-full",
    "px-4 py-3",
    "font-mono text-xs font-medium uppercase tracking-widest",
    "border",
    "cursor-pointer select-none",
    "transition-all duration-150 ease-in-out",
    "disabled:pointer-events-none disabled:opacity-50",
  ],
  {
    variants: {
      variant: {
        default: [
          "bg-gray-150 border-gray-250 text-gray-600",
          "hover:bg-gray-200 hover:border-gray-300",
          "active:translate-y-px",
        ],
        focus: [
          "bg-distracting-bg border-distracting-50 text-distracting-50",
          "hover:bg-distracting-bg-dark",
          "active:translate-y-px",
        ],
        ghost: [
          "border-transparent bg-transparent text-gray-400",
          "hover:bg-gray-200 hover:text-gray-600",
        ],
      },
    },
    defaultVariants: {
      variant: "default",
    },
  }
);

export type ButtonProps<T extends React.ElementType = "button"> = {
  as?: T;
} & Omit<React.ComponentPropsWithoutRef<T>, "as" | "className"> &
  VariantProps<typeof buttonVariants>;

export const Button = <T extends React.ElementType = "button">({
  as,
  className,
  variant = "default",
  ...props
}: ButtonProps<T>) => {
  const Component = as || "button";
  return (
    <Component
      data-slot="button"
      data-variant={variant}
      className={cn(buttonVariants({ variant, className }))}
      {...props}
    />
  );
};

export { buttonVariants };
