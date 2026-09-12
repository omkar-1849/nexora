import { Search as SearchIcon, X } from "lucide-react";
import { useEffect, useRef } from "react";

interface SearchProps {
  query: string;
  onQueryChange: (q: string) => void;
}

export function Search({ query, onQueryChange }: SearchProps) {
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "f") {
        e.preventDefault();
        inputRef.current?.focus();
      } else if (e.key === "Escape") {
        if (query) {
          onQueryChange("");
        } else {
          inputRef.current?.blur();
        }
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [query, onQueryChange]);

  return (
    <div className="relative flex items-center bg-bg-base border border-border-subtle rounded px-2 py-1 focus-within:border-accent-focus focus-within:ring-1 focus-within:ring-accent-focus transition-all w-48">
      <SearchIcon size={14} className="text-text-tertiary mr-2 flex-shrink-0" />
      <input
        ref={inputRef}
        type="text"
        value={query}
        onChange={(e) => onQueryChange(e.target.value)}
        placeholder="Search..."
        aria-label="Search"
        className="bg-transparent border-none outline-none text-sm text-text-primary w-full placeholder-text-tertiary"
      />
      {query && (
        <button
          onClick={() => onQueryChange("")}
          className="text-text-tertiary hover:text-text-primary p-0.5 rounded transition-colors"
          aria-label="Clear search"
        >
          <X size={14} />
        </button>
      )}
    </div>
  );
}
