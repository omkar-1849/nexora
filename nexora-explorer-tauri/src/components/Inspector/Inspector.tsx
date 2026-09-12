import { FileNodeDto } from "../../types";
import { FileNodeIcon } from "../Playground/FileNodeIcon";

import { motion } from "framer-motion";

interface InspectorProps {
  selectedEntries: FileNodeDto[];
}

export function Inspector({ selectedEntries }: InspectorProps) {
  if (selectedEntries.length === 0) {
    return null;
  }

  if (selectedEntries.length > 1) {
    const totalSize = selectedEntries.reduce((acc, e) => acc + (e.sizeBytes || 0), 0);
    return (
      <motion.aside
        key="multiple"
        initial={{ opacity: 0, x: 20 }}
        animate={{ opacity: 1, x: 0 }}
        exit={{ opacity: 0, x: 20 }}
        transition={{ duration: 0.2, ease: [0.16, 1, 0.3, 1] }}
        className="w-72 bg-bg-surface border-l border-border-subtle h-full flex flex-col p-6"
      >
        <h3 className="text-xl mb-6 text-text-primary">Multiple Selection</h3>
        <div className="space-y-4 text-sm">
          <DetailRow label="Items selected" value={String(selectedEntries.length)} />
          <DetailRow label="Total size" value={totalSize + " B"} />
        </div>
      </motion.aside>
    );
  }

  const entry = selectedEntries[0];

  return (
    <motion.aside
      key={entry.virtualPath}
      initial={{ opacity: 0, x: 20 }}
      animate={{ opacity: 1, x: 0 }}
      exit={{ opacity: 0, x: 20 }}
      transition={{ duration: 0.2, ease: [0.16, 1, 0.3, 1] }}
      className="w-72 bg-bg-surface border-l border-border-subtle h-full flex flex-col"
    >
      <div className="p-6 border-b border-border-subtle flex flex-col items-center text-center">
        <FileNodeIcon entry={entry} className="w-16 h-16 mb-4" />
        <h3 className="text-xl text-text-primary break-all line-clamp-2">{entry.name}</h3>
        <span className="text-sm text-text-secondary mt-1 capitalize">
          {entry.kind === "directory" ? "Folder" : entry.fileType || "File"}
        </span>
      </div>
      <div className="p-6 space-y-4 text-sm flex-1 overflow-auto">
        <DetailRow label="Virtual Path" value={entry.virtualPath} />
        <DetailRow label="Size" value={entry.sizeBytes !== undefined ? (entry.sizeBytes + " B") : undefined} />
        <DetailRow label="Owner" value={entry.owner} />
        <DetailRow label="Permissions" value={entry.permissions} />
        <DetailRow label="Created" value={entry.createdAt} />
        <DetailRow label="Modified" value={entry.modifiedAt} />
        <DetailRow label="Accessed" value={entry.accessedAt} />
      </div>
    </motion.aside>
  );
}

function DetailRow({ label, value }: { label: string; value?: string }) {
  return (
    <div>
      <div className="text-text-tertiary text-xs mb-0.5">{label}</div>
      <div className="text-text-secondary font-mono break-all text-[13px]">
        {value || "Not available"}
      </div>
    </div>
  );
}
