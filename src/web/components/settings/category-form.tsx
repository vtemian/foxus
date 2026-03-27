import { useState } from "react";
import { BUTTON_MUTED_CLASS, INPUT_CLASS } from "@/components/settings/shared";
import type { CategoryFormProps } from "@/components/settings/types";
import type { ProductivityLevel } from "@/types/api";

const ProductivitySelect = ({
  value,
  onChange,
}: {
  value: ProductivityLevel;
  onChange: (value: ProductivityLevel) => void;
}) => {
  const handleChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const numValue = Number(e.target.value);
    if (numValue === 1 || numValue === 0 || numValue === -1) {
      onChange(numValue);
    }
  };

  return (
    <select value={value} onChange={handleChange} className={INPUT_CLASS}>
      <option value={1}>Productive</option>
      <option value={0}>Neutral</option>
      <option value={-1}>Distracting</option>
    </select>
  );
};

const FormActions = ({
  saving,
  disabled,
  onCancel,
}: {
  saving: boolean;
  disabled: boolean;
  onCancel: () => void;
}) => (
  <div className="flex gap-2">
    <button
      type="submit"
      disabled={saving || disabled}
      className="font-mono text-xs text-gray-600 hover:text-gray-800 disabled:text-gray-300 px-2 py-1"
    >
      {saving ? "..." : "Save"}
    </button>
    <button type="button" onClick={onCancel} className={BUTTON_MUTED_CLASS}>
      Cancel
    </button>
  </div>
);

const CategoryForm = ({ initial, onSave, onCancel }: CategoryFormProps) => {
  const [name, setName] = useState(initial?.name ?? "");
  const [productivity, setProductivity] = useState<ProductivityLevel>(initial?.productivity ?? 0);
  const [saving, setSaving] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) return;
    setSaving(true);
    try {
      await onSave(name.trim(), productivity);
    } catch {
      // Error handled by hook
    } finally {
      setSaving(false);
    }
  };

  return (
    <form onSubmit={(e) => void handleSubmit(e)} className="space-y-2">
      <input
        type="text"
        value={name}
        onChange={(e) => setName(e.target.value)}
        placeholder="Category name"
        className={INPUT_CLASS}
        autoFocus
      />
      <ProductivitySelect value={productivity} onChange={setProductivity} />
      <FormActions saving={saving} disabled={!name.trim()} onCancel={onCancel} />
    </form>
  );
};

export { CategoryForm };
