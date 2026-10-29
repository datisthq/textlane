/** A native plugin as a pipeline names it, with its options when given. */
export interface Plugin<Name extends string = string, Options = unknown> {
  name: Name
  options?: Options
}

/** Plugins and the settings they share, as unified's presets. */
export interface Preset {
  plugins?: PluggableList
  settings?: Record<string, unknown>
}

/** A plugin, by descriptor or name, with options, or a preset. */
export type Pluggable = Plugin | string | [Plugin | string, unknown?] | Preset

/** Several pluggables, used in order. */
export type PluggableList = Pluggable[]
