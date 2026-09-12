import { Folder, FileCode, FileText, Image, FileCog, Terminal, Bot, File } from "lucide-react";
import { FileNodeDto } from "../../types";

export function FileNodeIcon({ entry, className }: { entry: FileNodeDto; className?: string }) {
  if (entry.kind === "directory" || entry.fileType === "folder") {
    return <Folder className={className} style={{ color: "var(--color-type-folder)" }} />;
  }

  switch (entry.fileType) {
    case "rust":
      return <FileCode className={className} style={{ color: "var(--color-type-rust)" }} />;
    case "python":
      return <FileCode className={className} style={{ color: "var(--color-type-python)" }} />;
    case "cpp":
      return <FileCode className={className} style={{ color: "var(--color-type-cpp)" }} />;
    case "config":
      return <FileCog className={className} style={{ color: "var(--color-type-config)" }} />;
    case "markdown":
      return <FileText className={className} style={{ color: "var(--color-type-markdown)" }} />;
    case "image":
      return <Image className={className} style={{ color: "var(--color-type-image)" }} />;
    case "shell":
      return <Terminal className={className} style={{ color: "var(--color-type-shell)" }} />;
    case "model":
      return <Bot className={className} style={{ color: "var(--color-type-model)" }} />;
    default:
      return <File className={className} style={{ color: "var(--color-text-secondary)" }} />;
  }
}
