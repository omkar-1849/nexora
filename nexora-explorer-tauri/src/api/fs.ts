import { invoke } from "@tauri-apps/api/core";
import { DirectoryListingDto, FileNodeDto } from "../types";

export async function listDirectory(path: string): Promise<DirectoryListingDto> {
  return await invoke("list_directory", { path });
}

export async function getMetadata(path: string): Promise<FileNodeDto> {
  return await invoke("get_metadata", { path });
}
