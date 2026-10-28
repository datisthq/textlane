import rehypeStringify from "rehype-stringify"
import remarkGfm from "remark-gfm"
import remarkParse from "remark-parse"
import remarkRehype from "remark-rehype"
import { unified } from "unified"

/** How a Markdown case is rendered: raw HTML handling, GFM and `md.toHtml` options. */
export type RenderOptions = {
  allowDangerousHtml: boolean
  gfm?: Record<string, unknown>
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
    .use(remarkRehype, { ...props.toHtml, ...html })
    .use(rehypeStringify, html)
  return String(processor.processSync(props.markdown))
}

/** Parse Markdown to mdast, with positions, through the pinned remark-parse. */
export function parseMarkdown(markdown: string): object {
  return JSON.parse(JSON.stringify(unified().use(remarkParse).parse(markdown)))
}

function withOptions(name: string, options: Record<string, unknown>) {
  return Object.keys(options).length === 0 ? [name] : [name, options]
}
