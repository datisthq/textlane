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
}

/** A file to run a tree against: its text, or a file holding it as `value`. */
export type Compatible = string | { value: string }

/** Where a message is: a point, a position, or a node holding one. */
export type Place =
  | { line: number; column: number; offset?: number | null }
  | { start: Point; end: Point }
  | { position?: unknown }

/** What `message` takes besides the reason. */
export interface MessageOptions {
  place?: Place
  ruleId?: string
  source?: string
}

/** The file a JavaScript plugin gets, as a vfile: the document, its data and messages. */
export interface HostFile {
  value: string | undefined
  data: Record<string, unknown>
  messages: Message[]
  /** Report a warning against the file. */
  message(reason: string, options?: MessageOptions | Place): Message
  /** Report an error and throw it. */
  fail(reason: string, options?: MessageOptions | Place): never
}
