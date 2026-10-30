import type { Attacher, Transformer } from "./plugin.ts"

/** A JavaScript plugin a processor uses: its attacher, options and, once attached, its transformer. */
export interface Host {
  attacher: Attacher
  parameters: unknown[]
  transformer?: Transformer
}
