import { useState } from "react";
import { DirectoryListingDto } from "../../types";
import { FileNodeIcon } from "./FileNodeIcon";
import { motion, AnimatePresence } from "framer-motion";
import clsx from "clsx";

interface PlaygroundProps {
  currentPath: string;
  onNavigate: (path: string) => void;
  viewMode: "grid" | "list";
  selectedPaths: Set<string>;
  onSelect: (paths: Set<string>) => void;
  listing: DirectoryListingDto | null;
  isLoading: boolean;
  error: string | null;
  searchQuery: string;
}

export function Playground({ currentPath, onNavigate, viewMode, selectedPaths, onSelect, listing, isLoading, error, searchQuery }: PlaygroundProps) {
  const [sortCol, setSortCol] = useState<"name" | "type" | "size">("name");
  const [sortDesc, setSortDesc] = useState(false);

  const toggleSort = (col: "name" | "type" | "size") => {
    if (sortCol === col) setSortDesc(!sortDesc);
    else {
      setSortCol(col);
      setSortDesc(false);
    }
  };

  const sortedEntries = listing?.entries ? [...listing.entries].filter(e => e.name.toLowerCase().includes(searchQuery.toLowerCase())).sort((a, b) => {
    let cmp = 0;
    if (sortCol === "name") cmp = a.name.localeCompare(b.name);
    else if (sortCol === "size") cmp = (a.sizeBytes || 0) - (b.sizeBytes || 0);
    else if (sortCol === "type") cmp = a.kind.localeCompare(b.kind) || (a.fileType || "").localeCompare(b.fileType || "");
    return sortDesc ? -cmp : cmp;
  }) : [];

  if (error) {
    return (
      <div className="flex-1 overflow-auto bg-bg-base p-6 text-text-primary">
        <div className="bg-bg-surface border border-border-subtle p-4 rounded text-sm">
          <p className="text-type-rust font-medium mb-1">Error loading directory</p>
          <p className="text-text-secondary">{error}</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-auto bg-bg-base p-6 relative">
      <AnimatePresence mode="wait">
        {isLoading && (!listing || listing.path !== currentPath) ? (
          <motion.div
            key="loading"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.15 }}
            className="animate-pulse flex gap-4 flex-wrap"
          >
            {[1, 2, 3, 4, 5].map((i) => (
              <div key={i} className="w-32 h-24 bg-bg-surface rounded border border-border-subtle" />
            ))}
          </motion.div>
        ) : sortedEntries.length === 0 ? (
          <motion.div
            key="empty"
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.2 }}
            className="flex items-center justify-center h-full text-text-tertiary"
          >
            This folder is empty
          </motion.div>
        ) : viewMode === "grid" ? (
          <motion.div
            key="grid"
            initial={{ opacity: 0, scale: 0.98 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.2, ease: [0.16, 1, 0.3, 1] }}
            className="grid grid-cols-[repeat(auto-fill,minmax(120px,1fr))] gap-4 outline-none"
            tabIndex={0}
          >
            {sortedEntries.map((entry) => {
              const isSelected = selectedPaths.has(entry.virtualPath);
              return (
                <button
                  key={entry.virtualPath}
                  onClick={(e) => {
                    const newSet = e.ctrlKey || e.metaKey ? new Set(selectedPaths) : new Set<string>();
                    if (newSet.has(entry.virtualPath)) newSet.delete(entry.virtualPath);
                    else newSet.add(entry.virtualPath);
                    onSelect(newSet);
                  }}
                  onDoubleClick={() => {
                    if (entry.kind === "directory") onNavigate(entry.virtualPath);
                  }}
                  className={clsx(
                    "flex flex-col items-center justify-center p-3 rounded-lg border text-center focus:outline-none focus:ring-2 focus:ring-accent-focus relative group",
                    isSelected
                      ? "border-accent-focus"
                      : "border-transparent hover:border-border-subtle"
                  )}
                >
                  <motion.div
                    className="absolute inset-0 rounded-lg -z-10"
                    initial={false}
                    animate={{
                      backgroundColor: isSelected ? "var(--color-bg-selected)" : "transparent",
                    }}
                    whileHover={!isSelected ? { backgroundColor: "var(--color-bg-hover)" } : undefined}
                    transition={{ duration: 0.15 }}
                  />
                  <FileNodeIcon entry={entry} className="mb-2 w-10 h-10" />
                  <span className="text-xs truncate w-full text-text-primary" title={entry.name}>
                    {entry.name}
                  </span>
                </button>
              );
            })}
          </motion.div>
        ) : (
          <motion.div
            key="list"
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.2, ease: [0.16, 1, 0.3, 1] }}
            className="flex flex-col outline-none"
            tabIndex={0}
          >
            <div className="grid grid-cols-[2fr_1fr_1fr_1fr] px-2 py-1 text-xs text-text-secondary border-b border-border-subtle mb-1">
              <button className="text-left font-medium hover:text-text-primary" onClick={() => toggleSort("name")}>Name {sortCol === "name" && (sortDesc ? "↓" : "↑")}</button>
              <button className="text-left font-medium hover:text-text-primary" onClick={() => toggleSort("type")}>Type {sortCol === "type" && (sortDesc ? "↓" : "↑")}</button>
              <button className="text-left font-medium hover:text-text-primary" onClick={() => toggleSort("size")}>Size {sortCol === "size" && (sortDesc ? "↓" : "↑")}</button>
              <div className="text-left font-medium">Modified</div>
            </div>
            {sortedEntries.map((entry) => {
              const isSelected = selectedPaths.has(entry.virtualPath);
              return (
                <button
                  key={entry.virtualPath}
                  onClick={(e) => {
                    const newSet = e.ctrlKey || e.metaKey ? new Set(selectedPaths) : new Set<string>();
                    if (newSet.has(entry.virtualPath)) newSet.delete(entry.virtualPath);
                    else newSet.add(entry.virtualPath);
                    onSelect(newSet);
                  }}
                  onDoubleClick={() => {
                    if (entry.kind === "directory") onNavigate(entry.virtualPath);
                  }}
                  className={clsx(
                    "grid grid-cols-[2fr_1fr_1fr_1fr] items-center px-2 h-9 rounded text-sm text-left focus:outline-none focus:ring-1 focus:ring-inset focus:ring-accent-focus relative group",
                    isSelected
                      ? "text-text-primary"
                      : "text-text-secondary hover:text-text-primary"
                  )}
                >
                  <motion.div
                    className="absolute inset-0 rounded -z-10"
                    initial={false}
                    animate={{
                      backgroundColor: isSelected ? "var(--color-bg-selected)" : "transparent",
                    }}
                    whileHover={!isSelected ? { backgroundColor: "var(--color-bg-hover)" } : undefined}
                    transition={{ duration: 0.15 }}
                  />
                  <div className="flex items-center gap-2 truncate pr-2">
                    <FileNodeIcon entry={entry} className="w-4 h-4 flex-shrink-0" />
                    <span className="truncate" title={entry.name}>{entry.name}</span>
                  </div>
                  <div className="truncate pr-2">{entry.kind === "directory" ? "Folder" : entry.fileType || "File"}</div>
                  <div className="truncate pr-2">{entry.sizeBytes !== undefined ? (entry.sizeBytes + " B") : "--"}</div>
                  <div className="truncate">{entry.modifiedAt || "--"}</div>
                </button>
              );
            })}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
