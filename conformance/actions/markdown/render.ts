import rehypeParse from "rehype-parse"
import rehypeStringify from "rehype-stringify"
import { directiveToMarkdown } from "mdast-util-directive"
import remarkDefinitionList, { defListHastHandlers } from "remark-definition-list"
import remarkDirective from "remark-directive"
import remarkFrontmatter from "remark-frontmatter"
import type { Options as FrontmatterOptions } from "remark-frontmatter"
import remarkGfm from "remark-gfm"
import remarkMath from "remark-math"
import remarkParse from "remark-parse"
import remarkRehype from "remark-rehype"
import remarkStringify from "remark-stringify"
import { unified } from "unified"

/** How a Markdown case is rendered: raw HTML handling, GFM and `md.toHtml` options. */
export type RenderOptions = {
  allowDangerousHtml: boolean
  // `null` enables a plugin whose options are not an object with its defaults
  frontmatter?: FrontmatterOptions | null
  gfm?: Record<string, unknown>
  math?: Record<string, unknown>
  definitionList?: boolean
  toHtml?: Record<string, unknown>
}

/** The textlane pipeline a Markdown case runs, as data. */
export function createPipeline(props: RenderOptions) {
  const toHtml = {
    ...props.toHtml,
    ...(props.allowDangerousHtml ? { allowDangerousHtml: true } : {}),
  }
  return [
    ["md.parse"],
    ...(props.frontmatter !== undefined
      ? [withOptions("md.frontmatter", props.frontmatter)]
      : []),
    ...(props.gfm ? [withOptions("md.gfm", props.gfm)] : []),
    ...(props.math ? [withOptions("md.math", props.math)] : []),
    ...(props.definitionList ? [["md.definitionList"]] : []),
    withOptions("md.toHtml", toHtml),
    withOptions(
      "html.stringify",
      props.allowDangerousHtml ? { allowDangerousHtml: true } : {},
    ),
  ]
}

/** Render Markdown to HTML through the pinned unified packages that pipeline mirrors. */
export function renderMarkdown(props: RenderOptions & { markdown: string }) {
  const html = { allowDangerousHtml: props.allowDangerousHtml }
  const processor = unified()
    .use(remarkParse)
    .use(
      props.frontmatter !== undefined
        ? [[remarkFrontmatter, props.frontmatter ?? undefined]]
        : [],
    )
    .use(props.gfm ? [[remarkGfm, props.gfm]] : [])
    .use(props.math ? [[remarkMath, props.math]] : [])
    .use(props.definitionList ? [remarkDefinitionList] : [])
    // remark-definition-list renders through remark-rehype handlers it exports, not by itself
    .use(remarkRehype, {
      ...props.toHtml,
      ...html,
      ...(props.definitionList ? { handlers: { ...defListHastHandlers } } : {}),
    })
    .use(rehypeStringify, html)
  return String(processor.processSync(props.markdown))
}

/** Syntax extensions a tree case parses or serializes with, and their options. */
export type Extensions = {
  gfm?: Record<string, unknown>
  directive?: Record<string, unknown>
  math?: Record<string, unknown>
  frontmatter?: FrontmatterOptions | null
}

/** Parse Markdown to mdast, with positions, through the pinned remark-parse and extensions. */
export function parseMarkdown(markdown: string, extensions: Extensions = {}): object {
  const processor = unified()
    .use(remarkParse)
    .use(extensions.gfm ? [[remarkGfm, extensions.gfm]] : [])
    .use(extensions.directive ? [remarkDirective] : [])
    .use(extensions.math ? [[remarkMath, extensions.math]] : [])
    .use(
      extensions.frontmatter !== undefined
        ? [[remarkFrontmatter, extensions.frontmatter ?? undefined]]
        : [],
    )
  return JSON.parse(JSON.stringify(processor.parse(markdown)))
}

/** Serialize mdast to Markdown through the pinned remark-stringify and extensions. */
export function stringifyTree(
  props: Extensions & { tree: object; stringify?: Record<string, unknown> },
) {
  const processor = unified()
    .use(remarkStringify, props.stringify ?? {})
    .use(props.gfm ? [[remarkGfm, props.gfm]] : [])
    .use(props.math ? [[remarkMath, props.math]] : [])
    .use(
      props.frontmatter !== undefined
        ? [[remarkFrontmatter, props.frontmatter ?? undefined]]
        : [],
    )
  // remark-directive takes no options, so its serializer is registered here with them
  if (props.directive) {
    const extensions = processor.data("toMarkdownExtensions") ?? []
    processor.data("toMarkdownExtensions", [
      ...extensions,
      directiveToMarkdown(props.directive),
    ])
  }
  // A JSON tree is a valid mdast root; unified types roots, not plain objects
  return processor.stringify(JSON.parse(JSON.stringify(props.tree)))
}

function withOptions(name: string, options: unknown) {
  const empty =
    options === null || (typeof options === "object" && Object.keys(options).length === 0)
  return empty ? [name] : [name, options]
}

/** Parse HTML to hast, with positions, through the pinned rehype-parse. */
export function parseHtml(html: string, options: Record<string, unknown> = {}): object {
  return JSON.parse(JSON.stringify(unified().use(rehypeParse, options).parse(html)))
}

/** Parse and serialize HTML through the pinned rehype-parse and rehype-stringify. */
export function processHtml(props: {
  html: string
  parse?: Record<string, unknown>
  stringify?: Record<string, unknown>
}) {
  const processor = unified()
    .use(rehypeParse, props.parse ?? {})
    .use(rehypeStringify, props.stringify ?? {})
  return String(processor.processSync(props.html))
}

/** The parse errors rehype-parse reports, as a case's `expected.messages`. */
export function collectParseErrors(html: string, options: Record<string, unknown> = {}) {
  const file = unified()
    .use(rehypeParse, { ...options, emitParseErrors: true })
    .use(rehypeStringify)
    .processSync(html)
  return file.messages.map(message => ({
    ruleId: message.ruleId ?? "",
    source: message.source ?? "",
    place: JSON.parse(JSON.stringify(message.place ?? null)),
  }))
}
