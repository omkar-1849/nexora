import { ArrowLeft, ArrowRight, ArrowUp, LayoutGrid, List } from "lucide-react";
import { Breadcrumbs } from "../Breadcrumbs/Breadcrumbs";
import { Search } from "./Search";
import clsx from "clsx";

interface ToolbarProps {
  currentPath: string;
  onNavigate: (path: string) => void;
  onBack: () => void;
  onForward: () => void;
  onUp: () => void;
  canGoBack: boolean;
  canGoForward: boolean;
  viewMode: "grid" | "list";
  onViewModeChange: (mode: "grid" | "list") => void;
  searchQuery: string;
  onSearchChange: (q: string) => void;
}

export function Toolbar({
  currentPath,
  onNavigate,
  onBack,
  onForward,
  onUp,
  canGoBack,
  canGoForward,
  viewMode,
  onViewModeChange,
  searchQuery,
  onSearchChange,
}: ToolbarProps) {
  const canGoUp = currentPath !== "/";

  return (
    <div className="h-14 border-b border-border-subtle bg-bg-surface flex items-center px-4 gap-4 flex-shrink-0">
      <div className="flex items-center gap-1">
        <IconButton onClick={onBack} disabled={!canGoBack} icon={ArrowLeft} title="Back" />
        <IconButton onClick={onForward} disabled={!canGoForward} icon={ArrowRight} title="Forward" />
        <IconButton onClick={onUp} disabled={!canGoUp} icon={ArrowUp} title="Up" />
      </div>

      <div className="flex-1 min-w-0">
        <Breadcrumbs path={currentPath} onNavigate={onNavigate} />
      </div>

      <Search query={searchQuery} onQueryChange={onSearchChange} />

      <div className="flex items-center gap-1 bg-bg-base rounded p-1 border border-border-subtle">
        <button
          onClick={() => onViewModeChange("grid")}
          className={clsx(
            "p-1.5 rounded transition-colors",
            viewMode === "grid" ? "bg-bg-selected text-text-primary shadow-sm" : "text-text-secondary hover:text-text-primary"
          )}
          title="Grid View"
          aria-label="Grid View"
        >
          <LayoutGrid size={16} />
        </button>
        <button
          onClick={() => onViewModeChange("list")}
          className={clsx(
            "p-1.5 rounded transition-colors",
            viewMode === "list" ? "bg-bg-selected text-text-primary shadow-sm" : "text-text-secondary hover:text-text-primary"
          )}
          title="List View"
          aria-label="List View"
        >
          <List size={16} />
        </button>
      </div>
    </div>
  );
}

function IconButton({ onClick, disabled, icon: Icon, title }: { onClick: () => void; disabled?: boolean; icon: React.ElementType; title?: string }) {
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      title={title}
      aria-label={title}
      className={clsx(
        "p-1.5 rounded transition-colors",
        disabled
          ? "text-text-tertiary cursor-not-allowed opacity-50"
          : "text-text-secondary hover:bg-bg-hover hover:text-text-primary"
      )}
    >
      <Icon size={18} />
    </button>
  );
}
