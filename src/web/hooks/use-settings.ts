import { invoke } from "@tauri-apps/api/core";
import { useState, useEffect, useCallback } from "react";
import type { Category, Rule, ProductivityLevel, MatchType } from "@/types/api";

export type UseSettingsReturn = {
  categories: Category[];
  rules: Rule[];
  isLoading: boolean;
  error: string | null;
  // Category operations
  createCategory: (name: string, productivity: ProductivityLevel) => Promise<void>;
  updateCategory: (id: number, name: string, productivity: ProductivityLevel) => Promise<void>;
  deleteCategory: (id: number) => Promise<void>;
  // Rule operations
  createRule: (pattern: string, matchType: MatchType, categoryId: number, priority: number) => Promise<void>;
  updateRule: (id: number, pattern: string, matchType: MatchType, categoryId: number, priority: number) => Promise<void>;
  deleteRule: (id: number) => Promise<void>;
  // Refresh
  refresh: () => Promise<void>;
};

export const useSettings = (): UseSettingsReturn => {
  const [categories, setCategories] = useState<Category[]>([]);
  const [rules, setRules] = useState<Rule[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadCategories = useCallback(async () => {
    try {
      const data = await invoke<Category[]>("get_categories");
      setCategories(data);
    } catch (e) {
      console.error("Failed to load categories:", e);
      throw e;
    }
  }, []);

  const loadRules = useCallback(async () => {
    try {
      const data = await invoke<Rule[]>("get_rules");
      setRules(data);
    } catch (e) {
      console.error("Failed to load rules:", e);
      throw e;
    }
  }, []);

  const refresh = useCallback(async () => {
    setError(null);
    try {
      await Promise.all([loadCategories(), loadRules()]);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }, [loadCategories, loadRules]);

  // Category operations
  const createCategory = useCallback(async (name: string, productivity: ProductivityLevel) => {
    setError(null);
    try {
      await invoke("create_category", { name, productivity });
      await loadCategories();
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
      throw new Error(msg);
    }
  }, [loadCategories]);

  const updateCategory = useCallback(async (id: number, name: string, productivity: ProductivityLevel) => {
    setError(null);
    try {
      await invoke("update_category", { id, name, productivity });
      await loadCategories();
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
      throw new Error(msg);
    }
  }, [loadCategories]);

  const deleteCategory = useCallback(async (id: number) => {
    setError(null);
    try {
      await invoke("delete_category", { id });
      await loadCategories();
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
      throw new Error(msg);
    }
  }, [loadCategories]);

  // Rule operations
  const createRule = useCallback(async (
    pattern: string,
    matchType: MatchType,
    categoryId: number,
    priority: number
  ) => {
    setError(null);
    try {
      await invoke("create_rule", {
        pattern,
        matchType,
        categoryId,
        priority,
      });
      await loadRules();
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
      throw new Error(msg);
    }
  }, [loadRules]);

  const updateRule = useCallback(async (
    id: number,
    pattern: string,
    matchType: MatchType,
    categoryId: number,
    priority: number
  ) => {
    setError(null);
    try {
      await invoke("update_rule", {
        id,
        pattern,
        matchType,
        categoryId,
        priority,
      });
      await loadRules();
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
      throw new Error(msg);
    }
  }, [loadRules]);

  const deleteRule = useCallback(async (id: number) => {
    setError(null);
    try {
      await invoke("delete_rule", { id });
      await loadRules();
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
      throw new Error(msg);
    }
  }, [loadRules]);

  // Initial load
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
    createCategory,
    updateCategory,
    deleteCategory,
    createRule,
    updateRule,
    deleteRule,
    refresh,
  };
};
