import MarkdownIt from "markdown-it"
import { runAdapter } from "./common.ts"

const markdownIt = new MarkdownIt("commonmark")

runAdapter(markdown => markdownIt.render(markdown))
