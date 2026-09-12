import { useState, useCallback, useEffect } from "react";
import { AnimatePresence } from "framer-motion";
import { Sidebar } from "./components/Sidebar/Sidebar";
import { Toolbar } from "./components/Toolbar/Toolbar";
import { Playground } from "./components/Playground/Playground";
import { Inspector } from "./components/Inspector/Inspector";
import { listDirectory } from "./api/fs";
import { DirectoryListingDto } from "./types";
import "./styles.css";

export default function App() {
  const [history, setHistory] = useState<string[]>(["/workspace"]);
  const [historyIndex, setHistoryIndex] = useState(0);
  const [viewMode, setViewMode] = useState<"grid" | "list">("grid");

  const currentPath = history[historyIndex];
  const [searchQuery, setSearchQuery] = useState("");

  const [listing, setListing] = useState<DirectoryListingDto | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [selectedPaths, setSelectedPaths] = useState<Set<string>>(new Set());

  const navigate = useCallback(
    (path: string) => {
      if (path === currentPath) return;
      const newHistory = history.slice(0, historyIndex + 1);
      newHistory.push(path);
      setHistory(newHistory);
      setHistoryIndex(newHistory.length - 1);
    },
    [currentPath, history, historyIndex]
  );
  useEffect(() => {
    let active = true;
    const fetchDir = async () => {
      setIsLoading(true);
      setError(null);
      try {
        const result = await listDirectory(currentPath);
        if (active) {
          setListing(result);
          setSelectedPaths(new Set());
        }
      } catch (err: any) {
        if (active) {
          setError(err.message || String(err));
          setListing(null);
          setSelectedPaths(new Set());
        }
      } finally {
        if (active) setIsLoading(false);
      }
    };
    fetchDir();
    return () => {
      active = false;
    };
  }, [currentPath]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Global navigation shortcuts (if not inside an input)
      if (e.target instanceof HTMLInputElement) {
        if (e.key === "Escape") {
          // Handled by Search component
          return;
        }
      }

      if (e.key === "Backspace" || (e.altKey && e.key === "ArrowLeft")) {
        e.preventDefault();
        if (historyIndex > 0) setHistoryIndex(h => h - 1);
      } else if (e.altKey && e.key === "ArrowRight") {
        e.preventDefault();
        if (historyIndex < history.length - 1) setHistoryIndex(h => h + 1);
      } else if (e.altKey && e.key === "ArrowUp") {
        e.preventDefault();
        if (currentPath !== "/") {
          const segments = currentPath.endsWith("/") ? currentPath.slice(0, -1).split("/") : currentPath.split("/");
          segments.pop();
          const parentPath = segments.length === 1 && segments[0] === "" ? "/" : segments.join("/");
          navigate(parentPath || "/");
        }
      } else if ((e.ctrlKey || e.metaKey) && e.key === "a") {
        if (!(e.target instanceof HTMLInputElement)) {
          e.preventDefault();
          if (listing) {
            setSelectedPaths(new Set(listing.entries.map(e => e.virtualPath)));
          }
        }
      } else if (e.key === "Escape") {
        if (searchQuery) {
          setSearchQuery("");
        } else if (selectedPaths.size > 0) {
          setSelectedPaths(new Set());
        }
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [currentPath, history, historyIndex, listing, navigate, searchQuery, selectedPaths]);


  const handleBack = () => {
    if (historyIndex > 0) setHistoryIndex(historyIndex - 1);
  };

  const handleForward = () => {
    if (historyIndex < history.length - 1) setHistoryIndex(historyIndex + 1);
  };

  const handleUp = () => {
    if (currentPath === "/") return;
    const segments = currentPath.endsWith("/") ? currentPath.slice(0, -1).split("/") : currentPath.split("/");
    segments.pop();
    const parentPath = segments.length === 1 && segments[0] === "" ? "/" : segments.join("/");
    navigate(parentPath || "/");
  };

  const selectedEntries = listing?.entries.filter((e) => selectedPaths.has(e.virtualPath)) || [];

  return (
    <div className="flex h-screen w-full bg-bg-base text-text-primary overflow-hidden font-sans">
      <Sidebar currentPath={currentPath} onNavigate={navigate} />
      <div className="flex-1 flex flex-col min-w-0">
        <Toolbar
          currentPath={currentPath}
          onNavigate={navigate}
          onBack={handleBack}
          onForward={handleForward}
          onUp={handleUp}
          canGoBack={historyIndex > 0}
          canGoForward={historyIndex < history.length - 1}
          viewMode={viewMode}
          onViewModeChange={setViewMode}
          searchQuery={searchQuery}
          onSearchChange={setSearchQuery}
        />
        <div className="flex-1 flex overflow-hidden">
          <Playground
            currentPath={currentPath}
            onNavigate={navigate}
            viewMode={viewMode}
            selectedPaths={selectedPaths}
            onSelect={setSelectedPaths}
            listing={listing}
            isLoading={isLoading}
            error={error}
            searchQuery={searchQuery}
          />
          <AnimatePresence>
            {selectedEntries.length > 0 && (
              <Inspector selectedEntries={selectedEntries} />
            )}
          </AnimatePresence>
        </div>
      </div>
    </div>
  );
}

