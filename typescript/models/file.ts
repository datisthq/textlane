/** A unist node as JSON: what `parse` returns and `run` and `stringify` take. */
export interface Node {
  type: string
  [key: string]: unknown
}

/** One place in a file: a 1-indexed line and column and a 0-indexed offset, when known. */
export interface Point {
  line: number
  column: number
  offset: number | null
}

/** A diagnostic a plugin reported, as vfile-message has it. */
export interface Message {
  reason: string
  ruleId: string | null
  source: string | null
  place: { start: Point; end: Point } | null
}

/** What `process` gives back, as a vfile: the output, the messages and the file's data. */
export interface VFile {
  value: string
  messages: Message[]
  data: Record<string, unknown>
  path?: string
}

/** A document: its text, or a file holding it as `value`, with its path when known. */
export type Compatible = string | { value: string; path?: string }

/** Where a message is: a point, a position, or a node holding one. */
export type Place =
  | { line: number; column: number; offset?: number | null }
  | { start: Point; end: Point }
  | { position?: unknown }

/** What `message` takes besides the reason, as vfile-message names it. */
export interface MessageOptions {
  ancestors?: unknown[]
  cause?: unknown
  place?: Place
  ruleId?: string
  source?: string
}
