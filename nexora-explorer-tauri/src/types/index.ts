export type FileTypeTag =
  | "folder"
  | "rust"
  | "python"
  | "cpp"
  | "config"
  | "markdown"
  | "image"
  | "shell"
  | "model"
  | "unknown";

export interface FileNodeDto {
  name: string;
  virtualPath: string;
  kind: "file" | "directory";
  fileType?: FileTypeTag;
  sizeBytes?: number;
  owner?: string;
  permissions?: string;
  createdAt?: string;
  modifiedAt?: string;
  accessedAt?: string;
}

export interface DirectoryListingDto {
  path: string;
  entries: FileNodeDto[];
}

export interface FsErrorDto {
  kind: "notFound" | "permissionDenied" | "invalidPath" | "ioError" | "unknown";
  message: string;
}
