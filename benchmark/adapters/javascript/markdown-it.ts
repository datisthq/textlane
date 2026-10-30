import MarkdownIt from "markdown-it"
import markdownItFootnote from "markdown-it-footnote"
import markdownItTaskLists from "markdown-it-task-lists"
import { runAdapter } from "./common.ts"

const markdownIt = new MarkdownIt({ html: true, linkify: true })
  .use(markdownItFootnote)
  .use(markdownItTaskLists)

runAdapter(markdown => markdownIt.render(markdown))
