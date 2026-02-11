import type * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/utils/helpers";

const typographyVariants = cva(["font-mono"], {
  variants: {
    variant: {
      h1: "text-sm font-semibold uppercase tracking-widest",
      h2: "text-xs font-semibold uppercase tracking-widest",
      h3: "text-xs font-semibold uppercase tracking-wide",
      body: "text-xs",
      label: "text-[10px] uppercase tracking-widest",
      time: "text-xs font-medium font-tabular tracking-wide",
      budget: "text-2xl font-semibold font-tabular tracking-wide",
    },
    color: {
      default: "text-gray-600",
      secondary: "text-gray-450",
      muted: "text-gray-350",
      productive: "text-productive-50",
      neutral: "text-neutral-50",
      distracting: "text-distracting-50",
      accent: "text-accent-50",
    },
  },
  defaultVariants: {
    variant: "body",
    color: "default",
  },
});

type VariantElementMap = {
  h1: "h1";
  h2: "h2";
  h3: "h3";
  body: "p";
  label: "span";
  time: "span";
  budget: "span";
};

type Variant = keyof VariantElementMap;

const defaultElements: VariantElementMap = {
  h1: "h1",
  h2: "h2",
  h3: "h3",
  body: "p",
  label: "span",
  time: "span",
  budget: "span",
};

export type TypographyProps<T extends React.ElementType = "span"> = {
  as?: T;
  variant?: Variant;
} & Omit<React.ComponentPropsWithoutRef<T>, "as"> &
  VariantProps<typeof typographyVariants>;

export const Typography = <T extends React.ElementType = "span">({
  as,
  variant = "body",
  color = "default",
  className,
  ...props
}: TypographyProps<T>) => {
  const Component = as || defaultElements[variant] || "span";

  return (
    <Component
      data-slot="typography"
      data-variant={variant}
      className={cn(typographyVariants({ variant, color, className }))}
      {...props}
    />
  );
};

export { typographyVariants };
