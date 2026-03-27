import { useState } from "react";
import { CategoryForm } from "@/components/settings/category-form";
import { ADD_BUTTON_CLASS, IconButton, productivityLabel } from "@/components/settings/shared";
import type { CategoriesTabProps } from "@/components/settings/types";
import { Badge, Card, Typography } from "@/components/ui";
import type { Category, ProductivityLevel } from "@/types/api";
import { productivityToVariant } from "@/types/api";

const CategoryRow = ({
  name,
  productivity,
  onEdit,
  onDelete,
}: {
  name: string;
  productivity: ProductivityLevel;
  onEdit: () => void;
  onDelete: () => void;
}) => (
  <div className="flex items-center justify-between">
    <div className="flex items-center gap-2">
      <Badge variant={productivityToVariant(productivity)} size="sm" />
      <Typography variant="body">{name}</Typography>
      <Typography variant="label" color="muted">
        ({productivityLabel(productivity)})
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

const CategoryItem = ({
  category,
  isEditing,
  onSave,
  onCancelEdit,
  onEdit,
  onDelete,
}: {
  category: Category;
  isEditing: boolean;
  onSave: (name: string, productivity: ProductivityLevel) => Promise<void>;
  onCancelEdit: () => void;
  onEdit: () => void;
  onDelete: () => void;
}) => (
  <Card className="p-2">
    {isEditing ? (
      <CategoryForm initial={category} onSave={onSave} onCancel={onCancelEdit} />
    ) : (
      <CategoryRow
        name={category.name}
        productivity={category.productivity}
        onEdit={onEdit}
        onDelete={onDelete}
      />
    )}
  </Card>
);

const AddCategorySection = ({
  isAdding,
  onAdd,
  onSave,
  onCancel,
}: {
  isAdding: boolean;
  onAdd: () => void;
  onSave: (name: string, productivity: ProductivityLevel) => Promise<void>;
  onCancel: () => void;
}) => (
  <>
    {!isAdding && (
      <button type="button" onClick={onAdd} className={ADD_BUTTON_CLASS}>
        + Add Category
      </button>
    )}
    {isAdding && <CategoryForm onSave={onSave} onCancel={onCancel} />}
  </>
);

const CategoriesTab = ({
  categories,
  createCategory,
  updateCategory,
  deleteCategory,
}: CategoriesTabProps) => {
  const [editingId, setEditingId] = useState<number | null>(null);
  const [isAdding, setIsAdding] = useState(false);

  return (
    <div className="space-y-2">
      <AddCategorySection
        isAdding={isAdding}
        onAdd={() => setIsAdding(true)}
        onSave={async (name, productivity) => {
          await createCategory(name, productivity);
          setIsAdding(false);
        }}
        onCancel={() => setIsAdding(false)}
      />
      {categories.map((category) => (
        <CategoryItem
          key={category.id}
          category={category}
          isEditing={editingId === category.id}
          onSave={async (name, productivity) => {
            await updateCategory(category.id, name, productivity);
            setEditingId(null);
          }}
          onCancelEdit={() => setEditingId(null)}
          onEdit={() => setEditingId(category.id)}
          onDelete={() => {
            if (confirm(`Delete category "${category.name}"?`)) {
              deleteCategory(category.id);
            }
          }}
        />
      ))}
    </div>
  );
};

export { CategoriesTab };
