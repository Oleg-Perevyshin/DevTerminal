export interface ISelectOption<T = unknown> {
  id: string
  value?: T
  name?: string
  class?: string
  icon?: {
    component?: ConstructorOfATypedSvelteComponent | null
    properties?: Record<string, unknown>
  }
  disabled?: boolean
}

export interface SavedCommands {
  SimpleSerial?: {
    data: string
    endPackage: string
    isShown: boolean
  }
  POESerial?: {
    header: string
    argument: string
    value: string
    isShown: boolean
  }
  POECanable?: {
    header: string
    argument: string
    targetID: string
    returnID: string
    data: string
    isShown: boolean
  }
  name: string
}
