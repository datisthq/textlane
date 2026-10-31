import { join } from "node:path"
import ts from "typescript"

// Renders the syntax trees `parse` returns as Python TypedDicts, from the same declarations
// the npm package types them with: @types/unist, mdast, hast and nlcst as typescript/
// depends on them, the mdast utilities that add the nodes textlane's syntax extensions make,
// and estree for MDX's `data.estree`.

/** The Python modules written, each from the package of its name. */
export const TREE_MODULES = ["unist", "mdast", "hast", "nlcst", "estree"] as const

export type TreeModule = (typeof TREE_MODULES)[number]

// Imported for their `declare module` additions to mdast's and hast's content
const EXTENSIONS = [
  "mdast-util-definition-list",
  "mdast-util-directive",
  "mdast-util-math",
  "mdast-util-mdx",
]

const OWNERS: [TreeModule, RegExp][] = [
  ["unist", /\/@types\/unist\//],
  ["mdast", /\/@types\/mdast\//],
  ["hast", /\/@types\/hast\//],
  ["nlcst", /\/@types\/nlcst\//],
  ["estree", /\/@types\/estree(-jsx)?\//],
]

// Keys class syntax cannot declare: Python keywords and the methods of `dict`
const RESERVED = new Set(
  (
    "False None True and as assert async await break class continue def del elif else " +
    "except finally for from global if import in is lambda nonlocal not or pass raise " +
    "return try while with yield clear copy fromkeys get items keys pop popitem " +
    "setdefault update values"
  ).split(" "),
)

interface Declaration {
  name: string
  module: TreeModule
  symbol: ts.Symbol
}

/** Each tree module's Python source, keyed by module name. */
export function renderTrees(props: { root: string; header: string }) {
  // TypeScript asks for files with forward slashes, so a Windows `join` would never match
  const entry = join(props.root, "typescript", "trees.generated.ts").replaceAll("\\", "/")
  const source = [
    ...TREE_MODULES.map(name => `export type * as ${name} from "${name}"`),
    ...EXTENSIONS.map(name => `import type {} from "${name}"`),
  ].join("\n")
  const options: ts.CompilerOptions = {
    strict: true,
    noEmit: true,
    module: ts.ModuleKind.NodeNext,
    moduleResolution: ts.ModuleResolutionKind.NodeNext,
    target: ts.ScriptTarget.ES2022,
    types: [],
  }
  const host = ts.createCompilerHost(options)
  const readFile = host.readFile
  host.readFile = name => (name === entry ? source : readFile(name))
  const fileExists = host.fileExists
  host.fileExists = name => name === entry || fileExists(name)
  const program = ts.createProgram([entry], options, host)
  const checker = program.getTypeChecker()
  const renderer = new Renderer(checker)

  for (const module of TREE_MODULES) {
    if (module === "estree") continue
    const file = program
      .getSourceFiles()
      .find(file => OWNERS.find(([name]) => name === module)?.[1].test(file.fileName))
    const symbol = file && checker.getSymbolAtLocation(file)
    if (!symbol) throw new Error(`no declarations for ${module}`)
    for (const exported of checker.getExportsOfModule(symbol)) {
      // unist's content registries index unions TypeScript alone can read
      if (exported.name.endsWith("Map")) continue
      if (exported.flags & (ts.SymbolFlags.Interface | ts.SymbolFlags.TypeAlias))
        renderer.declare(exported, module)
    }
    renderer.drain(module)
  }
  for (const module of TREE_MODULES) renderer.drain(module)

  const files = new Map<TreeModule, string>()
  for (const module of TREE_MODULES)
    files.set(module, renderer.write(module, props.header))
  return files
}

class Renderer {
  #checker: ts.TypeChecker
  #declarations = new Map<ts.Symbol, Declaration>()
  #names = new Map<string, ts.Symbol>()
  #pending: Declaration[] = []
  #classes = new Map<TreeModule, string[][]>()
  #aliases = new Map<TreeModule, { name: string; value: string; uses: string[] }[]>()
  #imports = new Map<TreeModule, Set<TreeModule>>()
  #module: TreeModule = "unist"

  constructor(checker: ts.TypeChecker) {
    this.#checker = checker
  }

  declare(symbol: ts.Symbol, fallback: TreeModule) {
    const known = this.#declarations.get(symbol)
    if (known) return known
    const file = symbol.declarations?.[0]?.getSourceFile().fileName ?? ""
    const module = OWNERS.find(([, pattern]) => pattern.test(file))?.[0] ?? fallback
    const declaration = { name: symbol.name, module, symbol }
    const key = `${module}.${symbol.name}`
    const other = this.#names.get(key)
    if (other && other !== symbol) throw new Error(`two declarations named ${key}`)
    this.#names.set(key, symbol)
    this.#declarations.set(symbol, declaration)
    this.#pending.push(declaration)
    return declaration
  }

  // Renders the module's declarations, leaving other modules' for their own turn, so a
  // node an extension shares between mdast and hast (a JSX attribute) lands in mdast
  drain(module: TreeModule) {
    for (;;) {
      const index = this.#pending.findIndex(declaration => declaration.module === module)
      if (index < 0) return
      const [declaration] = this.#pending.splice(index, 1)
      if (!declaration) return
      this.#module = module
      this.#render(declaration)
    }
  }

  write(module: TreeModule, header: string) {
    const imports = [...(this.#imports.get(module) ?? [])].filter(name => name !== module)
    const aliases = sortAliases(this.#aliases.get(module) ?? [])
    const lines = [
      `# ${header}`,
      `"""The ${module} syntax tree as TypedDicts, named as @types/${module} names it."""`,
      "",
      "from __future__ import annotations",
      "",
      "import typing",
      // mdast and hast name each other (hast properties in mdast's data, JSX attributes in
      // hast), so modules import one another for the checker only
      ...(imports.length > 0
        ? [
            "",
            "if typing.TYPE_CHECKING:",
            ...imports.sort().map(name => `    from . import ${name} as _${name}`),
          ]
        : []),
      "",
      "",
      ...(this.#classes.get(module) ?? []).flatMap(block => [...block, "", ""]),
      ...aliases.map(alias => `${alias.name} = ${alias.value}`),
    ]
    return `${lines.join("\n").trimEnd()}\n`
  }

  #render(declaration: Declaration) {
    const { symbol, name } = declaration
    const checker = this.#checker
    if (symbol.flags & ts.SymbolFlags.TypeAlias) {
      const uses: string[] = []
      const type = checker.getDeclaredTypeOfSymbol(symbol)
      const value = this.#type(type, {
        standalone: true,
        uses,
        self: symbol,
        quote: true,
      })
      this.#push(this.#aliases, declaration.module, { name, value, uses })
      return
    }
    const type = checker.getDeclaredTypeOfSymbol(symbol)
    const index = checker.getIndexInfoOfType(type, ts.IndexKind.String)
    if (index) {
      const uses: string[] = []
      const values = this.#type(index.type, { standalone: false, uses, quote: true })
      const value = `typing.Dict[str, ${values}]`
      this.#push(this.#aliases, declaration.module, { name, value, uses })
      return
    }
    const required: string[] = []
    const optional: string[] = []
    for (const property of checker.getPropertiesOfType(type)) {
      const declared = checker.getTypeOfSymbol(property)
      if (declared.getCallSignatures().length > 0) continue
      const value = this.#type(declared, { standalone: true, uses: [] })
      const field: [string, string] = [property.name, value]
      if (property.flags & ts.SymbolFlags.Optional) optional.push(field.join(": "))
      else required.push(field.join(": "))
    }
    const doc = writeDocstring(
      ts.displayPartsToString(symbol.getDocumentationComment(checker)),
    )
    this.#push(
      this.#classes,
      declaration.module,
      writeClass({ name, doc, required, optional }),
    )
  }

  #type(
    type: ts.Type,
    context: {
      standalone: boolean
      uses: string[]
      self?: ts.Symbol
      quote?: boolean
    },
  ): string {
    const checker = this.#checker
    const alias = type.aliasSymbol
    if (alias && alias !== context.self && !type.aliasTypeArguments && isOwn(alias)) {
      return this.#reference(alias, context)
    }
    if (type.isUnion()) return this.#union(type.types, context)
    if (type.flags & ts.TypeFlags.String) return "str"
    if (type.flags & ts.TypeFlags.Number) return context.standalone ? "int" : "float"
    if (type.flags & ts.TypeFlags.BigInt) return "int"
    if (type.flags & ts.TypeFlags.Boolean) return "bool"
    if (type.flags & ts.TypeFlags.Null) return "None"
    if (type.isStringLiteral()) return `typing.Literal[${JSON.stringify(type.value)}]`
    if (type.isNumberLiteral()) return `typing.Literal[${type.value}]`
    if (type.flags & ts.TypeFlags.BooleanLiteral)
      return `typing.Literal[${checker.typeToString(type) === "true" ? "True" : "False"}]`
    if (type.flags & (ts.TypeFlags.Any | ts.TypeFlags.Unknown)) return "object"
    if (type.isIntersection()) {
      if (type.types.some(member => member.flags & ts.TypeFlags.String)) return "str"
      return "object"
    }
    if (checker.isArrayType(type)) {
      const [element] = checker.getTypeArguments(type as ts.TypeReference)
      if (!element) return "typing.List[object]"
      return `typing.List[${this.#type(element, context)}]`
    }
    if (checker.isTupleType(type)) {
      const elements = checker.getTypeArguments(type as ts.TypeReference)
      return `typing.Tuple[${elements.map(element => this.#type(element, context)).join(", ")}]`
    }
    const symbol = type.getSymbol()
    if (symbol && symbol !== context.self && symbol.flags & ts.SymbolFlags.Interface) {
      if (isOwn(symbol)) return this.#reference(symbol, context)
    }
    return "object"
  }

  #union(
    types: ts.Type[],
    context: {
      standalone: boolean
      uses: string[]
      self?: ts.Symbol
      quote?: boolean
    },
  ) {
    const present = types.filter(member => !(member.flags & ts.TypeFlags.Undefined))
    const nullable = present.some(member => member.flags & ts.TypeFlags.Null)
    const values = present.filter(member => !(member.flags & ts.TypeFlags.Null))
    const booleans = values.filter(member => member.flags & ts.TypeFlags.BooleanLiteral)
    const rest =
      booleans.length === 2 ? values.filter(m => !booleans.includes(m)) : values
    const standalone =
      context.standalone && rest.length + (booleans.length === 2 ? 1 : 0) === 1
    const literals: string[] = []
    const parts: string[] = booleans.length === 2 ? ["bool"] : []
    for (const member of rest) {
      const text = this.#type(member, { ...context, standalone })
      const literal = /^typing\.Literal\[(.*)\]$/.exec(text)
      if (literal?.[1]) literals.push(literal[1])
      else if (!parts.includes(text)) parts.push(text)
    }
    if (literals.length > 0 && !parts.includes("str"))
      parts.unshift(`typing.Literal[${literals.join(", ")}]`)
    let value = parts.includes("object")
      ? "object"
      : parts.length === 1
        ? (parts[0] ?? "object")
        : `typing.Union[${parts.join(", ")}]`
    if (nullable && value !== "object") value = `typing.Optional[${value}]`
    return value
  }

  #reference(symbol: ts.Symbol, context: { uses: string[]; quote?: boolean }) {
    const declaration = this.declare(symbol, this.#module)
    if (declaration.module === this.#module) {
      context.uses.push(declaration.name)
      return declaration.name
    }
    const imports = this.#imports.get(this.#module) ?? new Set()
    imports.add(declaration.module)
    this.#imports.set(this.#module, imports)
    // underscored, or MDX's `estree` field would shadow the module in its class body
    const name = `_${declaration.module}.${declaration.name}`
    // an alias is evaluated at import, when the module it names is not imported
    return context.quote ? JSON.stringify(name) : name
  }

  #push<Item>(map: Map<TreeModule, Item[]>, module: TreeModule, item: Item) {
    const items = map.get(module) ?? []
    items.push(item)
    map.set(module, items)
  }
}

// A local `type` in an extension package is expanded rather than named: only what the
// package exports, or adds to a module, is part of the tree's vocabulary
function isOwn(symbol: ts.Symbol) {
  const declaration = symbol.declarations?.[0]
  if (!declaration) return false
  const file = declaration.getSourceFile()
  if (file.hasNoDefaultLib || /\/typescript\/lib\//.test(file.fileName)) return false
  if (ts.getCombinedModifierFlags(declaration) & ts.ModifierFlags.Export) return true
  return ts.isModuleBlock(declaration.parent)
}

function writeClass(props: {
  name: string
  doc: string[]
  required: string[]
  optional: string[]
}) {
  const fields = [...props.required, ...props.optional]
  const reserved = fields.some(field => RESERVED.has(field.split(":")[0] ?? ""))
  const body = (lines: string[]) => [
    ...props.doc.map(line => `    ${line}`),
    ...(props.doc.length > 0 && lines.length > 0 ? [""] : []),
    ...lines.map(line => `    ${line}`),
    ...(props.doc.length === 0 && lines.length === 0 ? ["    pass"] : []),
  ]
  // estree's `async`, `await` and `update`
  if (reserved) {
    const write = (name: string, items: string[], total: boolean) =>
      `${name} = typing.TypedDict("${name}", {${items
        .map(item => {
          // the functional form evaluates its values: quoted, they stay forward references, in
          // single quotes since a checker refuses escapes in one
          const [key, ...value] = item.split(": ")
          const text = value.join(": ")
          if (text.includes("'")) throw new Error(`cannot quote ${text}`)
          return `"${key}": '${text}'`
        })
        .join(", ")}}${total ? "" : ", total=False"})`
    return [
      write(`_${props.name}Required`, props.required, true),
      write(`_${props.name}Optional`, props.optional, false),
      "",
      "",
      `class ${props.name}(_${props.name}Required, _${props.name}Optional):`,
      ...body([]),
    ]
  }
  if (props.required.length > 0 && props.optional.length > 0) {
    return [
      `class _${props.name}Required(typing.TypedDict):`,
      ...props.required.map(line => `    ${line}`),
      "",
      "",
      `class ${props.name}(_${props.name}Required, total=False):`,
      ...body(props.optional),
    ]
  }
  if (props.required.length > 0)
    return [`class ${props.name}(typing.TypedDict):`, ...body(props.required)]
  return [`class ${props.name}(typing.TypedDict, total=False):`, ...body(props.optional)]
}

function writeDocstring(text: string) {
  const paragraph =
    text
      .split(/\n\s*\n/)[0]
      ?.replace(/\s+/g, " ")
      .trim() ?? ""
  if (!paragraph) return []
  const safe = paragraph.replace(/\\/g, "\\\\").replace(/"""/g, '\\"\\"\\"')
  const lines: string[] = []
  let line = ""
  for (const word of `"""${safe}"""`.split(" ")) {
    if (line && line.length + word.length + 1 > 84) {
      lines.push(line)
      line = word
    } else line = line ? `${line} ${word}` : word
  }
  lines.push(line)
  return lines
}

// Aliases are evaluated as the module loads, so each comes after those it names
function sortAliases(aliases: { name: string; value: string; uses: string[] }[]) {
  const byName = new Map(aliases.map(alias => [alias.name, alias]))
  const sorted: typeof aliases = []
  const seen = new Set<string>()
  const visit = (alias: (typeof aliases)[number]) => {
    if (seen.has(alias.name)) return
    seen.add(alias.name)
    for (const use of alias.uses) {
      const target = byName.get(use)
      if (target) visit(target)
    }
    sorted.push(alias)
  }
  for (const alias of aliases) visit(alias)
  return sorted
}
