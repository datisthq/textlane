/** A unist node as JSON: what `parse` returns and `run` and `stringify` take. */
export interface Node {
  type: string
  [key: string]: unknown
}

/** One place in a file: a 1-indexed line and column and a 0-indexed offset. */
export interface Point {
  line: number
  column: number
  offset: number
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
}

/** A file to run a tree against: its text, or a file holding it as `value`. */
export type Compatible = string | { value: string }
