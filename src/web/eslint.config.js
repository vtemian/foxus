import js from "@eslint/js";
import sonarjs from "eslint-plugin-sonarjs";
import unicorn from "eslint-plugin-unicorn";
import tseslint from "typescript-eslint";

export default [
  {
    ignores: ["dist/**", "node_modules/**", "*.config.ts", "*.config.js"],
  },
  js.configs.recommended,
  ...tseslint.configs.recommendedTypeChecked,
  {
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
    plugins: {
      sonarjs,
      unicorn,
    },
    rules: {
      // --- Disable rules that overlap with Biome ---
      indent: "off",
      quotes: "off",
      semi: "off",
      "comma-dangle": "off",
      "no-unused-vars": "off",
      "sort-imports": "off",
      "no-multiple-empty-lines": "off",
      "eol-last": "off",

      // --- Structural limits ---
      "max-lines": ["error", { max: 200, skipBlankLines: true, skipComments: true }],
      "max-depth": ["error", 2],
      "max-lines-per-function": ["error", { max: 40, skipBlankLines: true, skipComments: true }],

      // --- TypeScript-specific ---
      "@typescript-eslint/consistent-type-imports": ["error", { prefer: "type-imports" }],
      "@typescript-eslint/consistent-type-definitions": ["error", "interface"],
      "@typescript-eslint/prefer-readonly": "error",
      "@typescript-eslint/use-unknown-in-catch-callback-variable": "error",
      "func-style": ["error", "expression"],
      "@typescript-eslint/no-unused-vars": [
        "error",
        {
          argsIgnorePattern: "^_",
          varsIgnorePattern: "^_",
          caughtErrorsIgnorePattern: "^_",
        },
      ],
      "@typescript-eslint/naming-convention": [
        "error",
        { selector: "default", format: ["camelCase"], leadingUnderscore: "allow" },
        {
          selector: "import",
          format: ["camelCase", "PascalCase"],
        },
        {
          selector: "variable",
          format: ["camelCase", "UPPER_CASE", "PascalCase"],
          leadingUnderscore: "allow",
          filter: {
            regex: "(Map|Object|String|Array|List|Set|Dict|Number|Boolean|Fn|Func|Callback)$",
            match: false,
          },
        },
        {
          selector: "function",
          format: ["camelCase", "PascalCase"],
          filter: {
            regex: "(Map|Object|String|Array|List|Set|Dict|Number|Boolean|Fn|Func|Callback)$",
            match: false,
          },
        },
        {
          selector: "parameter",
          format: ["camelCase"],
          leadingUnderscore: "allow",
          filter: {
            regex: "(Map|Object|String|Array|List|Set|Dict|Number|Boolean|Fn|Func|Callback)$",
            match: false,
          },
        },
        { selector: "typeLike", format: ["PascalCase"] },
        {
          selector: "typeProperty",
          format: ["camelCase", "snake_case"],
        },
        {
          selector: "objectLiteralProperty",
          format: null,
          filter: { regex: "^[a-z]+(_[a-z]+)+$", match: true },
        },
      ],
      "@typescript-eslint/no-magic-numbers": [
        "error",
        {
          ignore: [0, 1, -1, 2],
          ignoreEnums: true,
          ignoreNumericLiteralTypes: true,
          ignoreReadonlyClassProperties: true,
        },
      ],
      "@typescript-eslint/no-explicit-any": "error",
      "@typescript-eslint/consistent-type-assertions": ["error", { assertionStyle: "never" }],
      "@typescript-eslint/no-floating-promises": "off",
      "@typescript-eslint/no-misused-promises": "error",

      // --- Sonarjs (complexity and duplication) ---
      "sonarjs/cognitive-complexity": ["error", 10],
      "sonarjs/no-duplicate-string": ["error", { threshold: 3 }],
      "sonarjs/no-identical-functions": "error",

      // --- Import hygiene ---
      "no-restricted-imports": [
        "error",
        {
          patterns: ["../*"],
        },
      ],

      // --- Export style + async style ---
      "no-restricted-syntax": [
        "error",
        {
          selector: "ExportNamedDeclaration[declaration.type='VariableDeclaration']",
          message: "Use `export { name }` at bottom of file instead of `export const`.",
        },
        {
          selector: "CallExpression[callee.property.name='then']",
          message: "Use async/await instead of .then(). Extract async function if needed.",
        },
        {
          selector: "CallExpression[callee.property.name='finally']",
          message: "Use try/finally with async/await instead of .finally().",
        },
      ],

      // --- Unicorn (patterns) ---
      "unicorn/no-nested-ternary": "error",
    },
  },
  {
    // Relax rules for test files
    files: ["**/*.test.ts", "**/*.test.tsx", "test/**/*.ts"],
    rules: {
      "@typescript-eslint/no-explicit-any": "off",
      "@typescript-eslint/no-magic-numbers": "off",
      "func-style": "off",
      "@typescript-eslint/no-floating-promises": "off",
      "@typescript-eslint/no-unsafe-assignment": "off",
      "@typescript-eslint/naming-convention": "off",
      "@typescript-eslint/consistent-type-definitions": "off",
      "@typescript-eslint/consistent-type-assertions": "off",
      "@typescript-eslint/prefer-readonly": "off",
      "@typescript-eslint/no-unsafe-argument": "off",
      "@typescript-eslint/no-unsafe-call": "off",
      "@typescript-eslint/no-unsafe-member-access": "off",
      "sonarjs/no-duplicate-string": "off",
      "sonarjs/cognitive-complexity": "off",
      "max-depth": "off",
      "max-lines-per-function": "off",
    },
  },
];
