import React from "react";

interface ToolButtonProps {
  icon: React.ReactNode;
  label: string;
  active?: boolean;
  onClick: () => void;
  shortcut?: string;
  disabled?: boolean;
}

export function ToolButton({ icon, label, active, onClick, shortcut, disabled }: ToolButtonProps) {
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      title={`${label}${shortcut ? ` (${shortcut})` : ""}`}
      className={`
        p-2 rounded-md transition-colors duration-150
        ${active
          ? "bg-copper text-white"
          : "text-text-secondary hover:bg-bg-elevated hover:text-text-primary"
        }
        ${disabled ? "opacity-40 cursor-not-allowed" : "cursor-pointer"}
      `}
    >
      {icon}
    </button>
  );
}
