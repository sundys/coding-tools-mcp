<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import AppShell from "$lib/components/AppShell.svelte";
  import ClosePrompt from "$lib/components/ClosePrompt.svelte";
  import ToastHost from "$lib/components/ToastHost.svelte";
  import WorkspaceNavItem from "$lib/components/WorkspaceNavItem.svelte";
  import {
    createWorkspace,
    getActionsRuntimeStatus,
    getRuntimeStatus,
    listWorkspaces,
  } from "$lib/api/workspaces";
  import { getLastWorkspaceId } from "$lib/api/settings";
  import { actionsRuntimeStates, mcpRuntimeStates, workspaces } from "$lib/stores/app";
  import { showToast } from "$lib/stores/toast";
  import { startUiMemoryGuard } from "$lib/ui-memory-guard";
  import type { RuntimeState } from "$lib/types";

  let { children } = $props();
  let closePromptOpen = $state(false);
  let closeActionPending = $state(false);

  async function handleCloseAction(action: "background" | "exit") {
    if (closeActionPending) return;
    closeActionPending = true;
    closePromptOpen = false;
    try {
      await invoke("handle_close_action", { action });
    } catch (error) {
      closePromptOpen = true;
      showToast(String(error), {
        title: "无法完成关闭操作",
        kind: "error",
        duration: 8000,
      });
    } finally {
      closeActionPending = false;
    }
  }

  async function refreshWorkspaces() {
    const items = await listWorkspaces();
    workspaces.set(items);

    const mcpStates: Record<string, RuntimeState> = {};
    const actionsStates: Record<string, RuntimeState> = {};
    await Promise.all(
      items.map(async (item) => {
        try {
          const [mcp, actions] = await Promise.all([
            getRuntimeStatus(item.id),
            getActionsRuntimeStatus(item.id),
          ]);
          mcpStates[item.id] = mcp.state;
          actionsStates[item.id] = actions.state;
        } catch {
          mcpStates[item.id] = "stopped";
          actionsStates[item.id] = "stopped";
        }
      }),
    );
    mcpRuntimeStates.set(mcpStates);
    actionsRuntimeStates.set(actionsStates);
  }

  async function addWorkspace() {
    try {
      const selected = await open({ directory: true, multiple: false });
      if (!selected || Array.isArray(selected)) return;
      const profile = await createWorkspace(selected);
      await refreshWorkspaces();
      goto(`/workspace/${profile.id}`);
    } catch (error) {
      showToast(String(error), {
        title: "添加工作区失败",
        kind: "error",
        duration: 8000,
      });
    }
  }

  function openWorkspace(id: string) {
    goto(`/workspace/${id}`);
  }

  function openFrpSettings() {
    goto("/settings/frp");
  }

  function openSoftwareSettings() {
    goto("/settings/software");
  }

  function openGeneralSettings() {
    goto("/settings/general");
  }

  function openKeysSettings() {
    goto("/settings/keys");
  }

  onMount(() => {
    const stopGuard = startUiMemoryGuard();
    let disposed = false;
    let stopCloseListener: UnlistenFn | undefined;

    void listen("app-close-requested", () => {
      closePromptOpen = true;
    })
      .then((unlisten) => {
        if (disposed) {
          unlisten();
        } else {
          stopCloseListener = unlisten;
        }
      })
      .catch((error) => {
        showToast(String(error), {
          title: "无法监听窗口关闭事件",
          kind: "error",
          duration: 8000,
        });
      });

    void (async () => {
      await refreshWorkspaces();
      const path = $page.url.pathname;
      if (path === "/") {
        const lastId = await getLastWorkspaceId();
        if (lastId && $workspaces.some((item) => item.id === lastId)) {
          goto(`/workspace/${lastId}`);
        } else if ($workspaces.length > 0) {
          goto(`/workspace/${$workspaces[0].id}`);
        }
      }
    })();
    return () => {
      disposed = true;
      stopCloseListener?.();
      stopGuard();
    };
  });
</script>

<AppShell onAddWorkspace={addWorkspace}>
  {#snippet settingsNav()}
    <button
      type="button"
      class="tx-settings-link {$page.url.pathname === '/settings/general' ? 'active' : ''}"
      onclick={openGeneralSettings}
    >
      通用
    </button>
    <button
      type="button"
      class="tx-settings-link {$page.url.pathname === '/settings/keys' ? 'active' : ''}"
      onclick={openKeysSettings}
    >
      共享密钥
    </button>
    <button
      type="button"
      class="tx-settings-link {$page.url.pathname === '/settings/frp' ? 'active' : ''}"
      onclick={openFrpSettings}
    >
      FRP 配置
    </button>
    <button
      type="button"
      class="tx-settings-link {$page.url.pathname === '/settings/software' ? 'active' : ''}"
      onclick={openSoftwareSettings}
    >
      软件管理
    </button>
  {/snippet}
  {#snippet sidebar()}
    <div class="space-y-1">
      {#each $workspaces as workspace (workspace.id)}
        <WorkspaceNavItem
          workspace={workspace}
          active={$page.url.pathname === `/workspace/${workspace.id}`}
          mcpState={$mcpRuntimeStates[workspace.id] ?? "stopped"}
          actionsState={$actionsRuntimeStates[workspace.id] ?? "stopped"}
          onClick={() => openWorkspace(workspace.id)}
        />
      {/each}
    </div>
  {/snippet}

  {#snippet children()}
    {@render children()}
  {/snippet}
</AppShell>

<ToastHost />

<ClosePrompt
  open={closePromptOpen}
  busy={closeActionPending}
  onCancel={() => (closePromptOpen = false)}
  onBackground={() => void handleCloseAction("background")}
  onExit={() => void handleCloseAction("exit")}
/>
