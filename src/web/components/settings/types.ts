import type { Category, MatchType, ProductivityLevel, Rule } from "@/types/api";

interface CategoriesTabProps {
  categories: Category[];
  createCategory: (name: string, productivity: ProductivityLevel) => Promise<void>;
  updateCategory: (id: number, name: string, productivity: ProductivityLevel) => Promise<void>;
  deleteCategory: (id: number) => Promise<void>;
}

interface CategoryFormProps {
  initial?: Category;
  onSave: (name: string, productivity: ProductivityLevel) => Promise<void>;
  onCancel: () => void;
}

interface RulesTabProps {
  rules: Rule[];
  categories: Category[];
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
}

interface RuleFormProps {
  initial?: Rule;
  categories: Category[];
  onSave: (
    pattern: string,
    matchType: MatchType,
    categoryId: number,
    priority: number,
  ) => Promise<void>;
  onCancel: () => void;
}

interface TabButtonProps {
  active: boolean;
  onClick: () => void;
  children: React.ReactNode;
}

interface IconButtonProps {
  onClick: () => void;
  label: string;
  children: React.ReactNode;
}

interface FormActionsProps {
  saving: boolean;
  disabled: boolean;
  onCancel: () => void;
}

interface SettingsViewProps {
  onClose: () => void;
}

export type {
  CategoriesTabProps,
  CategoryFormProps,
  FormActionsProps,
  IconButtonProps,
  RuleFormProps,
  RulesTabProps,
  SettingsViewProps,
  TabButtonProps,
};
