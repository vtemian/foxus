export default {
  "*.{ts,tsx}": (files) => {
    if (files.length === 0) return [];
    return [`biome check --write ${files.join(" ")}`, `eslint --fix ${files.join(" ")}`];
  },
  "*.{js,jsx}": ["biome check --write"],
  "*.json !*-lock.json": ["biome check --write"],
};
