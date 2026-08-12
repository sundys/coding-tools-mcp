import { listen } from "@tauri-apps/api/event";

export type CloseDialogOpener = () => void;

/**
 * Open the confirm dialog when the Rust close controller asks the UI to do so.
 * Rust owns interception and process-lifetime choice reuse so a WebView recreate
 * cannot lose the remembered action.
 * Returns an unsubscribe function.
 */
export function startCloseGuard(openDialog: CloseDialogOpener): () => void {
  if (typeof window === "undefined") {
    return () => {};
  }

  let unlistenEvent: (() => void) | undefined;
  let disposed = false;

  void (async () => {
    try {
      unlistenEvent = await listen("close-requested", () => {
        openDialog();
      });
    } catch {
      // ignore
    }

    if (disposed) {
      unlistenEvent?.();
    }
  })();

  return () => {
    disposed = true;
    unlistenEvent?.();
  };
}
