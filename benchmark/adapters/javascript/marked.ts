import { Marked } from "marked"
import markedFootnote from "marked-footnote"
import { runAdapter } from "./common.ts"

const marked = new Marked({ gfm: true, async: false }).use(markedFootnote())

runAdapter(markdown => {
  const html = marked.parse(markdown)
  if (typeof html !== "string") throw new Error("marked returned a promise")
  return html
})
