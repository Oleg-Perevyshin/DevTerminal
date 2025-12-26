<script lang="ts">
  import { onDestroy, onMount } from "svelte"
  import { SerialPort } from "tauri-plugin-serialplugin-api"
  import * as UI from "poe-svelte-ui-lib"

  import { StatusStore, UpdateStatus } from "./stores/StatusStore"
  import PortTab from "./components/PortTab.svelte"
  import type { ISelectOption } from "./stores/Interfaces"
  import { listen, type UnlistenFn } from "@tauri-apps/api/event"
  import type { Unsubscriber } from "svelte/store"
  import Block from "./components/UI/Block.svelte"

  /* Инициализация */
  let portList: ISelectOption[] = $state([])
  let statusMessage: string = $state("")

  let isLeftCollapsed: boolean = $state(false)
  let isRightCollapsed: boolean = $state(true)

  let unsubscribe: Unsubscriber[]
  let unlistenStatus: UnlistenFn

  let currentTheme: boolean = $state(true)

  $effect(() => {
    if (isLeftCollapsed) {
      isRightCollapsed = false
    }
  })
  $effect(() => {
    if (isRightCollapsed) {
      isLeftCollapsed = false
    }
  })

  /* Получение списка портов */
  const getPortList = async () => {
    try {
      const managedPorts = await SerialPort.managed_ports()
      portList = Object.keys(await SerialPort.available_ports())
        .sort()
        .map(portName => ({
          id: "portOption" + portName,
          name: managedPorts.includes(portName) ? portName + " (connected)" : portName,
          value: portName,
          class: managedPorts.includes(portName) ? "bg-red" : "",
          disabled: managedPorts.includes(portName) ? true : false,
        }))
    } catch (err) {
      UpdateStatus(`Error getting port list:: ${err}`)
    }
  }
  setInterval(getPortList, 1000)

  /* Переключение темы */
  // const switchTheme = () => {
  //   currentTheme = !currentTheme
  //   document.body.classList.toggle('dark', !currentTheme)
  //   document.body.classList.toggle('light', currentTheme)
  //   localStorage.setItem('AppTheme', currentTheme ? 'light' : 'dark')
  // }

  onMount(async () => {
    getPortList()

    unlistenStatus = await listen<string>("app-status", event => {
      const statusMessage = event.payload
      UpdateStatus(statusMessage)
    })
    unsubscribe = [StatusStore.subscribe(value => (statusMessage = value || ""))]

    const savedTheme = localStorage.getItem("AppTheme") || "light"
    localStorage.setItem("AppTheme", `${savedTheme}`)
    document.body.classList.toggle("dark", savedTheme === "dark")
    document.body.classList.toggle("light", savedTheme === "light")
    currentTheme = savedTheme === "light"
  })

  onDestroy(() => {
    if (unlistenStatus) unlistenStatus()
    unsubscribe.forEach(fn => fn())
  })
</script>

<svelte:head>
  <link rel="preconnect" href="https://fonts.googleapis.com" />
  <link rel="preconnect" href="https://fonts.gstatic.com" />
  <link href="https://fonts.googleapis.com/css2?family=Montserrat:ital,wght@0,100..900;1,100..900&display=swap" rel="stylesheet" />
</svelte:head>

<div class="flex h-screen flex-col">
  <div class="flex flex-1">
    <PortTab {portList} bind:isCollapsed={isLeftCollapsed} />
    <PortTab {portList} bind:isCollapsed={isRightCollapsed} />
  </div>
  <Block label="Status" wrapperClass="h-10">
    {#snippet main()}
      <div class="-m-2 w-full font-semibold text-yellow-600">
        {statusMessage}
      </div>
    {/snippet}
  </Block>
</div>

<style>
  /* Стили для светлой темы */
  :global(body.light) {
    color: #333; /* Цвет текста для светлой темы */
    background: #f5f7fa;
  }

  /* Стили для темной темы */
  :global(body.dark) {
    color: #e2e3e7; /* Цвет текста для темной темы */
    background: #f5f7fa;
  }
</style>
