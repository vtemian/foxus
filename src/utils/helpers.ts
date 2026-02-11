import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

/**
 * Merge class names with Tailwind conflict resolution.
 * Combines clsx for conditional classes and tailwind-merge
 * to properly handle conflicting Tailwind utilities.
 *
 * @example
 * cn("px-4 py-2", condition && "bg-red-500", className)
 */
export const cn = (...inputs: ClassValue[]): string => {
  return twMerge(clsx(inputs));
};
