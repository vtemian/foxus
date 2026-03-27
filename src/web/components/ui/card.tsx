import { createContext, useContext } from "react";
import { cn } from "@/utils/helpers";

type CardVariant = "default" | "active";

const CardContext = createContext<CardVariant>("default");

const CARD_VARIANT_CLASS_NAMES: Record<CardVariant, string> = {
  default: "border-gray-250",
  active: "border-productive-50",
};

const CARD_HEADER_VARIANT_CLASSES: Record<CardVariant, string> = {
  default: "border-gray-250",
  active: "border-productive-50",
};

const CARD_BODY_VARIANT_CLASSES: Record<CardVariant, string> = {
  default: "",
  active: "",
};

const CARD_TITLE_VARIANT_CLASSES: Record<CardVariant, string> = {
  default: "text-gray-450",
  active: "text-productive-50",
};

type CardProps = React.ComponentProps<"div"> & {
  variant?: CardVariant;
};

const Card = ({ variant = "default", className, children, ...props }: CardProps) => {
  return (
    <CardContext.Provider value={variant}>
      <div
        data-slot="card"
        data-variant={variant}
        className={cn("bg-gray-150 border p-4", CARD_VARIANT_CLASS_NAMES[variant], className)}
        {...props}
      >
        {children}
      </div>
    </CardContext.Provider>
  );
};

type CardHeaderProps = React.ComponentProps<"div">;

const CardHeader = ({ className, ...props }: CardHeaderProps) => {
  const variant = useContext(CardContext);
  return (
    <div
      data-slot="card-header"
      className={cn("mb-3 pb-2 border-b", CARD_HEADER_VARIANT_CLASSES[variant], className)}
      {...props}
    />
  );
};

type CardBodyProps = React.ComponentProps<"div">;

const CardBody = ({ className, ...props }: CardBodyProps) => {
  const variant = useContext(CardContext);
  return (
    <div
      data-slot="card-body"
      className={cn(CARD_BODY_VARIANT_CLASSES[variant], className)}
      {...props}
    />
  );
};

type CardTitleProps = React.ComponentProps<"h3">;

const CardTitle = ({ className, ...props }: CardTitleProps) => {
  const variant = useContext(CardContext);
  return (
    <h3
      data-slot="card-title"
      className={cn(
        "font-mono text-xs font-semibold uppercase tracking-widest",
        CARD_TITLE_VARIANT_CLASSES[variant],
        className,
      )}
      {...props}
    />
  );
};

export type { CardBodyProps, CardHeaderProps, CardProps, CardTitleProps };
export { Card, CardBody, CardHeader, CardTitle };
