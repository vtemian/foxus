import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import type { Category, MatchType, ProductivityLevel, Rule } from "@/types/api";

interface UseSettingsReturn {
  categories: Category[];
  rules: Rule[];
  isLoading: boolean;
  error: string | null;
  createCategory: (name: string, productivity: ProductivityLevel) => Promise<void>;
  updateCategory: (id: number, name: string, productivity: ProductivityLevel) => Promise<void>;
  deleteCategory: (id: number) => Promise<void>;
  createRule: (
    pattern: string,
    matchType: MatchType,
    categoryId: number,
    priority: number,
  ) => Promise<void>;
  updateRule: (
    id: number,
    pattern: string,
    matchType: MatchType,
    categoryId: number,
    priority: number,
  ) => Promise<void>;
  deleteRule: (id: number) => Promise<void>;
  refresh: () => Promise<void>;
}

const toErrorMessage = (e: unknown): string => (e instanceof Error ? e.message : String(e));

const handleOperationError = (e: unknown, setError: (msg: string) => void): never => {
  const msg = toErrorMessage(e);
  setError(msg);
  throw new Error(msg);
};

const useInvokeWithReload = (
  reload: () => Promise<void>,
  setError: (msg: string) => void,
  clearError: () => void,
) => {
  const invokeAndReload = useCallback(
    async (cmd: string, args?: Record<string, unknown>) => {
      clearError();
      try {
        await invoke(cmd, args);
        await reload();
      } catch (e: unknown) {
        handleOperationError(e, setError);
      }
    },
    [reload, setError, clearError],
  );

  return invokeAndReload;
};

const useLoadCategories = () => {
  const [categories, setCategories] = useState<Category[]>([]);

  const loadCategories = useCallback(async () => {
    try {
      setCategories(await invoke<Category[]>("get_categories"));
    } catch (e: unknown) {
      console.error("Failed to load categories:", e);
      throw e;
    }
  }, []);

  return { categories, loadCategories };
};

const useLoadRules = () => {
  const [rules, setRules] = useState<Rule[]>([]);

  const loadRules = useCallback(async () => {
    try {
      setRules(await invoke<Rule[]>("get_rules"));
    } catch (e: unknown) {
      console.error("Failed to load rules:", e);
      throw e;
    }
  }, []);

  return { rules, loadRules };
};

const useCategoryOperations = (
  loadCategories: () => Promise<void>,
  setError: (msg: string) => void,
  clearError: () => void,
) => {
  const run = useInvokeWithReload(loadCategories, setError, clearError);

  const createCategory = useCallback(
    (name: string, productivity: ProductivityLevel) =>
      run("create_category", { name, productivity }),
    [run],
  );

  const updateCategory = useCallback(
    (id: number, name: string, productivity: ProductivityLevel) =>
      run("update_category", { id, name, productivity }),
    [run],
  );

  const deleteCategory = useCallback((id: number) => run("delete_category", { id }), [run]);

  return { createCategory, updateCategory, deleteCategory };
};

const useRuleOperations = (
  loadRules: () => Promise<void>,
  setError: (msg: string) => void,
  clearError: () => void,
) => {
  const run = useInvokeWithReload(loadRules, setError, clearError);

  const createRule = useCallback(
    (pattern: string, matchType: MatchType, categoryId: number, priority: number) =>
      run("create_rule", { pattern, matchType, categoryId, priority }),
    [run],
  );

  const updateRule = useCallback(
    (id: number, pattern: string, matchType: MatchType, categoryId: number, priority: number) =>
      run("update_rule", { id, pattern, matchType, categoryId, priority }),
    [run],
  );

  const deleteRule = useCallback((id: number) => run("delete_rule", { id }), [run]);

  return { createRule, updateRule, deleteRule };
};

const useSettings = (): UseSettingsReturn => {
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const { categories, loadCategories } = useLoadCategories();
  const { rules, loadRules } = useLoadRules();

  const stableSetError = useCallback((msg: string) => setError(msg), []);
  const clearError = useCallback(() => setError(null), []);
  const categoryOps = useCategoryOperations(loadCategories, stableSetError, clearError);
  const ruleOps = useRuleOperations(loadRules, stableSetError, clearError);

  const refresh = useCallback(async () => {
    setError(null);
    try {
      await Promise.all([loadCategories(), loadRules()]);
    } catch (e: unknown) {
      setError(toErrorMessage(e));
    }
  }, [loadCategories, loadRules]);

  useEffect(() => {
    const initialize = async () => {
      await refresh();
      setIsLoading(false);
    };
    initialize();
  }, [refresh]);

  return {
    categories,
    rules,
    isLoading,
    error,
    ...categoryOps,
    ...ruleOps,
    refresh,
  };
};

export type { UseSettingsReturn };
export { useSettings };
