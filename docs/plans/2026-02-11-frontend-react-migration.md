# Foxus Frontend React Migration Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Migrate Foxus desktop app and Chrome extension from vanilla JS to React + TypeScript + Tailwind v4 + CVA, matching claudebin.com's component architecture.

**Architecture:** React 18 with Vite build tooling, TypeScript for type safety, Tailwind v4 with @theme blocks for styling, class-variance-authority (CVA) for component variants, compound components with React Context for composition. Tauri integration via custom hooks. Shared component library between desktop app and Chrome extension.

**Tech Stack:** React 18, TypeScript 5, Vite 5, Tailwind CSS 4, class-variance-authority, clsx, tailwind-merge, @tauri-apps/api

---

## Task 1: Initialize Package.json

**Files:**
- Create: `/Users/whitemonk/projects/ai/foxus/package.json`

**Step 1: Create package.json with all dependencies**

```json
{
  "name": "foxus",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "type-check": "tsc --noEmit"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "class-variance-authority": "^0.7.1",
    "clsx": "^2.1.1",
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "tailwind-merge": "^3.0.0"
  },
  "devDependencies": {
    "@tailwindcss/postcss": "^4.0.0",
    "@types/react": "^18.2.0",
    "@types/react-dom": "^18.2.0",
    "@vitejs/plugin-react": "^4.2.0",
    "postcss": "^8.4.38",
    "tailwindcss": "^4.0.0",
    "typescript": "^5.3.0",
    "vite": "^5.2.0"
  }
}
```

**Step 2: Verify file created**

Run: `cat /Users/whitemonk/projects/ai/foxus/package.json | head -5`
Expected: Shows name and version fields

**Step 3: Commit**

```bash
git add package.json
git commit -m "chore: add package.json with React and Tailwind v4 dependencies"
```

---

## Task 2: Create Vite Configuration

**Files:**
- Create: `/Users/whitemonk/projects/ai/foxus/vite.config.ts`

**Step 1: Create Vite config with React plugin and path aliases**

```typescript
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  build: {
    outDir: "dist",
    emptyOutDir: true,
  },
  server: {
    port: 1420,
    strictPort: true,
  },
  clearScreen: false,
});
```

**Step 2: Verify file created**

Run: `cat /Users/whitemonk/projects/ai/foxus/vite.config.ts | head -3`
Expected: Shows import statements

**Step 3: Commit**

```bash
git add vite.config.ts
git commit -m "chore: add Vite configuration with React plugin"
```

---

## Task 3: Create TypeScript Configuration

**Files:**
- Create: `/Users/whitemonk/projects/ai/foxus/tsconfig.json`

**Step 1: Create tsconfig.json with strict mode and path aliases**

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "baseUrl": ".",
    "paths": {
      "@/*": ["./src/*"]
    }
  },
  "include": ["src/**/*.ts", "src/**/*.tsx", "vite.config.ts"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
```

**Step 2: Create tsconfig.node.json for Vite**

```json
{
  "compilerOptions": {
    "composite": true,
    "skipLibCheck": true,
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowSyntheticDefaultImports": true
  },
  "include": ["vite.config.ts"]
}
```

**Step 3: Verify files created**

Run: `ls -la /Users/whitemonk/projects/ai/foxus/tsconfig*.json`
Expected: Shows both tsconfig.json and tsconfig.node.json

**Step 4: Commit**

```bash
git add tsconfig.json tsconfig.node.json
git commit -m "chore: add TypeScript configuration with strict mode"
```

---

## Task 4: Create PostCSS Configuration

**Files:**
- Create: `/Users/whitemonk/projects/ai/foxus/postcss.config.js`

**Step 1: Create PostCSS config for Tailwind v4**

```javascript
export default {
  plugins: {
    "@tailwindcss/postcss": {},
  },
};
```

**Step 2: Verify file created**

Run: `cat /Users/whitemonk/projects/ai/foxus/postcss.config.js`
Expected: Shows plugins object with @tailwindcss/postcss

**Step 3: Commit**

```bash
git add postcss.config.js
git commit -m "chore: add PostCSS configuration for Tailwind v4"
```

---

## Task 5: Install Dependencies

**Files:**
- Modify: `/Users/whitemonk/projects/ai/foxus/package.json` (lockfile generated)

**Step 1: Install all npm dependencies**

Run: `cd /Users/whitemonk/projects/ai/foxus && npm install`
Expected: node_modules created, package-lock.json generated

**Step 2: Verify installation**

Run: `ls /Users/whitemonk/projects/ai/foxus/node_modules | head -5`
Expected: Shows installed packages

**Step 3: Add node_modules to gitignore if not already present**

Check if .gitignore exists and contains node_modules:
```bash
grep -q "node_modules" .gitignore || echo "node_modules" >> .gitignore
```

**Step 4: Commit**

```bash
git add package-lock.json .gitignore
git commit -m "chore: install dependencies and update gitignore"
```

---

## Task 6: Create Directory Structure

**Files:**
- Create: `/Users/whitemonk/projects/ai/foxus/src/components/ui/.gitkeep`
- Create: `/Users/whitemonk/projects/ai/foxus/src/hooks/.gitkeep`
- Create: `/Users/whitemonk/projects/ai/foxus/src/utils/.gitkeep`
- Create: `/Users/whitemonk/projects/ai/foxus/src/types/.gitkeep`
- Create: `/Users/whitemonk/projects/ai/foxus/src/static/css/.gitkeep`

**Step 1: Create directory structure**

```bash
mkdir -p src/components/ui src/hooks src/utils src/types src/static/css
touch src/components/ui/.gitkeep src/hooks/.gitkeep src/utils/.gitkeep src/types/.gitkeep src/static/css/.gitkeep
```

**Step 2: Verify directories created**

Run: `find src -type d | sort`
Expected: Shows all new directories

**Step 3: Commit**

```bash
git add src/components src/hooks src/utils src/types src/static
git commit -m "chore: create React source directory structure"
```

---

## Task 7: Create Tailwind v4 Global CSS with @theme

**Files:**
- Create: `/Users/whitemonk/projects/ai/foxus/src/static/css/globals.css`

**Step 1: Create globals.css with @theme and @utility blocks**

```css
@import "tailwindcss";

@theme {
  /* ----------------------------------------
   * Color Palette: Productivity States
   * ---------------------------------------- */
  --color-productive-50: #22c55e;
  --color-productive-100: #4ade80;
  --color-productive-bg: rgba(34, 197, 94, 0.1);
  --color-productive-bg-dark: rgba(34, 197, 94, 0.15);

  --color-neutral-50: #f59e0b;
  --color-neutral-100: #fbbf24;
  --color-neutral-bg: rgba(245, 158, 11, 0.1);
  --color-neutral-bg-dark: rgba(245, 158, 11, 0.15);

  --color-distracting-50: #ef4444;
  --color-distracting-100: #f87171;
  --color-distracting-bg: rgba(239, 68, 68, 0.1);
  --color-distracting-bg-dark: rgba(239, 68, 68, 0.15);

  /* ----------------------------------------
   * Color Palette: Gray Scale (Dark Theme)
   * 100 = darkest (near black), 600 = lightest
   * ---------------------------------------- */
  --color-gray-100: #0a0a0a;
  --color-gray-150: #141414;
  --color-gray-200: #1a1a1a;
  --color-gray-250: #2a2a2a;
  --color-gray-300: #404040;
  --color-gray-350: #666666;
  --color-gray-400: #999999;
  --color-gray-450: #a0a0a0;
  --color-gray-500: #cccccc;
  --color-gray-550: #e0e0e0;
  --color-gray-600: #f0f0f0;

  /* ----------------------------------------
   * Color Palette: Accent (Blue)
   * ---------------------------------------- */
  --color-accent-50: #3b82f6;
  --color-accent-100: #2563eb;

  /* ----------------------------------------
   * Typography
   * ---------------------------------------- */
  --font-family-mono: 'IBM Plex Mono', 'SF Mono', 'Monaco', 'Inconsolata', 'Fira Mono', monospace;

  /* ----------------------------------------
   * Container
   * ---------------------------------------- */
  --spacing-container: 400px;
}

/* ----------------------------------------
 * Custom Utilities
 * ---------------------------------------- */
@utility noise-overlay {
  position: relative;
  &::before {
    content: '';
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
    opacity: 0.02;
    background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 200 200' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noise'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.65' numOctaves='3' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noise)'/%3E%3C/svg%3E");
    z-index: 9999;
  }
}

@utility font-tabular {
  font-variant-numeric: tabular-nums;
}

@utility scrollbar-hidden {
  scrollbar-width: none;
  &::-webkit-scrollbar {
    display: none;
  }
}

/* ----------------------------------------
 * Base Styles
 * ---------------------------------------- */
html {
  font-family: var(--font-family-mono);
}

body {
  background-color: var(--color-gray-100);
  color: var(--color-gray-600);
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}
```

**Step 2: Verify file created**

Run: `head -20 /Users/whitemonk/projects/ai/foxus/src/static/css/globals.css`
Expected: Shows @import and @theme block

**Step 3: Commit**

```bash
git add src/static/css/globals.css
git commit -m "style: add Tailwind v4 globals.css with @theme and @utility"
```

---

## Task 8: Create TypeScript API Types

**Files:**
- Create: `/Users/whitemonk/projects/ai/foxus/src/types/api.ts`

**Step 1: Create type definitions for Tauri API responses**

```typescript
/**
 * Productivity level for an app or activity.
 * -1 = distracting, 0 = neutral, 1 = productive
 */
export type ProductivityLevel = -1 | 0 | 1;

/**
 * A single app activity with duration and productivity score.
 */
export type AppActivity = {
  name: string;
  duration_secs: number;
  productivity: ProductivityLevel;
};

/**
 * Response from get_today_stats Tauri command.
 */
export type TauriStats = {
  productive_secs: number;
  neutral_secs: number;
  distracting_secs: number;
  top_apps: AppActivity[];
};

/**
 * Response from get_focus_state Tauri command.
 */
export type FocusState = {
  active: boolean;
  budget_remaining: number;
};

/**
 * Productivity variant for styling components.
 */
export type ProductivityVariant = "productive" | "neutral" | "distracting";

/**
 * Convert numeric productivity to variant string.
 */
export const productivityToVariant = (p: ProductivityLevel): ProductivityVariant => {
  if (p > 0) return "productive";
  if (p < 0) return "distracting";
  return "neutral";
};
```

**Step 2: Verify file created**

Run: `cat /Users/whitemonk/projects/ai/foxus/src/types/api.ts | head -10`
Expected: Shows ProductivityLevel type definition

**Step 3: Commit**

```bash
git add src/types/api.ts
git commit -m "feat: add TypeScript type definitions for Tauri API"
```

---

## Task 9: Create cn() Utility Helper

**Files:**
- Create: `/Users/whitemonk/projects/ai/foxus/src/utils/helpers.ts`

**Step 1: Create cn utility combining clsx and tailwind-merge**

```typescript
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
```

**Step 2: Verify file created**

Run: `cat /Users/whitemonk/projects/ai/foxus/src/utils/helpers.ts`
Expected: Shows cn function implementation

**Step 3: Commit**

```bash
git add src/utils/helpers.ts
git commit -m "feat: add cn() utility for class name merging"
```

---

## Task 10: Create Time Formatting Utilities

**Files:**
- Create: `/Users/whitemonk/projects/ai/foxus/src/utils/formatters.ts`

**Step 1: Create time formatting functions**

```typescript
/**
 * Format seconds as "Xh Ym" for display in stats.
 * @example formatTime(3665) // "1h 1m"
 */
export const formatTime = (secs: number): string => {
  const hours = Math.floor(secs / 3600);
  const mins = Math.floor((secs % 3600) / 60);
  return `${hours}h ${mins}m`;
};

/**
 * Format seconds as "M:SS" for focus budget countdown.
 * @example formatBudget(125) // "2:05"
 */
export const formatBudget = (secs: number): string => {
  const mins = Math.floor(secs / 60);
  const s = secs % 60;
  return `${mins}:${s.toString().padStart(2, "0")}`;
};

/**
 * Escape HTML special characters for safe DOM insertion.
 * @example escapeHtml("<script>") // "&lt;script&gt;"
 */
export const escapeHtml = (text: string): string => {
  const div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML;
};
```

**Step 2: Verify file created**

Run: `cat /Users/whitemonk/projects/ai/foxus/src/utils/formatters.ts | head -10`
Expected: Shows formatTime function

**Step 3: Commit**

```bash
git add src/utils/formatters.ts
git commit -m "feat: add time formatting utilities"
```

---

## Task 11: Create Button Component with CVA

**Files:**
- Create: `/Users/whitemonk/projects/ai/foxus/src/components/ui/button.tsx`

**Step 1: Create Button component with class-variance-authority**

```typescript
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

export type ButtonProps = React.ComponentProps<"button"> &
  VariantProps<typeof buttonVariants>;

export const Button = ({
  className,
  variant = "default",
  ...props
}: ButtonProps) => {
  return (
    <button
      data-slot="button"
      data-variant={variant}
      className={cn(buttonVariants({ variant, className }))}
      {...props}
    />
  );
};

export { buttonVariants };
```

**Step 2: Verify file created**

Run: `cat /Users/whitemonk/projects/ai/foxus/src/components/ui/button.tsx | head -15`
Expected: Shows import statements and cva call

**Step 3: Commit**

```bash
git add src/components/ui/button.tsx
git commit -m "feat: add Button component with CVA variants"
```

---

## Task 12: Create Typography Component with CVA

**Files:**
- Create: `/Users/whitemonk/projects/ai/foxus/src/components/ui/typography.tsx`

**Step 1: Create Typography component with polymorphic "as" prop**

```typescript
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
```

**Step 2: Verify file created**

Run: `cat /Users/whitemonk/projects/ai/foxus/src/components/ui/typography.tsx | head -20`
Expected: Shows import and typographyVariants definition

**Step 3: Commit**

```bash
git add src/components/ui/typography.tsx
git commit -m "feat: add Typography component with CVA and polymorphic as prop"
```

---

## Task 13: Create Card Compound Component

**Files:**
- Create: `/Users/whitemonk/projects/ai/foxus/src/components/ui/card.tsx`

**Step 1: Create Card with React Context for compound pattern**

```typescript
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
```

**Step 2: Verify file created**

Run: `cat /Users/whitemonk/projects/ai/foxus/src/components/ui/card.tsx | head -25`
Expected: Shows CardContext and Card component

**Step 3: Commit**

```bash
git add src/components/ui/card.tsx
git commit -m "feat: add Card compound component with React Context"
```

---

## Task 14: Create ProgressBar Component with CVA

**Files:**
- Create: `/Users/whitemonk/projects/ai/foxus/src/components/ui/progress-bar.tsx`

**Step 1: Create ProgressBar with productivity variants**

```typescript
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
```

**Step 2: Verify file created**

Run: `cat /Users/whitemonk/projects/ai/foxus/src/components/ui/progress-bar.tsx | head -20`
Expected: Shows progressBarVariants with productivity colors

**Step 3: Commit**

```bash
git add src/components/ui/progress-bar.tsx
git commit -m "feat: add ProgressBar component with CVA productivity variants"
```

---

## Task 15: Create Badge Component

**Files:**
- Create: `/Users/whitemonk/projects/ai/foxus/src/components/ui/badge.tsx`

**Step 1: Create Badge component for productivity indicators**

```typescript
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
```

**Step 2: Verify file created**

Run: `cat /Users/whitemonk/projects/ai/foxus/src/components/ui/badge.tsx | head -15`
Expected: Shows badgeVariants definition

**Step 3: Commit**

```bash
git add src/components/ui/badge.tsx
git commit -m "feat: add Badge component for productivity indicators"
```

---

## Task 16: Create Component Index Export

**Files:**
- Create: `/Users/whitemonk/projects/ai/foxus/src/components/ui/index.ts`

**Step 1: Create barrel export for all UI components**

```typescript
// UI Component Library
export { Button, buttonVariants, type ButtonProps } from "./button";
export { Typography, typographyVariants, type TypographyProps } from "./typography";
export {
  Card,
  CardHeader,
  CardBody,
  CardTitle,
  useCardContext,
  type CardProps,
  type CardHeaderProps,
  type CardBodyProps,
  type CardTitleProps,
} from "./card";
export { ProgressBar, progressBarVariants, type ProgressBarProps } from "./progress-bar";
export { Badge, badgeVariants, type BadgeProps } from "./badge";
```

**Step 2: Verify file created**

Run: `cat /Users/whitemonk/projects/ai/foxus/src/components/ui/index.ts`
Expected: Shows all component exports

**Step 3: Commit**

```bash
git add src/components/ui/index.ts
git commit -m "feat: add barrel export for UI components"
```

---

## Task 17: Create useTauri Hook

**Files:**
- Create: `/Users/whitemonk/projects/ai/foxus/src/hooks/use-tauri.ts`

**Step 1: Create hook for Tauri API integration**

```typescript
import { invoke } from "@tauri-apps/api/core";
import { useState, useEffect, useCallback, useRef } from "react";
import type { TauriStats, FocusState } from "@/types/api";

const REFRESH_INTERVAL = 5000; // 5 seconds

export type UseTauriReturn = {
  stats: TauriStats | null;
  focusState: FocusState | null;
  isLoading: boolean;
  error: Error | null;
  toggleFocus: () => Promise<void>;
  refresh: () => Promise<void>;
};

export const useTauri = (): UseTauriReturn => {
  const [stats, setStats] = useState<TauriStats | null>(null);
  const [focusState, setFocusState] = useState<FocusState | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);
  const refreshInProgress = useRef(false);

  const loadStats = useCallback(async () => {
    try {
      const data = await invoke<TauriStats>("get_today_stats");
      setStats(data);
      setError(null);
    } catch (e) {
      console.error("Failed to load stats:", e);
      setError(e instanceof Error ? e : new Error(String(e)));
    }
  }, []);

  const loadFocusState = useCallback(async () => {
    try {
      const state = await invoke<FocusState>("get_focus_state");
      setFocusState(state);
      setError(null);
    } catch (e) {
      console.error("Failed to load focus state:", e);
      setError(e instanceof Error ? e : new Error(String(e)));
    }
  }, []);

  const refresh = useCallback(async () => {
    if (refreshInProgress.current) return;
    refreshInProgress.current = true;
    try {
      await Promise.all([loadStats(), loadFocusState()]);
    } finally {
      refreshInProgress.current = false;
    }
  }, [loadStats, loadFocusState]);

  const toggleFocus = useCallback(async () => {
    if (!focusState) return;

    try {
      if (focusState.active) {
        await invoke("end_focus_session");
      } else {
        await invoke("start_focus_session", { budgetMinutes: 10 });
      }
      await refresh();
    } catch (e) {
      console.error("Failed to toggle focus:", e);
      setError(e instanceof Error ? e : new Error(String(e)));
    }
  }, [focusState, refresh]);

  // Initial load
  useEffect(() => {
    const initialize = async () => {
      await refresh();
      setIsLoading(false);
    };
    initialize();
  }, [refresh]);

  // Periodic refresh
  useEffect(() => {
    const interval = setInterval(refresh, REFRESH_INTERVAL);
    return () => clearInterval(interval);
  }, [refresh]);

  return { stats, focusState, isLoading, error, toggleFocus, refresh };
};
```

**Step 2: Verify file created**

Run: `cat /Users/whitemonk/projects/ai/foxus/src/hooks/use-tauri.ts | head -25`
Expected: Shows imports and type definition

**Step 3: Commit**

```bash
git add src/hooks/use-tauri.ts
git commit -m "feat: add useTauri hook for API integration"
```

---

## Task 18: Create Vite Environment Types

**Files:**
- Create: `/Users/whitemonk/projects/ai/foxus/src/vite-env.d.ts`

**Step 1: Create Vite type reference**

```typescript
/// <reference types="vite/client" />
```

**Step 2: Verify file created**

Run: `cat /Users/whitemonk/projects/ai/foxus/src/vite-env.d.ts`
Expected: Shows reference directive

**Step 3: Commit**

```bash
git add src/vite-env.d.ts
git commit -m "chore: add Vite environment type reference"
```

---

## Task 19: Create StatRow Component

**Files:**
- Create: `/Users/whitemonk/projects/ai/foxus/src/components/stat-row.tsx`

**Step 1: Create StatRow component for stats display**

```typescript
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
```

**Step 2: Verify file created**

Run: `cat /Users/whitemonk/projects/ai/foxus/src/components/stat-row.tsx`
Expected: Shows StatRow component

**Step 3: Commit**

```bash
git add src/components/stat-row.tsx
git commit -m "feat: add StatRow component for stats display"
```

---

## Task 20: Create AppListItem Component

**Files:**
- Create: `/Users/whitemonk/projects/ai/foxus/src/components/app-list-item.tsx`

**Step 1: Create AppListItem for top apps display**

```typescript
import { Typography, Badge } from "@/components/ui";
import { formatTime } from "@/utils/formatters";
import { productivityToVariant, type AppActivity } from "@/types/api";

export type AppListItemProps = {
  app: AppActivity;
};

export const AppListItem = ({ app }: AppListItemProps) => {
  const variant = productivityToVariant(app.productivity);

  return (
    <li className="flex items-center justify-between py-2 border-b border-gray-250 last:border-b-0">
      <span className="flex items-center gap-2">
        <Badge variant={variant} size="dot" />
        <Typography variant="body">{app.name}</Typography>
      </span>
      <Typography variant="time" color="secondary">
        {formatTime(app.duration_secs)}
      </Typography>
    </li>
  );
};
```

**Step 2: Verify file created**

Run: `cat /Users/whitemonk/projects/ai/foxus/src/components/app-list-item.tsx`
Expected: Shows AppListItem component

**Step 3: Commit**

```bash
git add src/components/app-list-item.tsx
git commit -m "feat: add AppListItem component for top apps list"
```

---

## Task 21: Create StatsView Component

**Files:**
- Create: `/Users/whitemonk/projects/ai/foxus/src/components/stats-view.tsx`

**Step 1: Create StatsView container component**

```typescript
import { Card, CardHeader, CardBody, CardTitle, Typography } from "@/components/ui";
import { StatRow } from "./stat-row";
import { AppListItem } from "./app-list-item";
import type { TauriStats } from "@/types/api";

export type StatsViewProps = {
  stats: TauriStats | null;
};

export const StatsView = ({ stats }: StatsViewProps) => {
  const total =
    (stats?.productive_secs ?? 0) +
    (stats?.neutral_secs ?? 0) +
    (stats?.distracting_secs ?? 0);

  return (
    <>
      {/* Stats Bars */}
      <Card className="mb-4">
        <CardBody className="space-y-3">
          <StatRow
            label="Productive"
            variant="productive"
            value={stats?.productive_secs ?? 0}
            total={total}
          />
          <StatRow
            label="Neutral"
            variant="neutral"
            value={stats?.neutral_secs ?? 0}
            total={total}
          />
          <StatRow
            label="Distracting"
            variant="distracting"
            value={stats?.distracting_secs ?? 0}
            total={total}
          />
        </CardBody>
      </Card>

      {/* Top Apps */}
      <Card className="mb-4">
        <CardHeader>
          <CardTitle>Top Apps</CardTitle>
        </CardHeader>
        <CardBody>
          {stats?.top_apps && stats.top_apps.length > 0 ? (
            <ul className="space-y-0">
              {stats.top_apps.map((app) => (
                <AppListItem key={app.name} app={app} />
              ))}
            </ul>
          ) : (
            <Typography variant="body" color="muted" className="text-center py-4">
              No activity tracked yet
            </Typography>
          )}
        </CardBody>
      </Card>
    </>
  );
};
```

**Step 2: Verify file created**

Run: `cat /Users/whitemonk/projects/ai/foxus/src/components/stats-view.tsx | head -30`
Expected: Shows StatsView component with imports

**Step 3: Commit**

```bash
git add src/components/stats-view.tsx
git commit -m "feat: add StatsView container component"
```

---

## Task 22: Create FocusView Component

**Files:**
- Create: `/Users/whitemonk/projects/ai/foxus/src/components/focus-view.tsx`

**Step 1: Create FocusView for active focus session**

```typescript
import { Card, CardBody, Typography } from "@/components/ui";
import { formatBudget } from "@/utils/formatters";

export type FocusViewProps = {
  budgetRemaining: number;
};

export const FocusView = ({ budgetRemaining }: FocusViewProps) => {
  return (
    <Card variant="active" className="mb-4">
      <CardBody className="py-6 text-center">
        <Typography variant="h2" color="productive" className="mb-4">
          Focus Mode Active
        </Typography>
        <div className="space-y-1">
          <Typography variant="label" color="muted">
            Budget remaining
          </Typography>
          <Typography variant="budget">
            {formatBudget(budgetRemaining)}
          </Typography>
        </div>
      </CardBody>
    </Card>
  );
};
```

**Step 2: Verify file created**

Run: `cat /Users/whitemonk/projects/ai/foxus/src/components/focus-view.tsx`
Expected: Shows FocusView component

**Step 3: Commit**

```bash
git add src/components/focus-view.tsx
git commit -m "feat: add FocusView component for active focus display"
```

---

## Task 23: Create Header Component

**Files:**
- Create: `/Users/whitemonk/projects/ai/foxus/src/components/header.tsx`

**Step 1: Create Header component with period selector**

```typescript
import { Typography } from "@/components/ui";

export type HeaderProps = {
  period: "today" | "week";
  onPeriodChange: (period: "today" | "week") => void;
};

export const Header = ({ period, onPeriodChange }: HeaderProps) => {
  return (
    <header className="flex items-center justify-between mb-4 pb-3 border-b border-gray-250">
      <Typography as="h1" variant="h1">
        Foxus
      </Typography>
      <label className="sr-only" htmlFor="period-select">
        Time period
      </label>
      <select
        id="period-select"
        value={period}
        onChange={(e) => onPeriodChange(e.target.value as "today" | "week")}
        className="font-mono text-xs bg-gray-150 border border-gray-250 px-2 py-1 uppercase tracking-wide text-gray-600 cursor-pointer hover:border-gray-300 focus:outline-none focus:border-accent-50"
      >
        <option value="today">Today</option>
        <option value="week">This Week</option>
      </select>
    </header>
  );
};
```

**Step 2: Verify file created**

Run: `cat /Users/whitemonk/projects/ai/foxus/src/components/header.tsx`
Expected: Shows Header component with select

**Step 3: Commit**

```bash
git add src/components/header.tsx
git commit -m "feat: add Header component with period selector"
```

---

## Task 24: Create Main App Component

**Files:**
- Create: `/Users/whitemonk/projects/ai/foxus/src/App.tsx`

**Step 1: Create main App component**

```typescript
import { useState } from "react";
import "@/static/css/globals.css";

import { useTauri } from "@/hooks/use-tauri";
import { Button, Typography } from "@/components/ui";
import { Header } from "@/components/header";
import { StatsView } from "@/components/stats-view";
import { FocusView } from "@/components/focus-view";

export default function App() {
  const { stats, focusState, isLoading, error, toggleFocus } = useTauri();
  const [period, setPeriod] = useState<"today" | "week">("today");

  const isFocusActive = focusState?.active ?? false;

  if (isLoading) {
    return (
      <div className="min-h-screen bg-gray-100 noise-overlay">
        <div className="max-w-[400px] mx-auto p-4">
          <Typography variant="body" color="muted" className="text-center py-8">
            Loading...
          </Typography>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="min-h-screen bg-gray-100 noise-overlay">
        <div className="max-w-[400px] mx-auto p-4">
          <Typography variant="body" color="distracting" className="text-center py-8">
            Failed to connect to Foxus backend
          </Typography>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-100 noise-overlay">
      <div className="max-w-[400px] mx-auto p-4">
        <Header period={period} onPeriodChange={setPeriod} />

        {!isFocusActive && <StatsView stats={stats} />}
        {isFocusActive && (
          <FocusView budgetRemaining={focusState?.budget_remaining ?? 0} />
        )}

        <footer className="mt-4">
          <Button
            variant={isFocusActive ? "focus" : "default"}
            onClick={toggleFocus}
            aria-pressed={isFocusActive}
          >
            {isFocusActive ? "End Focus Session" : "Start Focus Session"}
          </Button>
        </footer>
      </div>
    </div>
  );
}
```

**Step 2: Verify file created**

Run: `cat /Users/whitemonk/projects/ai/foxus/src/App.tsx | head -30`
Expected: Shows imports and App function

**Step 3: Commit**

```bash
git add src/App.tsx
git commit -m "feat: add main App component with all views"
```

---

## Task 25: Create React Entry Point

**Files:**
- Create: `/Users/whitemonk/projects/ai/foxus/src/main.tsx`

**Step 1: Create React 18 entry point**

```typescript
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";

const root = document.getElementById("root");

if (!root) {
  throw new Error("Root element not found. Add <div id='root'></div> to index.html");
}

ReactDOM.createRoot(root).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
```

**Step 2: Verify file created**

Run: `cat /Users/whitemonk/projects/ai/foxus/src/main.tsx`
Expected: Shows ReactDOM.createRoot

**Step 3: Commit**

```bash
git add src/main.tsx
git commit -m "feat: add React 18 entry point"
```

---

## Task 26: Create New index.html

**Files:**
- Backup: `/Users/whitemonk/projects/ai/foxus/src/index.html` → `/Users/whitemonk/projects/ai/foxus/src/_archived/index.html.bak`
- Create: `/Users/whitemonk/projects/ai/foxus/src/index.html`

**Step 1: Create archived directory and backup old files**

```bash
mkdir -p src/_archived
cp src/index.html src/_archived/index.html.bak
cp src/style.css src/_archived/style.css.bak
cp src/app.js src/_archived/app.js.bak
```

**Step 2: Create new minimal index.html**

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Foxus</title>
    <link rel="preconnect" href="https://fonts.googleapis.com" />
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
    <link
      href="https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500;600&display=swap"
      rel="stylesheet"
    />
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/main.tsx"></script>
  </body>
</html>
```

**Step 3: Remove old style.css and app.js (now archived)**

```bash
rm src/style.css src/app.js
```

**Step 4: Verify new structure**

Run: `ls -la src/`
Expected: Shows index.html, main.tsx, App.tsx, and directories

**Step 5: Commit**

```bash
git add src/index.html src/_archived/ src/style.css src/app.js
git commit -m "refactor: replace vanilla JS with React entry point

- Archive old index.html, style.css, app.js
- Create minimal React shell
- Remove old vanilla JS files"
```

---

## Task 27: Update Tauri Configuration

**Files:**
- Modify: `/Users/whitemonk/projects/ai/foxus/src-tauri/tauri.conf.json`

**Step 1: Read current tauri.conf.json**

Run: `cat /Users/whitemonk/projects/ai/foxus/src-tauri/tauri.conf.json | head -20`

**Step 2: Update build configuration**

Change the build section to:
```json
{
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:1420",
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build"
  }
}
```

**Step 3: Verify changes**

Run: `cat /Users/whitemonk/projects/ai/foxus/src-tauri/tauri.conf.json | grep -A4 '"build"'`
Expected: Shows updated frontendDist and devUrl

**Step 4: Commit**

```bash
git add src-tauri/tauri.conf.json
git commit -m "chore: update Tauri config for Vite build"
```

---

## Task 28: Verify Build Works

**Files:**
- None (verification only)

**Step 1: Run type check**

Run: `cd /Users/whitemonk/projects/ai/foxus && npm run type-check`
Expected: No TypeScript errors

**Step 2: Run Vite build**

Run: `npm run build`
Expected: Creates dist/ directory with built files

**Step 3: Verify dist output**

Run: `ls -la dist/`
Expected: Shows index.html and assets/

**Step 4: Commit dist to gitignore**

```bash
echo "dist" >> .gitignore
git add .gitignore
git commit -m "chore: add dist to gitignore"
```

---

## Task 29: Test Tauri Development Mode

**Files:**
- None (verification only)

**Step 1: Run Tauri dev**

Run: `cd /Users/whitemonk/projects/ai/foxus && cargo tauri dev`
Expected: Opens Foxus window with React UI

**Step 2: Verify functionality**

- Stats bars should display
- Focus button should toggle
- Period selector should work

**Step 3: Commit any fixes if needed**

---

## Task 30: Create Component Index for Non-UI Components

**Files:**
- Create: `/Users/whitemonk/projects/ai/foxus/src/components/index.ts`

**Step 1: Create barrel export for app components**

```typescript
export { Header, type HeaderProps } from "./header";
export { StatsView, type StatsViewProps } from "./stats-view";
export { FocusView, type FocusViewProps } from "./focus-view";
export { StatRow, type StatRowProps } from "./stat-row";
export { AppListItem, type AppListItemProps } from "./app-list-item";
```

**Step 2: Verify file created**

Run: `cat /Users/whitemonk/projects/ai/foxus/src/components/index.ts`
Expected: Shows all exports

**Step 3: Commit**

```bash
git add src/components/index.ts
git commit -m "feat: add barrel export for app components"
```

---

## Task 31: Final Cleanup and Documentation

**Files:**
- Remove: `/Users/whitemonk/projects/ai/foxus/src/components/ui/.gitkeep`
- Remove: `/Users/whitemonk/projects/ai/foxus/src/hooks/.gitkeep`
- Remove: `/Users/whitemonk/projects/ai/foxus/src/utils/.gitkeep`
- Remove: `/Users/whitemonk/projects/ai/foxus/src/types/.gitkeep`
- Remove: `/Users/whitemonk/projects/ai/foxus/src/static/css/.gitkeep`

**Step 1: Remove .gitkeep files**

```bash
find src -name ".gitkeep" -delete
```

**Step 2: Final commit**

```bash
git add -A
git commit -m "feat: complete React migration

- React 18 + TypeScript + Vite setup
- Tailwind v4 with @theme blocks
- CVA component library (Button, Typography, Card, ProgressBar, Badge)
- Compound components with React Context
- useTauri hook for API integration
- All vanilla JS migrated to React components"
```

---

## Extension Migration Tasks (Phase 2)

> Note: These tasks are for the Chrome extension migration. Execute after desktop app is verified working.

### Task 32-40: Extension React Build Setup

(Detailed tasks for extension migration would follow the same pattern - creating Vite entry points for popup.html and blocked.html, bundling fonts locally, etc.)

---

## Verification Checklist

### Desktop App
- [ ] `npm install` succeeds
- [ ] `npm run type-check` passes with no errors
- [ ] `npm run build` creates dist/ directory
- [ ] `cargo tauri dev` launches app
- [ ] Stats load and display correctly
- [ ] Focus toggle works
- [ ] Progress bars animate
- [ ] Period selector visible (even if week not implemented)

### Code Quality
- [ ] No TypeScript errors
- [ ] All components use CVA patterns
- [ ] All components have data-slot attributes
- [ ] cn() utility used for class merging
- [ ] Types defined for all API responses
