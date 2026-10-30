import type {
  HostFile,
  Message,
  MessageOptions,
  Place,
  Point,
} from "../../models/file.ts"

/** The file JavaScript plugins share during one run, holding the document's text. */
export function createFile(value: string | undefined): HostFile {
  const file: HostFile = {
    value,
    data: {},
    messages: [],
    message(reason, options) {
      const message = createMessage(reason, options)
      file.messages.push(message)
      return message
    },
    fail(reason, options) {
      throw Object.assign(new Error(reason), file.message(reason, options))
    },
  }
  return file
}

function createMessage(
  reason: string,
  options: MessageOptions | Place | undefined,
): Message {
  const settings =
    options && ("ruleId" in options || "source" in options || "place" in options)
      ? options
      : { place: options }
  return {
    reason,
    ruleId: "ruleId" in settings ? (settings.ruleId ?? null) : null,
    source: "source" in settings ? (settings.source ?? null) : null,
    place: readPlace("place" in settings ? settings.place : undefined),
  }
}

function readPlace(place: unknown): Message["place"] {
  if (typeof place !== "object" || place === null) return null
  if ("position" in place) return readPlace(place.position)
  if ("start" in place && "end" in place && isPoint(place.start) && isPoint(place.end)) {
    return { start: place.start, end: place.end }
  }
  return isPoint(place) ? { start: place, end: place } : null
}

function isPoint(value: unknown): value is Point {
  return (
    typeof value === "object" && value !== null && "line" in value && "column" in value
  )
}
