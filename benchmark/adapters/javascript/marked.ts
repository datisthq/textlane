import { Marked } from "marked"
import { runAdapter } from "./common.ts"

const marked = new Marked({ gfm: false, async: false })

runAdapter(markdown => {
  const html = marked.parse(markdown)
  if (typeof html !== "string") throw new Error("marked returned a promise")
  return html
})
