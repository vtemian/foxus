import type { IconButtonProps, TabButtonProps } from "@/components/settings/types";
import type { ProductivityLevel } from "@/types/api";

const INPUT_CLASS =
  "w-full font-mono text-xs bg-gray-100 border border-gray-250 px-2 py-1 focus:outline-none focus:border-gray-400";

const BUTTON_MUTED_CLASS = "font-mono text-xs text-gray-400 hover:text-gray-600 px-2 py-1";

const TabButton = ({ active, onClick, children }: TabButtonProps) => (
  <button
    type="button"
    onClick={onClick}
    className={`font-mono text-xs uppercase tracking-wide px-3 py-1 transition-colors ${
      active
        ? "text-gray-600 border-b-2 border-gray-600 -mb-[9px]"
        : "text-gray-400 hover:text-gray-600"
    }`}
  >
    {children}
  </button>
);

const IconButton = ({ onClick, label, children }: IconButtonProps) => (
  <button
    type="button"
    onClick={onClick}
    aria-label={label}
    className="font-mono text-[10px] text-gray-400 hover:text-gray-600 w-5 h-5 flex items-center justify-center"
  >
    {children}
  </button>
);

const productivityLabel = (p: ProductivityLevel): string => {
  if (p > 0) return "productive";
  if (p < 0) return "distracting";
  return "neutral";
};

export { BUTTON_MUTED_CLASS, IconButton, INPUT_CLASS, productivityLabel, TabButton };
