<script lang="ts">
  import { onMount, type Snippet } from "svelte"

  let {
    isOpen = $bindable(false),
    title,
    width = "",
    enterClick = () => {},
    main,
    footer,
  }: {
    isOpen?: boolean
    title?: string
    width?: string
    enterClick?: () => void
    main: Snippet
    footer?: Snippet
  } = $props()

  const handleKeyDown = (event: KeyboardEvent) => {
    if (event.key === "Escape") {
      isOpen = false
    }
    if (event.key === "Enter" && enterClick) {
      enterClick()
    }
  }

  const handleClickOutside = (event: MouseEvent) => {
    const target = event.target as HTMLElement
    if (target.classList.contains("absolute")) {
      isOpen = false
    }
  }

  onMount(() => {
    document.addEventListener("keydown", handleKeyDown)
    document.addEventListener("click", handleClickOutside)
    return () => {
      document.removeEventListener("keydown", handleKeyDown)
      document.removeEventListener("click", handleClickOutside)
    }
  })
</script>

{#if isOpen}
  <div class="bg-opacity-50 absolute inset-0 z-60 flex items-center justify-center backdrop-blur-[1px] transition-opacity duration-300">
    <div class="flex max-h-[90vh] flex-col rounded-lg bg-[#f5f7fa] shadow-xl transition-all duration-300 {width} w-full">
      <div class="flex shrink-0 items-center justify-between border-b border-gray-200 px-4 py-2">
        <h3 class="text-lg font-medium text-gray-900">
          {title}
        </h3>
        <button class="cursor-pointer text-gray-400 hover:text-gray-500" onclick={() => (isOpen = false)} aria-label="Close modal">
          <svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <div class="grow overflow-y-auto p-4 py-1 text-left whitespace-pre">
        {@render main?.()}
      </div>
      {#if footer}
        <div class="flex shrink-0 justify-end space-x-3 border-t border-gray-200 p-4">
          {@render footer?.()}
        </div>
      {/if}
    </div>
  </div>
{/if}
