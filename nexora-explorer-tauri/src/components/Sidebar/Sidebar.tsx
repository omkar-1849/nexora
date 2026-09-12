import { Home, Briefcase, FolderGit2, Database, Bot, Clock } from "lucide-react";
import { motion } from "framer-motion";
import clsx from "clsx";

interface SidebarProps {
  currentPath: string;
  onNavigate: (path: string) => void;
}

const ROOTS = [
  { label: "Home", path: "/home", icon: Home },
  { label: "Workspace", path: "/workspace", icon: Briefcase },
  { label: "Projects", path: "/projects", icon: FolderGit2 },
  { label: "Data", path: "/data", icon: Database },
  { label: "Models", path: "/models", icon: Bot },
  { label: "Temporary", path: "/tmp", icon: Clock },
];

export function Sidebar({ currentPath, onNavigate }: SidebarProps) {
  return (
    <aside className="w-64 bg-bg-surface border-r border-border-subtle h-full flex flex-col p-4">
      <div className="text-text-tertiary text-xs font-medium uppercase tracking-wider mb-4 px-2">
        Locations
      </div>
      <nav className="flex flex-col gap-1 relative">
        {ROOTS.map((root) => {
          const isActive = currentPath.startsWith(root.path);
          const Icon = root.icon;
          return (
            <button
              key={root.path}
              onClick={() => onNavigate(root.path)}
              className={clsx(
                "relative flex items-center gap-3 px-2 py-2 rounded text-sm text-left transition-colors z-10",
                isActive
                  ? "text-text-primary font-medium"
                  : "text-text-secondary hover:text-text-primary hover:bg-bg-hover"
              )}
            >
              {isActive && (
                <motion.div
                  layoutId="sidebar-active"
                  className="absolute inset-0 bg-bg-selected rounded -z-10"
                  initial={false}
                  transition={{ type: "spring", stiffness: 500, damping: 30 }}
                />
              )}
              <Icon size={18} className={isActive ? "text-accent-focus" : ""} />
              {root.label}
            </button>
          );
        })}
      </nav>
    </aside>
  );
}
