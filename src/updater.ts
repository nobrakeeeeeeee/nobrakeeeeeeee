import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export async function checkForUpdate(): Promise<Update | null> {
  try {
    return await check();
  } catch {
    // Dev builds / unsigned local builds have no updater endpoint to hit.
    return null;
  }
}

export async function installAndRelaunch(update: Update): Promise<void> {
  await update.downloadAndInstall();
  await relaunch();
}
