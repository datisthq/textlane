import { basename, dirname, extname } from "node:path"
import type { Message, MessageOptions, Place, Point } from "./file.ts"

/**
 * The file a JavaScript plugin gets, as a vfile: the document, its path, data and messages.
 * Only what plugins report here reaches the result; native stages report there directly.
 */
export class HostFile {
  value: string | undefined
  data: Record<string, unknown> = {}
  messages: HostMessage[] = []
  history: string[] = []
  cwd = process.cwd()

  constructor(options: { value?: string; path?: string }) {
    this.value = options.value
    if (options.path !== undefined) this.history.push(options.path)
  }

  /** The current path; setting it adds to `history`. */
  get path() {
    return this.history.at(-1)
  }

  set path(path: string | undefined) {
    if (path !== undefined && path !== this.path) this.history.push(path)
  }

  get dirname() {
    return this.path === undefined ? undefined : dirname(this.path)
  }

  get basename() {
    return this.path === undefined ? undefined : basename(this.path)
  }

  get extname() {
    return this.path === undefined ? undefined : extname(this.path)
  }

  get stem() {
    return this.path === undefined ? undefined : basename(this.path, extname(this.path))
  }

  /** Report a warning against the file, as vfile's `message`; `origin` is `source:ruleId`. */
  message(
    reason: string | Error,
    options?: MessageOptions | Place | null,
    origin?: string,
  ) {
    const message = new HostMessage(reason, options, origin)
    message.file = this.path ?? ""
    message.fatal = false
    this.messages.push(message)
    return message
  }

  /** Report a note against the file, neither a warning nor an error. */
  info(reason: string | Error, options?: MessageOptions | Place | null, origin?: string) {
    const message = this.message(reason, options, origin)
    message.fatal = null
    return message
  }

  /** Report an error against the file and throw it. */
  fail(
    reason: string | Error,
    options?: MessageOptions | Place | null,
    origin?: string,
  ): never {
    const message = this.message(reason, options, origin)
    message.fatal = true
    throw message
  }

  toString() {
    return this.value ?? ""
  }
}

/** A diagnostic a JavaScript plugin reported, as vfile-message has it. */
export class HostMessage extends Error {
  reason: string
  ruleId: string | null = null
  source: string | null = null
  place: Message["place"]
  line: number | undefined
  column: number | undefined
  fatal: boolean | null | undefined
  file = ""
  ancestors: unknown[] | undefined
  cause: unknown
  actual: string | undefined
  expected: string[] | undefined
  url: string | undefined
  note: string | undefined

  constructor(
    reason: string | Error,
    options?: MessageOptions | Place | null,
    origin?: string,
  ) {
    const text = typeof reason === "string" ? reason : reason.message
    super(text)
    this.reason = text
    if (typeof reason !== "string") this.cause = reason
    const settings = readSettings(options)
    if (settings.cause !== undefined) this.cause = settings.cause
    this.ancestors = settings.ancestors
    this.ruleId = settings.ruleId ?? null
    this.source = settings.source ?? null
    if (origin !== undefined) {
      const index = origin.indexOf(":")
      if (index === -1) this.ruleId = origin
      else [this.source, this.ruleId] = [origin.slice(0, index), origin.slice(index + 1)]
    }
    this.place = readPlace(settings.place ?? settings.ancestors?.at(-1))
    this.line = this.place?.start.line
    this.column = this.place?.start.column
    this.name = writePlace(this.place)
  }

  /** A message a native stage reported, as the file's other messages are. */
  static fromMessage(message: Message, path: string | undefined) {
    const options = {
      place: message.place ?? undefined,
      ruleId: message.ruleId ?? undefined,
      source: message.source ?? undefined,
    }
    const created = new HostMessage(message.reason, options)
    created.file = path ?? ""
    created.fatal = false
    return created
  }

  /** The message as it crosses into the crate. */
  toJSON(): Message {
    return {
      reason: this.reason,
      ruleId: this.ruleId,
      source: this.source,
      place: this.place,
    }
  }
}

// vfile tells options from a place by their keys; anything else is the place itself
function readSettings(
  options: MessageOptions | Place | null | undefined,
): MessageOptions {
  if (!options) return {}
  return isMessageOptions(options) ? options : { place: options }
}

function isMessageOptions(options: MessageOptions | Place): options is MessageOptions {
  return ["ancestors", "cause", "place", "ruleId", "source"].some(key => key in options)
}

function readPlace(place: unknown): Message["place"] {
  if (typeof place !== "object" || place === null) return null
  if ("position" in place) return readPlace(place.position)
  if ("start" in place && "end" in place) {
    const start = readPoint(place.start)
    const end = readPoint(place.end)
    return start && end ? { start, end } : null
  }
  const point = readPoint(place)
  return point ? { start: point, end: point } : null
}

function readPoint(value: unknown): Point | undefined {
  if (typeof value !== "object" || value === null) return undefined
  if (!("line" in value && "column" in value)) return undefined
  const { line, column } = value
  if (typeof line !== "number" || typeof column !== "number") return undefined
  const offset =
    "offset" in value && typeof value.offset === "number" ? value.offset : null
  return { line, column, offset }
}

// unist-util-stringify-position: `1:1-1:2`, or `1:1` for an unknown place
function writePlace(place: Message["place"]) {
  if (!place) return "1:1"
  const start = `${place.start.line}:${place.start.column}`
  const end = `${place.end.line}:${place.end.column}`
  return start === end ? start : `${start}-${end}`
}
