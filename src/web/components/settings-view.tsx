import { useState } from "react";
import { useSettings } from "@/hooks/use-settings";
import { Typography, Card, Badge } from "@/components/ui";
import type { Category, Rule, ProductivityLevel, MatchType } from "@/types/api";
import { productivityToVariant } from "@/types/api";

type Tab = "categories" | "rules";

type SettingsViewProps = {
  onClose: () => void;
};

export const SettingsView = ({ onClose }: SettingsViewProps) => {
  const [activeTab, setActiveTab] = useState<Tab>("categories");
  const settings = useSettings();

  if (settings.isLoading) {
    return (
      <div className="py-8">
        <Typography variant="body" color="muted" className="text-center">
          Loading settings...
        </Typography>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {/* Header with close button */}
      <div className="flex items-center justify-between">
        <Typography variant="h2">Settings</Typography>
        <button
          onClick={onClose}
          className="font-mono text-xs text-gray-400 hover:text-gray-600 px-2 py-1"
          aria-label="Close settings"
        >
          [X]
        </button>
      </div>

      {/* Tab navigation */}
      <div className="flex gap-2 border-b border-gray-250 pb-2">
        <TabButton
          active={activeTab === "categories"}
          onClick={() => setActiveTab("categories")}
        >
          Categories
        </TabButton>
        <TabButton
          active={activeTab === "rules"}
          onClick={() => setActiveTab("rules")}
        >
          Rules
        </TabButton>
      </div>

      {/* Error display */}
      {settings.error && (
        <Typography variant="body" color="distracting" className="text-xs">
          {settings.error}
        </Typography>
      )}

      {/* Tab content */}
      {activeTab === "categories" && (
        <CategoriesTab
          categories={settings.categories}
          createCategory={settings.createCategory}
          updateCategory={settings.updateCategory}
          deleteCategory={settings.deleteCategory}
        />
      )}
      {activeTab === "rules" && (
        <RulesTab
          rules={settings.rules}
          categories={settings.categories}
          createRule={settings.createRule}
          updateRule={settings.updateRule}
          deleteRule={settings.deleteRule}
        />
      )}
    </div>
  );
};

// Tab button component
type TabButtonProps = {
  active: boolean;
  onClick: () => void;
  children: React.ReactNode;
};

const TabButton = ({ active, onClick, children }: TabButtonProps) => (
  <button
    onClick={onClick}
    className={`font-mono text-xs uppercase tracking-wide px-3 py-1 transition-colors ${
      active
        ? "text-gray-600 border-b-2 border-gray-600 -mb-[9px]"
        : "text-gray-400 hover:text-gray-600"
    }`}
  >
    {children}
  </button>
);

// Categories Tab
type CategoriesTabProps = {
  categories: Category[];
  createCategory: (name: string, productivity: ProductivityLevel) => Promise<void>;
  updateCategory: (id: number, name: string, productivity: ProductivityLevel) => Promise<void>;
  deleteCategory: (id: number) => Promise<void>;
};

const CategoriesTab = ({ categories, createCategory, updateCategory, deleteCategory }: CategoriesTabProps) => {
  const [editingId, setEditingId] = useState<number | null>(null);
  const [isAdding, setIsAdding] = useState(false);

  return (
    <div className="space-y-2">
      {/* Add button */}
      {!isAdding && (
        <button
          onClick={() => setIsAdding(true)}
          className="font-mono text-[10px] text-gray-400 hover:text-gray-600 px-2 py-1"
        >
          + Add Category
        </button>
      )}

      {/* Add form */}
      {isAdding && (
        <CategoryForm
          onSave={async (name, productivity) => {
            await createCategory(name, productivity);
            setIsAdding(false);
          }}
          onCancel={() => setIsAdding(false)}
        />
      )}

      {/* Categories list */}
      {categories.map((category) => (
        <Card key={category.id} className="p-2">
          {editingId === category.id ? (
            <CategoryForm
              initial={category}
              onSave={async (name, productivity) => {
                await updateCategory(category.id, name, productivity);
                setEditingId(null);
              }}
              onCancel={() => setEditingId(null)}
            />
          ) : (
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Badge variant={productivityToVariant(category.productivity)} size="sm" />
                <Typography variant="body">{category.name}</Typography>
                <Typography variant="label" color="muted">
                  ({productivityLabel(category.productivity)})
                </Typography>
              </div>
              <div className="flex gap-1">
                <IconButton onClick={() => setEditingId(category.id)} label="Edit">
                  E
                </IconButton>
                <IconButton
                  onClick={() => {
                    if (confirm(`Delete category "${category.name}"?`)) {
                      deleteCategory(category.id);
                    }
                  }}
                  label="Delete"
                >
                  X
                </IconButton>
              </div>
            </div>
          )}
        </Card>
      ))}
    </div>
  );
};

// Category Form
type CategoryFormProps = {
  initial?: Category;
  onSave: (name: string, productivity: ProductivityLevel) => Promise<void>;
  onCancel: () => void;
};

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
    <form onSubmit={handleSubmit} className="space-y-2">
      <input
        type="text"
        value={name}
        onChange={(e) => setName(e.target.value)}
        placeholder="Category name"
        className="w-full font-mono text-xs bg-gray-100 border border-gray-250 px-2 py-1 focus:outline-none focus:border-gray-400"
        autoFocus
      />
      <select
        value={productivity}
        onChange={(e) => setProductivity(Number(e.target.value) as ProductivityLevel)}
        className="w-full font-mono text-xs bg-gray-100 border border-gray-250 px-2 py-1 focus:outline-none focus:border-gray-400"
      >
        <option value={1}>Productive</option>
        <option value={0}>Neutral</option>
        <option value={-1}>Distracting</option>
      </select>
      <div className="flex gap-2">
        <button
          type="submit"
          disabled={saving || !name.trim()}
          className="font-mono text-xs text-gray-600 hover:text-gray-800 disabled:text-gray-300 px-2 py-1"
        >
          {saving ? "..." : "Save"}
        </button>
        <button
          type="button"
          onClick={onCancel}
          className="font-mono text-xs text-gray-400 hover:text-gray-600 px-2 py-1"
        >
          Cancel
        </button>
      </div>
    </form>
  );
};

// Rules Tab
type RulesTabProps = {
  rules: Rule[];
  categories: Category[];
  createRule: (pattern: string, matchType: MatchType, categoryId: number, priority: number) => Promise<void>;
  updateRule: (id: number, pattern: string, matchType: MatchType, categoryId: number, priority: number) => Promise<void>;
  deleteRule: (id: number) => Promise<void>;
};

const RulesTab = ({ rules, categories, createRule, updateRule, deleteRule }: RulesTabProps) => {
  const [editingId, setEditingId] = useState<number | null>(null);
  const [isAdding, setIsAdding] = useState(false);

  const getCategoryName = (categoryId: number) => {
    return categories.find((c) => c.id === categoryId)?.name ?? "Unknown";
  };

  return (
    <div className="space-y-2">
      {/* Add button */}
      {!isAdding && (
        <button
          onClick={() => setIsAdding(true)}
          className="font-mono text-[10px] text-gray-400 hover:text-gray-600 px-2 py-1"
        >
          + Add Rule
        </button>
      )}

      {/* Add form */}
      {isAdding && (
        <RuleForm
          categories={categories}
          onSave={async (pattern, matchType, categoryId, priority) => {
            await createRule(pattern, matchType, categoryId, priority);
            setIsAdding(false);
          }}
          onCancel={() => setIsAdding(false)}
        />
      )}

      {/* Rules list */}
      {rules.map((rule) => (
        <Card key={rule.id} className="p-2">
          {editingId === rule.id ? (
            <RuleForm
              initial={rule}
              categories={categories}
              onSave={async (pattern, matchType, categoryId, priority) => {
                await updateRule(rule.id, pattern, matchType, categoryId, priority);
                setEditingId(null);
              }}
              onCancel={() => setEditingId(null)}
            />
          ) : (
            <div className="flex items-center justify-between">
              <div className="flex flex-col gap-0.5">
                <div className="flex items-center gap-2">
                  <Typography variant="body" className="font-medium">
                    {rule.pattern}
                  </Typography>
                  <Typography variant="label" color="muted">
                    [{rule.match_type}]
                  </Typography>
                </div>
                <Typography variant="label" color="secondary">
                  {getCategoryName(rule.category_id)} | Priority: {rule.priority}
                </Typography>
              </div>
              <div className="flex gap-1">
                <IconButton onClick={() => setEditingId(rule.id)} label="Edit">
                  E
                </IconButton>
                <IconButton
                  onClick={() => {
                    if (confirm(`Delete rule "${rule.pattern}"?`)) {
                      deleteRule(rule.id);
                    }
                  }}
                  label="Delete"
                >
                  X
                </IconButton>
              </div>
            </div>
          )}
        </Card>
      ))}
    </div>
  );
};

// Rule Form
type RuleFormProps = {
  initial?: Rule;
  categories: Category[];
  onSave: (pattern: string, matchType: MatchType, categoryId: number, priority: number) => Promise<void>;
  onCancel: () => void;
};

const RuleForm = ({ initial, categories, onSave, onCancel }: RuleFormProps) => {
  const [pattern, setPattern] = useState(initial?.pattern ?? "");
  const [matchType, setMatchType] = useState<MatchType>(initial?.match_type ?? "app");
  const [categoryId, setCategoryId] = useState<number>(initial?.category_id ?? categories[0]?.id ?? 0);
  const [priority, setPriority] = useState(initial?.priority ?? 10);
  const [saving, setSaving] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!pattern.trim() || !categoryId) return;
    setSaving(true);
    try {
      await onSave(pattern.trim(), matchType, categoryId, priority);
    } catch {
      // Error handled by hook
    } finally {
      setSaving(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-2">
      <input
        type="text"
        value={pattern}
        onChange={(e) => setPattern(e.target.value)}
        placeholder="Pattern (e.g., youtube.com)"
        className="w-full font-mono text-xs bg-gray-100 border border-gray-250 px-2 py-1 focus:outline-none focus:border-gray-400"
        autoFocus
      />
      <div className="flex gap-2">
        <select
          value={matchType}
          onChange={(e) => setMatchType(e.target.value as MatchType)}
          className="flex-1 font-mono text-xs bg-gray-100 border border-gray-250 px-2 py-1 focus:outline-none focus:border-gray-400"
        >
          <option value="app">App</option>
          <option value="domain">Domain</option>
          <option value="title">Title</option>
        </select>
        <input
          type="number"
          value={priority}
          onChange={(e) => setPriority(Number(e.target.value))}
          placeholder="Priority"
          min={0}
          max={100}
          className="w-16 font-mono text-xs bg-gray-100 border border-gray-250 px-2 py-1 focus:outline-none focus:border-gray-400"
        />
      </div>
      <select
        value={categoryId}
        onChange={(e) => setCategoryId(Number(e.target.value))}
        className="w-full font-mono text-xs bg-gray-100 border border-gray-250 px-2 py-1 focus:outline-none focus:border-gray-400"
      >
        {categories.map((cat) => (
          <option key={cat.id} value={cat.id}>
            {cat.name}
          </option>
        ))}
      </select>
      <div className="flex gap-2">
        <button
          type="submit"
          disabled={saving || !pattern.trim() || !categoryId}
          className="font-mono text-xs text-gray-600 hover:text-gray-800 disabled:text-gray-300 px-2 py-1"
        >
          {saving ? "..." : "Save"}
        </button>
        <button
          type="button"
          onClick={onCancel}
          className="font-mono text-xs text-gray-400 hover:text-gray-600 px-2 py-1"
        >
          Cancel
        </button>
      </div>
    </form>
  );
};

// Small icon button
type IconButtonProps = {
  onClick: () => void;
  label: string;
  children: React.ReactNode;
};

const IconButton = ({ onClick, label, children }: IconButtonProps) => (
  <button
    onClick={onClick}
    aria-label={label}
    className="font-mono text-[10px] text-gray-400 hover:text-gray-600 w-5 h-5 flex items-center justify-center"
  >
    {children}
  </button>
);

// Helper to show productivity level as text
const productivityLabel = (p: ProductivityLevel): string => {
  if (p > 0) return "productive";
  if (p < 0) return "distracting";
  return "neutral";
};
