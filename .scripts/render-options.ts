// Renders each plugin's options into the docs pages, between the markers
// `<!-- generated:options <plugin> -->` and `<!-- /generated -->`, in each language.

export interface Schema {
  type?: string | string[]
  description?: string
  default?: unknown
  format?: string
  $ref?: string
  anyOf?: Schema[]
  oneOf?: Schema[]
  enum?: unknown[]
  const?: unknown
  items?: Schema
  properties?: Record<string, Schema>
  required?: string[]
  additionalProperties?: Schema | boolean
}

export interface DescribedPlugin {
  name: string
  upstream: string | null
  options: Schema | null
}

export type Language = "typescript" | "python" | "rust"

const MARKERS = /<!-- generated:options (\S+) -->\n[\s\S]*?<!-- \/generated -->/g

export function renderOptions(props: {
  language: Language
  plugin: DescribedPlugin
  definitions: Record<string, Schema>
}) {
  const lines: string[] = []
  if (props.plugin.upstream) {
    const upstream = props.plugin.upstream
    lines.push(`Follows [${upstream}](https://www.npmjs.com/package/${upstream}).`, "")
  }
  const root = props.plugin.options && readRef(props.plugin.options)
  const options = root ? props.definitions[root] : undefined
  if (!root || !options || isEmpty(options)) return [...lines, "No options."].join("\n")

  const queue = [root]
  const seen = new Set(queue)
  for (const name of queue) {
    const schema = props.definitions[name]
    if (!schema) continue
    const table = Boolean(schema.properties && !schema.anyOf)
    if (name !== root) lines.push("", `**\`${name}\`**`, "")
    const summary = schema.description?.startsWith(`\`${props.plugin.name}\`:`)
    if (!summary) {
      const description = joinLines(schema.description)
      if (description) lines.push(description, "")
    }
    if (table) lines.push(...writeTable(props.language, schema))
    else lines.push(`Takes \`${writeLanguageType(props.language, schema)}\`.`)
    for (const reference of listRefs(schema)) {
      if (seen.has(reference)) continue
      seen.add(reference)
      queue.push(reference)
    }
  }
  return lines.join("\n").trimEnd()
}

export function replaceSections(props: {
  text: string
  names: string[]
  render: (name: string) => string
}) {
  const found = new Set<string>()
  const unknown: string[] = []
  const text = props.text.replace(MARKERS, (section, name: string) => {
    found.add(name)
    if (!props.names.includes(name)) {
      unknown.push(name)
      return section
    }
    const rendered = props.render(name)
    return `<!-- generated:options ${name} -->\n\n${rendered}\n\n<!-- /generated -->`
  })
  const missing = props.names.filter(name => !found.has(name))
  return { text, missing, unknown }
}

function isEmpty(schema: Schema) {
  const object = schema.type === "object" && !schema.properties
  return object && !schema.additionalProperties && !schema.anyOf && !schema.oneOf
}

function writeTable(language: Language, schema: Schema) {
  const rows = [["Option", "Type", "Default", "Description"]]
  for (const [key, property] of Object.entries(schema.properties ?? {})) {
    const type = escapeCell(writeLanguageType(language, property))
    const fallback = writeDefault(language, property.default)
    const description = escapeCell(joinLines(property.description))
    rows.push([`\`${key}\``, `\`${type}\``, fallback, description])
  }
  // Padded as `vp fmt` pads a table, or every page would drift from the formatted one
  const widths = [0, 1, 2, 3].map(column =>
    Math.max(3, ...rows.map(row => (row[column] ?? "").length)),
  )
  const writeRow = (cells: string[]) =>
    `| ${cells.map((cell, column) => cell.padEnd(widths[column] ?? 0)).join(" | ")} |`
  const [header = [], ...body] = rows
  return [
    writeRow(header),
    writeRow(widths.map(width => "-".repeat(width))),
    ...body.map(writeRow),
  ]
}

function writeLanguageType(language: Language, schema: Schema) {
  if (language === "python") return writePythonType(schema, "Inline", [])
  return writeType(schema)
}

function writeDefault(language: Language, value: unknown) {
  if (value === undefined) return ""
  if (language !== "python") return `\`${escapeCell(JSON.stringify(value))}\``
  const python = JSON.stringify(value)
    .replace(/\btrue\b/g, "True")
    .replace(/\bfalse\b/g, "False")
    .replace(/\bnull\b/g, "None")
  return `\`${escapeCell(python)}\``
}

export function writeType(schema: Schema | boolean): string {
  if (typeof schema === "boolean") return "unknown"
  if (schema.$ref) return readRef(schema)
  const union = schema.anyOf ?? schema.oneOf
  if (union) return joinUnion(union.map(writeType))
  if (schema.enum) return joinUnion(schema.enum.map(value => JSON.stringify(value)))
  if (schema.const !== undefined) return JSON.stringify(schema.const)
  if (Array.isArray(schema.type)) {
    return joinUnion(schema.type.map(type => writeType({ ...schema, type })))
  }
  switch (schema.type) {
    case "string":
      return "string"
    case "boolean":
      return "boolean"
    case "number":
    case "integer":
      return "number"
    case "null":
      return "null"
    case "array": {
      const item = schema.items ? writeType(schema.items) : "unknown"
      return item.includes(" ") ? `(${item})[]` : `${item}[]`
    }
    case "object": {
      if (schema.properties) {
        const fields = Object.entries(schema.properties).map(([key, property]) => {
          const optional = schema.required?.includes(key) ? "" : "?"
          return `${key}${optional}: ${writeType(property)}`
        })
        return `{ ${fields.join("; ")} }`
      }
      const value = schema.additionalProperties
      return `Record<string, ${value === undefined ? "unknown" : writeType(value)}>`
    }
  }
  return "unknown"
}

export function writePythonType(
  schema: Schema | boolean,
  name: string,
  definitions: string[],
): string {
  if (typeof schema === "boolean") return "Any"
  if (schema.$ref) return readRef(schema)
  const union = schema.anyOf ?? schema.oneOf
  if (union) {
    return joinPythonUnion(
      union.map((item, index) => writePythonType(item, `${name}${index}`, definitions)),
    )
  }
  if (schema.enum)
    return `Literal[${schema.enum.map(value => JSON.stringify(value)).join(", ")}]`
  if (schema.const !== undefined) return `Literal[${JSON.stringify(schema.const)}]`
  if (Array.isArray(schema.type)) {
    return joinPythonUnion(
      schema.type.map(type => writePythonType({ ...schema, type }, name, definitions)),
    )
  }
  switch (schema.type) {
    case "string":
      return "str"
    case "boolean":
      return "bool"
    case "integer":
      return "int"
    case "number":
      return "float"
    case "null":
      return "None"
    case "array":
      return `List[${schema.items ? writePythonType(schema.items, name, definitions) : "Any"}]`
    case "object": {
      if (schema.properties) {
        definitions.push(
          ...writeTypedDict(name, { ...schema, description: undefined }, definitions),
        )
        return name
      }
      const value = schema.additionalProperties
      const item =
        value === undefined ? "Any" : writePythonType(value, `${name}Value`, definitions)
      return `Dict[str, ${item}]`
    }
  }
  return "Any"
}

// A TypedDict is total=False, with required keys in a base of their own: Python 3.9 has
// no `Required`
export function writeTypedDict(
  name: string,
  schema: Schema,
  definitions: string[],
): string[] {
  const properties = Object.entries(schema.properties ?? {})
  const required = properties.filter(([key]) => schema.required?.includes(key))
  const optional = properties.filter(([key]) => !schema.required?.includes(key))
  const field = ([key, property]: [string, Schema]) => [
    ...writePythonComment(property.description, "    "),
    `    ${key}: ${writePythonType(property, name + toPascalCase(key), definitions)}`,
  ]
  const lines: string[] = []
  let base = "TypedDict"
  if (required.length > 0) {
    base = `_${name}Required`
    lines.push(`class ${base}(TypedDict):`, ...required.flatMap(field), "")
  }
  lines.push(...writePythonComment(schema.description, ""))
  lines.push(`class ${name}(${base}, total=False):`)
  lines.push(...(optional.length > 0 ? optional.flatMap(field) : ["    pass"]), "")
  return lines
}

export function writePythonComment(description: string | undefined, indent: string) {
  if (!description) return []
  return description.split("\n").map(line => `${indent}# ${line}`.trimEnd())
}

export function listRefs(schema: Schema | boolean): string[] {
  if (typeof schema === "boolean") return []
  const nested = [
    ...(schema.anyOf ?? []),
    ...(schema.oneOf ?? []),
    ...Object.values(schema.properties ?? {}),
    ...(schema.items ? [schema.items] : []),
    ...(typeof schema.additionalProperties === "object"
      ? [schema.additionalProperties]
      : []),
  ]
  return [...(schema.$ref ? [readRef(schema)] : []), ...nested.flatMap(listRefs)]
}

export function readRef(schema: Schema) {
  return (schema.$ref ?? "").replace("#/$defs/", "")
}

function joinUnion(types: string[]) {
  return [...new Set(types)].join(" | ")
}

function joinPythonUnion(types: string[]) {
  const unique = [...new Set(types)]
  if (unique.length === 1) return unique[0] ?? "Any"
  const rest = unique.filter(type => type !== "None")
  if (rest.length === 1 && unique.includes("None")) return `Optional[${rest[0]}]`
  return `Union[${unique.join(", ")}]`
}

export function toSnakeCase(name: string) {
  return name.replace(/[A-Z]/g, letter => `_${letter.toLowerCase()}`)
}

export function toPascalCase(name: string) {
  return name.charAt(0).toUpperCase() + name.slice(1)
}

function joinLines(text: string | undefined) {
  return (text ?? "").split("\n").join(" ").trim()
}

function escapeCell(text: string) {
  return text.replace(/\|/g, "\\|")
}
