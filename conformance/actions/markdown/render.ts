import rehypeStringify from "rehype-stringify"
import remarkParse from "remark-parse"
import remarkRehype from "remark-rehype"
import { unified } from "unified"

/** The textlane pipeline a Markdown case runs, as data, for the given HTML handling. */
export function createPipeline(props: { allowDangerousHtml: boolean }) {
  if (!props.allowDangerousHtml) return [["md.parse"], ["md.toHtml"], ["html.stringify"]]
  return [
    ["md.parse"],
    ["md.toHtml", { allowDangerousHtml: true }],
    ["html.stringify", { allowDangerousHtml: true }],
  ]
}

/** Render Markdown to HTML through the pinned unified packages that pipeline mirrors. */
export function renderMarkdown(props: { markdown: string; allowDangerousHtml: boolean }) {
  const options = { allowDangerousHtml: props.allowDangerousHtml }
  const processor = unified()
    .use(remarkParse)
    .use(remarkRehype, options)
    .use(rehypeStringify, options)
  return String(processor.processSync(props.markdown))
}
