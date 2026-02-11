import type * as React from "react";
import { createContext, useContext } from "react";
import { cn } from "@/utils/helpers";

// Card variants
type CardVariant = "default" | "active";

const CardContext = createContext<CardVariant>("default");

const cardVariantClassNames: Record<CardVariant, string> = {
  default: "border-gray-250",
  active: "border-productive-50",
};

// Card
export type CardProps = React.ComponentProps<"div"> & {
  variant?: CardVariant;
};

export const Card = ({
  variant = "default",
  className,
  children,
  ...props
}: CardProps) => {
  return (
    <CardContext.Provider value={variant}>
      <div
        data-slot="card"
        data-variant={variant}
        className={cn(
          "bg-gray-150 border p-4",
          cardVariantClassNames[variant],
          className
        )}
        {...props}
      >
        {children}
      </div>
    </CardContext.Provider>
  );
};

// CardHeader
export type CardHeaderProps = React.ComponentProps<"div">;

export const CardHeader = ({ className, ...props }: CardHeaderProps) => {
  return (
    <div
      data-slot="card-header"
      className={cn("mb-3 pb-2 border-b border-gray-250", className)}
      {...props}
    />
  );
};

// CardBody
export type CardBodyProps = React.ComponentProps<"div">;

export const CardBody = ({ className, ...props }: CardBodyProps) => {
  return <div data-slot="card-body" className={cn(className)} {...props} />;
};

// CardTitle
export type CardTitleProps = React.ComponentProps<"h3">;

export const CardTitle = ({ className, ...props }: CardTitleProps) => {
  return (
    <h3
      data-slot="card-title"
      className={cn(
        "font-mono text-xs font-semibold uppercase tracking-widest text-gray-450",
        className
      )}
      {...props}
    />
  );
};

// Hook to access card context
export const useCardContext = () => useContext(CardContext);
