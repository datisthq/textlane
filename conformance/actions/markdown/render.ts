import rehypeStringify from "rehype-stringify"
import { directiveToMarkdown } from "mdast-util-directive"
import remarkDirective from "remark-directive"
import remarkGfm from "remark-gfm"
import remarkMath from "remark-math"
import remarkParse from "remark-parse"
import remarkRehype from "remark-rehype"
import remarkStringify from "remark-stringify"
import { unified } from "unified"

/** How a Markdown case is rendered: raw HTML handling, GFM and `md.toHtml` options. */
export type RenderOptions = {
  allowDangerousHtml: boolean
  gfm?: Record<string, unknown>
  math?: Record<string, unknown>
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
    ...(props.gfm ? [withOptions("md.gfm", props.gfm)] : []),
    ...(props.math ? [withOptions("md.math", props.math)] : []),
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
    .use(props.gfm ? [[remarkGfm, props.gfm]] : [])
    .use(props.math ? [[remarkMath, props.math]] : [])
    .use(remarkRehype, { ...props.toHtml, ...html })
    .use(rehypeStringify, html)
  return String(processor.processSync(props.markdown))
}

/** Syntax extensions a tree case parses or serializes with, and their options. */
export type Extensions = {
  gfm?: Record<string, unknown>
  directive?: Record<string, unknown>
  math?: Record<string, unknown>
}

/** Parse Markdown to mdast, with positions, through the pinned remark-parse and extensions. */
export function parseMarkdown(markdown: string, extensions: Extensions = {}): object {
  const processor = unified()
    .use(remarkParse)
    .use(extensions.gfm ? [[remarkGfm, extensions.gfm]] : [])
    .use(extensions.directive ? [remarkDirective] : [])
    .use(extensions.math ? [[remarkMath, extensions.math]] : [])
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

function withOptions(name: string, options: Record<string, unknown>) {
  return Object.keys(options).length === 0 ? [name] : [name, options]
}
