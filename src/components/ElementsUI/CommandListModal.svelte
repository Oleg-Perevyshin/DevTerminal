<script lang="ts">
  import type { SavedCommands } from "src/stores/Interfaces"
  import { END_PACKAGE, POE_CANABLE_ARGUMENT, POE_CANABLE_HEADER, POE_SERIAL_ARGUMENT, POE_SERIAL_HEADER } from "../options"
  import * as UI from "poe-svelte-ui-lib"
  import { UpdateStatus } from "../../stores/StatusStore"
  import { open, save } from "@tauri-apps/plugin-dialog"
  import { create, readTextFile, writeTextFile } from "@tauri-apps/plugin-fs"
  import yaml from "js-yaml"
  import { commandTabs } from "../../stores/Dictionary"
  import PhEyeSlash from "../../appIcons/PhEyeSlash.svelte"
  import PhEye from "../../appIcons/PhEye.svelte"
  import ButtonDelete from "../../appIcons/ButtonDelete.svelte"
  import { log } from "../Common"

  let {
    showCommandListModal = $bindable(false),
    savedCommands = $bindable(),
    selectedProtocol,
  }: { showCommandListModal: boolean; savedCommands: SavedCommands[]; selectedProtocol: string } = $props()

  let isFileValid = $state(true)

  /* Открыть файл с быстрыми командами */
  const openCommandFile = async () => {
    log("INFO", "openCommandFile", "Начало открытия файла с командами")
    const selectedFile = await open({
      multiple: false,
      filters: [{ name: "Yaml File", extensions: ["yaml", "yml"] }],
    })

    if (!selectedFile) {
      log("INFO", "openCommandFile", "Файл не выбран, операция открытия отменена")
      return
    }
    log("INFO", "openCommandFile", `Выбран файл: ${selectedFile}`)

    const yamlData = (await readTextFile(selectedFile)).toString()
    const openedCommands: SavedCommands[] = yaml.load(yamlData) as SavedCommands[]

    let addedCount = 0
    openedCommands.forEach(newCommand => {
      if (!savedCommands.some(command => command.name === newCommand.name)) {
        savedCommands = [...savedCommands, newCommand]
        addedCount++
        log("INFO", "openCommandFile", `Добавлена команда: ${newCommand.name}`)
      } else {
        log("INFO", "openCommandFile", `Команда ${newCommand.name} уже существует, пропущена`)
      }
    })
    log("INFO", "openCommandFile", `Команд добавлено: ${addedCount}, всего команд теперь: ${savedCommands.length}`)
    UpdateStatus(`Command list is loaded`)
    log("INFO", "openCommandFile", "Файл с командами успешно загружен")
  }

  /* Сохранить файл с быстрыми командами */
  const saveCommandFile = async () => {
    log("INFO", "saveCommandFile", "Начало сохранения файла с командами")
    const path = await save({
      filters: [{ name: "Yaml File", extensions: ["yaml", "yml"] }],
    })
    if (!path) {
      log("INFO", "saveCommandFile", "Путь для сохранения не выбран, операция отменена")
      return
    }
    await create(path).then(file => file.close())

    const newSaved = yaml.dump($state.snapshot(savedCommands))
    log("INFO", "saveCommandFile", `Состояние команд сериализовано, размер данных: ${newSaved.length} символов`)

    await writeTextFile(path, newSaved)
    UpdateStatus(`Command list is saved`)
    log("INFO", "saveCommandFile", "Файл с командами успешно сохранён")
  }
</script>

<UI.Modal bind:isOpen={showCommandListModal} mainClass="overflow-y-hidden" title="Command list" wrapperClass="w-[80%] h-[75%]">
  {#snippet main()}
    <UI.Tabs
      items={Object.entries(commandTabs).map(([name]) => ({ name, class: `w-[${(1 / Object.entries(commandTabs).length) * 100}%]` }))}
      children={Tab}
      activeTab={Object.entries(commandTabs).findIndex(([, item]) => selectedProtocol.startsWith(item))}
      size={{ height: 1, width: 1 }} />
    {#snippet Tab(item: { name: string })}
      {@const category = commandTabs[item.name as keyof typeof commandTabs]}
      {@const categoryName = category as keyof Omit<SavedCommands, "name">}
      <div class="flex flex-col">
        {#each savedCommands as command}
          {#if command[categoryName]}
            <div class="my-1 flex items-end gap-3 px-0.5">
              <UI.Input
                label={{ name: "Command name", class: "!px-0" }}
                bind:value={command.name}
                wrapperClass="w-150"
                componentClass={savedCommands.filter(c => categoryName in c && c.name === command.name).length >= 2
                  ? "border-red-400 shadow-[0_0_6px_var(--red-color)] focus:border-red-400"
                  : ""} />

              {#if categoryName === "SimpleSerial"}
                <UI.Input label={{ name: "Regular String" }} bind:value={command[categoryName].data} type="text" />
                <UI.Select
                  wrapperClass="w-60"
                  label={{ name: "End", class: "" }}
                  value={END_PACKAGE.find(o => o.value == command[categoryName]?.endPackage)}
                  options={END_PACKAGE}
                  onUpdate={option => (command[categoryName]!.endPackage = option.value as string)} />
              {:else if categoryName === "POESerial"}
                <UI.Select
                  label={{ name: "Header", class: "px-0" }}
                  wrapperClass="w-23"
                  value={POE_SERIAL_HEADER.find(o => o.value == command.POESerial?.header) || {
                    id: `input-${command.POESerial?.header}`,
                    name: command.POESerial?.header,
                    value: command.POESerial?.header as string,
                  }}
                  options={POE_SERIAL_HEADER}
                  onUpdate={value => (command.POESerial!.header = value.value as string)} />
                <UI.Select
                  label={{ name: "Argument" }}
                  type="input"
                  wrapperClass="w-70"
                  value={POE_SERIAL_ARGUMENT.find(e => e.value == command.POESerial?.argument) || {
                    id: `input-${command.POESerial?.argument}`,
                    name: command.POESerial?.argument,
                    value: command.POESerial?.argument as string,
                  }}
                  options={POE_SERIAL_ARGUMENT}
                  onUpdate={value => {
                    command.POESerial!.argument = value.value as string
                  }} />
                <UI.Input label={{ name: "Value" }} bind:value={command.POESerial!.value} type="text" />
              {:else if categoryName === "POECanable"}
                <UI.Select
                  label={{ name: "Header", class: "px-0" }}
                  wrapperClass="w-23"
                  value={POE_CANABLE_HEADER.find(e => e.value == command.POECanable?.header) || {
                    id: `input-${command.POECanable?.header}`,
                    name: command.POECanable?.header,
                    value: command.POECanable?.header as string,
                  }}
                  options={POE_CANABLE_HEADER}
                  onUpdate={value => {
                    command.POECanable!.header = value.value?.toString() ?? ""
                    console.log(value.value, command.POECanable?.header)
                  }} />
                <UI.Select
                  label={{ name: "Argument" }}
                  type="input"
                  wrapperClass="w-[400px]"
                  value={POE_CANABLE_ARGUMENT.find(e => e.value == command.POECanable?.argument) || {
                    id: `input-${command.POECanable?.argument}`,
                    name: command.POECanable?.argument,
                    value: command.POECanable?.argument as string,
                  }}
                  options={POE_CANABLE_ARGUMENT}
                  onUpdate={value => {
                    command.POECanable!.argument = value.value?.toString() ?? ""
                  }} />
                <UI.Input
                  label={{ name: "TargetID", class: "!px-0" }}
                  wrapperClass="w-20"
                  bind:value={command.POECanable!.targetID}
                  type="text"
                  maxlength={3}
                  help={{ regExp: /^(?:[1-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])$/ }} />
                <UI.Input
                  label={{ name: "ReturnID", class: "!px-0" }}
                  wrapperClass="w-20"
                  bind:value={command.POECanable!.returnID}
                  type="text"
                  maxlength={3}
                  help={{ regExp: /^(?:[1-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])$/ }} />
                <UI.Input label={{ name: "Data", class: "!px-0" }} bind:value={command.POECanable!.data} type="text" />
              {/if}

              <UI.Button
                wrapperClass="w-9"
                componentClass="p-0.5"
                content={{
                  icon: command[categoryName]?.isShown ? PhEyeSlash : PhEye,
                  info: { text: command[categoryName]?.isShown ? "Hide command" : "Show command", side: "top" },
                }}
                onClick={() => (command[categoryName]!.isShown = !command[categoryName]?.isShown)} />
              <UI.Button
                wrapperClass="w-9"
                componentClass="p-0.5"
                content={{ icon: ButtonDelete, info: { text: "Delete", side: "top" } }}
                onClick={() => (savedCommands = savedCommands.filter(c => !(c.name === command.name && categoryName in c)))} />
            </div>
          {/if}
        {/each}
        {#if savedCommands.filter(c => categoryName in c && c.name == "").length != 1}
          <UI.Button
            content={{ name: "Create command" }}
            wrapperClass="mt-2 w-60 self-center"
            onClick={() => savedCommands.push({ [categoryName]: { data: "", endPackage: "\n", isShown: true }, name: "" })} />
        {/if}
      </div>
    {/snippet}
  {/snippet}
  {#snippet footer()}
    <div class="flex w-full flex-row justify-between">
      <UI.FileAttach wrapperClass="w-120" accept=".yaml" onChange={() => openCommandFile()} />

      <UI.Button wrapperClass="w-1/4" componentClass="bg-green" content={{ name: "Save file" }} disabled={!isFileValid} onClick={() => saveCommandFile()} />
    </div>
  {/snippet}
</UI.Modal>
