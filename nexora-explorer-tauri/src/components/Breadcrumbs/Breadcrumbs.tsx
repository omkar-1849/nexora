import { ChevronRight } from "lucide-react";
import React from "react";

interface BreadcrumbsProps {
  path: string;
  onNavigate: (path: string) => void;
}

export function Breadcrumbs({ path, onNavigate }: BreadcrumbsProps) {
  // Strip trailing slash except for root itself
  const normalized = path.endsWith("/") && path.length > 1 ? path.slice(0, -1) : path;
  const segments = normalized.split("/").filter(Boolean);

  let currentBuiltPath = "";

  return (
    <div className="flex items-center gap-1 text-sm font-mono text-text-secondary overflow-hidden">
      <button
        onClick={() => onNavigate("/")}
        className="hover:text-text-primary transition-colors px-1 rounded hover:bg-bg-hover"
      >
        /
      </button>
      {segments.map((segment, idx) => {
        currentBuiltPath += "/" + segment;
        const isLast = idx === segments.length - 1;
        const thisPath = currentBuiltPath;
        
        return (
          <React.Fragment key={thisPath}>
            <ChevronRight size={14} className="text-text-tertiary flex-shrink-0" />
            <button
              onClick={() => onNavigate(thisPath)}
              className={"px-1 rounded transition-colors truncate " + (isLast ? "text-text-primary font-medium" : "hover:text-text-primary hover:bg-bg-hover")}
            >
              {segment}
            </button>
          </React.Fragment>
        );
      })}
    </div>
  );
}
