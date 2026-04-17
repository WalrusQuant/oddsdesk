<script lang="ts">
  interface Props {
    checked: boolean;
    onchange?: (checked: boolean) => void;
    label?: string;
    disabled?: boolean;
  }

  let { checked, onchange, label, disabled = false }: Props = $props();

  function handle() {
    if (disabled) return;
    onchange?.(!checked);
  }
</script>

<label class="toggle" class:disabled>
  <button
    class="track"
    class:on={checked}
    {disabled}
    onclick={handle}
    aria-pressed={checked}
    aria-label={label ?? 'toggle'}
  >
    <span class="thumb"></span>
  </button>
  {#if label}<span class="label">{label}</span>{/if}
</label>

<style>
  .toggle {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-2);
    font-size: var(--fs-sm);
    color: var(--text);
    cursor: pointer;
  }
  .toggle.disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .track {
    position: relative;
    width: 32px;
    height: 18px;
    border-radius: 999px;
    border: 1px solid var(--border);
    background: var(--surface);
    cursor: pointer;
    transition: background 0.15s, border-color 0.15s;
    padding: 0;
  }
  .track.on {
    background: var(--accent);
    border-color: var(--accent);
  }
  .thumb {
    position: absolute;
    top: 1px;
    left: 1px;
    width: 14px;
    height: 14px;
    border-radius: 999px;
    background: var(--text);
    transition: transform 0.15s;
  }
  .track.on .thumb {
    transform: translateX(14px);
    background: #0b0d10;
  }
  .label {
    user-select: none;
  }
</style>
