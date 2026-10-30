import { Marked } from "marked"
import markedFootnote from "marked-footnote"
import { isGfm, runAdapter } from "./common.ts"

const marked = new Marked({ gfm: isGfm, async: false })
if (isGfm) marked.use(markedFootnote())

runAdapter(markdown => {
  const html = marked.parse(markdown)
  if (typeof html !== "string") throw new Error("marked returned a promise")
  return html
})
