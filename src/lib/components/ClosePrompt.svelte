<script lang="ts">
  interface Props {
    open: boolean;
    busy?: boolean;
    onCancel: () => void;
    onBackground: () => void;
    onExit: () => void;
  }

  let {
    open,
    busy = false,
    onCancel,
    onBackground,
    onExit,
  }: Props = $props();

  let cancelButton = $state<HTMLButtonElement>();

  $effect(() => {
    if (open) {
      requestAnimationFrame(() => cancelButton?.focus());
    }
  });

  function handleKeydown(event: KeyboardEvent) {
    if (open && !busy && event.key === "Escape") {
      event.preventDefault();
      onCancel();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <div class="close-prompt-backdrop">
    <div
      class="close-prompt"
      role="dialog"
      aria-modal="true"
      aria-labelledby="close-prompt-title"
      aria-describedby="close-prompt-description"
    >
      <h2 id="close-prompt-title">关闭 Coding Tools MCP？</h2>
      <p id="close-prompt-description">
        选择后台运行可隐藏窗口并保持 MCP、Actions 和隧道服务继续运行，之后可通过系统托盘重新打开。
      </p>

      <div class="close-prompt-actions">
        <button
          bind:this={cancelButton}
          type="button"
          class="close-prompt-button close-prompt-button-secondary"
          disabled={busy}
          onclick={onCancel}
        >
          取消
        </button>
        <button
          type="button"
          class="close-prompt-button close-prompt-button-secondary"
          disabled={busy}
          onclick={onBackground}
        >
          后台运行
        </button>
        <button
          type="button"
          class="close-prompt-button close-prompt-button-danger"
          disabled={busy}
          onclick={onExit}
        >
          直接关闭
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .close-prompt-backdrop {
    position: fixed;
    inset: 0;
    z-index: 10000;
    display: grid;
    place-items: center;
    padding: 24px;
    background: rgba(15, 23, 42, 0.42);
    backdrop-filter: blur(4px);
  }

  .close-prompt {
    width: min(896px, calc(100vw - 48px));
    padding: 48px 50px 42px;
    border: 1px solid var(--border);
    border-radius: 24px;
    background: var(--card-bg);
    box-shadow: 0 24px 64px rgba(15, 23, 42, 0.22);
  }

  h2 {
    margin: 0;
    color: var(--text-main);
    font-size: 30px;
    font-weight: 700;
    letter-spacing: 0;
  }

  p {
    max-width: 760px;
    margin: 20px 0 0;
    color: var(--text-muted);
    font-size: 20px;
    line-height: 1.65;
    letter-spacing: 0;
  }

  .close-prompt-actions {
    display: flex;
    justify-content: flex-end;
    gap: 16px;
    margin-top: 44px;
  }

  .close-prompt-button {
    min-width: 104px;
    min-height: 62px;
    padding: 0 24px;
    border-radius: 16px;
    font-size: 20px;
    font-weight: 600;
    letter-spacing: 0;
    cursor: pointer;
    transition: background var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out),
      opacity var(--duration-fast) var(--ease-out);
  }

  .close-prompt-button:disabled {
    cursor: wait;
    opacity: 0.62;
  }

  .close-prompt-button-secondary {
    border: 1px solid var(--border);
    background: var(--card-bg);
    color: var(--text-secondary);
  }

  .close-prompt-button-secondary:hover:not(:disabled) {
    border-color: var(--text-muted);
    background: var(--surface-hover);
    color: var(--text-main);
  }

  .close-prompt-button-danger {
    border: 1px solid #e60012;
    background: #e60012;
    color: #fff;
  }

  .close-prompt-button-danger:hover:not(:disabled) {
    border-color: #c80010;
    background: #c80010;
  }

  @media (max-width: 720px) {
    .close-prompt {
      padding: 32px 28px 28px;
      border-radius: 18px;
    }

    h2 {
      font-size: 24px;
    }

    p {
      font-size: 16px;
    }

    .close-prompt-actions {
      flex-wrap: wrap;
      gap: 10px;
      margin-top: 32px;
    }

    .close-prompt-button {
      flex: 1 1 140px;
      min-height: 52px;
      font-size: 16px;
    }
  }
</style>
