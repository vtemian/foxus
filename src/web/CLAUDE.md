# Frontend Rules

## Code Style

- No nesting beyond 2 levels inside a function body. Prefer early returns and small helpers
- Max function length: 40 lines (skipBlankLines, skipComments)
- Max file length: 200 lines. If a file exceeds this, split it
- Max cognitive complexity: 10 (enforced by sonarjs)
- No magic numbers/strings. Use named constants
- No duplicate string literals (3+ occurrences must be extracted)
- No nested ternaries
- No `any` types, no type assertions (`as Type`)
- No `.then()`/`.finally()` chains: use `async`/`await`
- No parameter reassignment (enforced by Biome)
- Double quotes, semicolons, trailing commas (enforced by Biome)

## TypeScript

- Prefer `interface` for contracts and `type` for unions/aliases
- Use `import type` for type-only imports
- Arrow functions everywhere (`const fn = () => {}`), no `function` declarations
- Bottom-of-file exports: `export { Name }`, never `export const`

## Quality Gate

```bash
npm run check          # full pipeline: biome + eslint + tsc + vitest
npm run lint           # biome + eslint (no tests)
npm run format         # biome format
```

Pre-commit hook runs Biome + ESLint fix on staged files via lint-staged.

## Testing

- Test real behavior, not mocked behavior
- Mock Tauri `invoke` in test setup (`test/setup.ts`)
- Place tests alongside source: `*.test.ts` / `*.test.tsx`
- Test files have relaxed ESLint rules

## Structure

```
components/
  ui/         Design system primitives (Button, Card, Typography, Badge, ProgressBar)
  settings/   Settings sub-components (split from monolithic settings-view)
  *           Feature components (Header, StatsView, FocusView, etc.)
hooks/        Custom hooks (use-tauri, use-settings)
types/        TypeScript type definitions
utils/        Utilities (formatters, cn helper)
static/css/   Global styles
```
