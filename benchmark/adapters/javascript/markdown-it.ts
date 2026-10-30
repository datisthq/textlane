import MarkdownIt from "markdown-it"
import markdownItFootnote from "markdown-it-footnote"
import markdownItTaskLists from "markdown-it-task-lists"
import { isGfm, runAdapter } from "./common.ts"

const markdownIt = isGfm
  ? new MarkdownIt({ html: true, linkify: true })
      .use(markdownItFootnote)
      .use(markdownItTaskLists)
  : new MarkdownIt("commonmark")

runAdapter(markdown => markdownIt.render(markdown))
