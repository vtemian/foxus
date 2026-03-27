import { useState } from "react";
import { FormActions, INPUT_CLASS } from "@/components/settings/shared";
import type { RuleFormProps } from "@/components/settings/types";
import type { Category, MatchType } from "@/types/api";

const DEFAULT_PRIORITY = 10;
const MAX_PRIORITY = 100;

const PRIORITY_INPUT_CLASS =
  "w-16 font-mono text-xs bg-gray-100 border border-gray-250 px-2 py-1 focus:outline-none focus:border-gray-400";

const SELECT_CLASS =
  "flex-1 font-mono text-xs bg-gray-100 border border-gray-250 px-2 py-1 focus:outline-none focus:border-gray-400";

const MatchTypeSelect = ({
  value,
  onChange,
}: {
  value: MatchType;
  onChange: (value: MatchType) => void;
}) => {
  const handleChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const newValue = e.target.value;
    if (newValue === "app" || newValue === "domain" || newValue === "title") {
      onChange(newValue);
    }
  };

  return (
    <select value={value} onChange={handleChange} className={SELECT_CLASS}>
      <option value="app">App</option>
      <option value="domain">Domain</option>
      <option value="title">Title</option>
    </select>
  );
};

const CategorySelect = ({
  categories,
  value,
  onChange,
}: {
  categories: Category[];
  value: number;
  onChange: (value: number) => void;
}) => (
  <select value={value} onChange={(e) => onChange(Number(e.target.value))} className={INPUT_CLASS}>
    {categories.map((cat) => (
      <option key={cat.id} value={cat.id}>
        {cat.name}
      </option>
    ))}
  </select>
);

const useRuleFormState = (initial: RuleFormProps["initial"], categories: Category[]) => {
  const [pattern, setPattern] = useState(initial?.pattern ?? "");
  const [matchType, setMatchType] = useState<MatchType>(initial?.match_type ?? "app");
  const [categoryId, setCategoryId] = useState<number>(
    initial?.category_id ?? categories[0]?.id ?? 0,
  );
  const [priority, setPriority] = useState(initial?.priority ?? DEFAULT_PRIORITY);
  const [saving, setSaving] = useState(false);

  return {
    pattern,
    setPattern,
    matchType,
    setMatchType,
    categoryId,
    setCategoryId,
    priority,
    setPriority,
    saving,
    setSaving,
  };
};

const RuleFormFields = ({
  state,
  categories,
}: {
  state: ReturnType<typeof useRuleFormState>;
  categories: Category[];
}) => (
  <>
    <input
      type="text"
      value={state.pattern}
      onChange={(e) => state.setPattern(e.target.value)}
      placeholder="Pattern (e.g., youtube.com)"
      className={INPUT_CLASS}
      autoFocus
    />
    <div className="flex gap-2">
      <MatchTypeSelect value={state.matchType} onChange={state.setMatchType} />
      <input
        type="number"
        value={state.priority}
        onChange={(e) => state.setPriority(Number(e.target.value))}
        placeholder="Priority"
        min={0}
        max={MAX_PRIORITY}
        className={PRIORITY_INPUT_CLASS}
      />
    </div>
    <CategorySelect
      categories={categories}
      value={state.categoryId}
      onChange={state.setCategoryId}
    />
  </>
);

const RuleForm = ({ initial, categories, onSave, onCancel }: RuleFormProps) => {
  const state = useRuleFormState(initial, categories);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!state.pattern.trim() || !state.categoryId) return;
    state.setSaving(true);
    try {
      await onSave(state.pattern.trim(), state.matchType, state.categoryId, state.priority);
    } catch {
      // Error handled by hook
    } finally {
      state.setSaving(false);
    }
  };

  return (
    <form onSubmit={(e) => void handleSubmit(e)} className="space-y-2">
      <RuleFormFields state={state} categories={categories} />
      <FormActions
        saving={state.saving}
        disabled={!state.pattern.trim() || !state.categoryId}
        onCancel={onCancel}
      />
    </form>
  );
};

export { RuleForm };
