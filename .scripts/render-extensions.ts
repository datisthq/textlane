import { readFileSync } from "node:fs"
import { createRequire } from "node:module"
import { dirname, join } from "node:path"
import ts from "typescript"

// Copies the declarations of the mdast utilities whose nodes textlane's syntax extensions
// make, so the npm package types those nodes without depending on the utilities' code

/** Each copied package, or hand-written source, and the declaration file it is read from. */
export const EXTENSIONS = [
  { name: "mdast-util-definition-list", file: "lib/types.d.ts" },
  { name: "mdast-util-directive", file: "index.d.ts" },
  { name: "mdast-util-math", file: "index.d.ts" },
  { name: "mdast-util-mdx-expression", file: "index.d.ts", via: "mdast-util-mdx" },
  { name: "mdast-util-mdx-jsx", file: "index.d.ts", via: "mdast-util-mdx" },
  { name: "mdast-util-mdxjs-esm", file: "index.d.ts", via: "mdast-util-mdx" },
  // hast's `raw` node and the `hName` family on mdast's data
  { name: "mdast-util-to-hast", file: "index.d.ts", via: "mdast-util-definition-list" },
] as const

// remark-frontmatter parses TOML but no package declares its node, so textlane does
const TOML = `import type { Data, Literal } from "mdast"

/** TOML frontmatter, as remark-frontmatter parses it. */
export interface Toml extends Literal {
  /** Node type. */
  type: "toml"
  /** Data associated with the mdast TOML node. */
  data?: TomlData | undefined
}

/** Info associated with mdast TOML nodes by the ecosystem. */
export interface TomlData extends Data {}

declare module "mdast" {
  interface FrontmatterContentMap {
    /** TOML frontmatter. */
    toml: Toml
  }

  interface RootContentMap {
    /** TOML frontmatter. */
    toml: Toml
  }
}
`

// Only the trees are copied: augmentations of the utilities' own markdown state would name
// modules the npm package does not depend on
const TREES = new Set(["mdast", "hast"])

/** Each extension's TypeScript source, keyed by module name (`mdast-util-math`). */
export function renderExtensions(props: { root: string; header: string }) {
  const require = createRequire(join(props.root, "package.json"))
  const files = new Map<string, string>()
  for (const extension of EXTENSIONS) {
    const from =
      "via" in extension ? createRequire(require.resolve(extension.via)) : require
    const entry = from.resolve(extension.name)
    const path = join(dirname(entry), extension.file)
    files.set(extension.name, copyDeclarations(readFileSync(path, "utf8"), props.header))
  }
  files.set("toml", `// ${props.header}\n\n${TOML}`)
  return files
}

function copyDeclarations(text: string, header: string) {
  const source = ts.createSourceFile("index.d.ts", text, ts.ScriptTarget.ES2022, true)
  const constants = new Map<string, string>()
  const kept: string[] = []
  for (const statement of source.statements) {
    // re-exports and relative imports reach the utility's code: its options, markdown state
    if (ts.isExportDeclaration(statement)) continue
    const specifier = ts.isImportDeclaration(statement) && statement.moduleSpecifier
    if (specifier && ts.isStringLiteral(specifier) && specifier.text.startsWith("."))
      continue
    if (ts.isModuleDeclaration(statement) && !TREES.has(statement.name.text)) continue
    // `export declare const DefList = "defList"` would be a value the package lacks
    if (ts.isVariableStatement(statement)) {
      for (const declaration of statement.declarationList.declarations) {
        const value = declaration.initializer
        if (ts.isIdentifier(declaration.name) && value && ts.isStringLiteral(value))
          constants.set(declaration.name.text, JSON.stringify(value.text))
      }
      continue
    }
    kept.push(statement.getFullText(source).replace(/^\s*\n/, ""))
  }
  const body = kept
    .join("\n")
    .replace(/typeof (\w+)/g, (match, name: string) => constants.get(name) ?? match)
  return `// ${header}\n\n${body.trimEnd()}\n`
}
