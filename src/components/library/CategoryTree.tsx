import React, { useState } from "react";
import { Folder, ChevronRight, ChevronDown } from "lucide-react";
import { Category } from "../../types/library";

interface CategoryTreeProps {
  categories: Category[];
  selected: string | null;
  onSelect: (categoryId: string | null) => void;
}

function hasSubcategories(categories: Category[], parentId: string): boolean {
  return categories.some((c) => c.parent_id === parentId);
}

function TreeNode({
  category,
  categories,
  selected,
  onSelect,
  depth,
}: {
  category: Category;
  categories: Category[];
  selected: string | null;
  onSelect: (categoryId: string | null) => void;
  depth: number;
}) {
  const [expanded, setExpanded] = useState(false);
  const children = categories.filter((c) => c.parent_id === category.id);
  const hasChildren = children.length > 0;

  return (
    <>
      <button
        onClick={() => {
          if (hasChildren) setExpanded((prev) => !prev);
          onSelect(category.id);
        }}
        className={`w-full flex items-center gap-1 text-left px-2 py-1 text-xs rounded transition-colors ${
          selected === category.id
            ? "bg-copper text-white"
            : "text-text-secondary hover:bg-bg-elevated hover:text-text-primary"
        }`}
        style={{ paddingLeft: `${12 + depth * 16}px` }}
      >
        {hasChildren ? (
          expanded ? (
            <ChevronDown size={12} className="shrink-0" />
          ) : (
            <ChevronRight size={12} className="shrink-0" />
          )
        ) : (
          <span className="w-3 shrink-0" />
        )}
        <Folder size={12} className="shrink-0" />
        <span className="truncate">{category.name}</span>
      </button>
      {expanded &&
        children.map((child) => (
          <TreeNode
            key={child.id}
            category={child}
            categories={categories}
            selected={selected}
            onSelect={onSelect}
            depth={depth + 1}
          />
        ))}
    </>
  );
}

export function CategoryTree({ categories, selected, onSelect }: CategoryTreeProps) {
  const roots = categories.filter((c) => !c.parent_id);

  return (
    <div className="space-y-0.5">
      {roots.map((cat) => (
        <TreeNode
          key={cat.id}
          category={cat}
          categories={categories}
          selected={selected}
          onSelect={onSelect}
          depth={0}
        />
      ))}
    </div>
  );
}
