import { useState } from "react";
import { RuleForm } from "@/components/settings/rule-form";
import { IconButton } from "@/components/settings/shared";
import type { RulesTabProps } from "@/components/settings/types";
import { Card, Typography } from "@/components/ui";
import type { Category, MatchType, Rule } from "@/types/api";

const ADD_BUTTON_CLASS = "font-mono text-[10px] text-gray-400 hover:text-gray-600 px-2 py-1";

const RuleRow = ({
  pattern,
  matchType,
  categoryName,
  priority,
  onEdit,
  onDelete,
}: {
  pattern: string;
  matchType: string;
  categoryName: string;
  priority: number;
  onEdit: () => void;
  onDelete: () => void;
}) => (
  <div className="flex items-center justify-between">
    <div className="flex flex-col gap-0.5">
      <div className="flex items-center gap-2">
        <Typography variant="body" className="font-medium">
          {pattern}
        </Typography>
        <Typography variant="label" color="muted">
          [{matchType}]
        </Typography>
      </div>
      <Typography variant="label" color="secondary">
        {categoryName} | Priority: {priority}
      </Typography>
    </div>
    <div className="flex gap-1">
      <IconButton onClick={onEdit} label="Edit">
        E
      </IconButton>
      <IconButton onClick={onDelete} label="Delete">
        X
      </IconButton>
    </div>
  </div>
);

const RuleItem = ({
  rule,
  categories,
  isEditing,
  onSave,
  onCancelEdit,
  onEdit,
  onDelete,
}: {
  rule: Rule;
  categories: Category[];
  isEditing: boolean;
  onSave: (
    pattern: string,
    matchType: MatchType,
    categoryId: number,
    priority: number,
  ) => Promise<void>;
  onCancelEdit: () => void;
  onEdit: () => void;
  onDelete: () => void;
}) => (
  <Card className="p-2">
    {isEditing ? (
      <RuleForm initial={rule} categories={categories} onSave={onSave} onCancel={onCancelEdit} />
    ) : (
      <RuleRow
        pattern={rule.pattern}
        matchType={rule.match_type}
        categoryName={categories.find((c) => c.id === rule.category_id)?.name ?? "Unknown"}
        priority={rule.priority}
        onEdit={onEdit}
        onDelete={onDelete}
      />
    )}
  </Card>
);

const AddRuleSection = ({
  isAdding,
  categories,
  onAdd,
  onSave,
  onCancel,
}: {
  isAdding: boolean;
  categories: Category[];
  onAdd: () => void;
  onSave: (
    pattern: string,
    matchType: MatchType,
    categoryId: number,
    priority: number,
  ) => Promise<void>;
  onCancel: () => void;
}) => (
  <>
    {!isAdding && (
      <button type="button" onClick={onAdd} className={ADD_BUTTON_CLASS}>
        + Add Rule
      </button>
    )}
    {isAdding && <RuleForm categories={categories} onSave={onSave} onCancel={onCancel} />}
  </>
);

const RulesTab = ({ rules, categories, createRule, updateRule, deleteRule }: RulesTabProps) => {
  const [editingId, setEditingId] = useState<number | null>(null);
  const [isAdding, setIsAdding] = useState(false);

  return (
    <div className="space-y-2">
      <AddRuleSection
        isAdding={isAdding}
        categories={categories}
        onAdd={() => setIsAdding(true)}
        onSave={async (pattern, matchType, categoryId, priority) => {
          await createRule(pattern, matchType, categoryId, priority);
          setIsAdding(false);
        }}
        onCancel={() => setIsAdding(false)}
      />
      {rules.map((rule) => (
        <RuleItem
          key={rule.id}
          rule={rule}
          categories={categories}
          isEditing={editingId === rule.id}
          onSave={async (pattern, matchType, categoryId, priority) => {
            await updateRule(rule.id, pattern, matchType, categoryId, priority);
            setEditingId(null);
          }}
          onCancelEdit={() => setEditingId(null)}
          onEdit={() => setEditingId(rule.id)}
          onDelete={() => {
            if (confirm(`Delete rule "${rule.pattern}"?`)) {
              deleteRule(rule.id);
            }
          }}
        />
      ))}
    </div>
  );
};

export { RulesTab };
