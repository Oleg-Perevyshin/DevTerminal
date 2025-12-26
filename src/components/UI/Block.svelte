<!-- src\components\UI\Block.svelte -->
<script lang="ts">
  import { slide } from "svelte/transition"
  import ButtonArrow from "../../appIcons/ButtonArrow.svelte"
  import { type Snippet } from "svelte"
  import { twMerge } from "tailwind-merge"

  let {
    id = crypto.randomUUID(),
    main,
    details,
    label,
    isAccordion = false,
    isDropdownOpen = $bindable(false),
    wrapperClass,
    detailsClass,
    disabled = false,
  }: {
    id?: string
    main: Snippet
    details?: Snippet
    label: string
    isAccordion?: boolean
    isDropdownOpen?: boolean
    wrapperClass?: string
    detailsClass?: string
    disabled?: boolean
  } = $props()

  const toggleDropdown = () => {
    isDropdownOpen = !isDropdownOpen
  }

  const handleKeydown = (event: KeyboardEvent) => {
    if (event.key === " ") {
      toggleDropdown()
      event.preventDefault()
    }
  }
</script>

<div {id} class={twMerge(`relative m-1.5 flex flex-col rounded-xl border border-(--gray-color) p-3 inset-shadow-[0_0_3px_rgb(0_0_0_/0.75)]`, wrapperClass)}>
  <div class="absolute -top-3 left-3 bg-[#f5f7fa] px-1">
    <button class={` font-semibold ${disabled ? "cursor-not-allowed opacity-50" : ""}`} onclick={toggleDropdown} onkeydown={handleKeydown}>
      {label}
      {#if isAccordion}
        <ButtonArrow id={`svg-arrow-${id}`} bind:isDropdownOpen {disabled} />
      {/if}
    </button>
  </div>

  <div class="flex items-center">
    {@render main?.()}
  </div>

  {#if isAccordion && isDropdownOpen && !disabled}
    <div class={`z-10 w-full transform rounded-b-2xl ${detailsClass}`} transition:slide={{ duration: 300 }}>
      {@render details?.()}
    </div>
  {/if}
</div>
