import { micromark } from "micromark"
import { gfm, gfmHtml } from "micromark-extension-gfm"
import { runAdapter } from "./common.ts"

const options = {
  allowDangerousHtml: true,
  allowDangerousProtocol: true,
  extensions: [gfm()],
  htmlExtensions: [gfmHtml()],
}

runAdapter(markdown => micromark(markdown, options))
