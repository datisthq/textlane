import { micromark } from "micromark"
import { gfm, gfmHtml } from "micromark-extension-gfm"
import { isGfm, runAdapter } from "./common.ts"

const options = {
  allowDangerousHtml: true,
  allowDangerousProtocol: true,
  extensions: isGfm ? [gfm()] : [],
  htmlExtensions: isGfm ? [gfmHtml()] : [],
}

runAdapter(markdown => micromark(markdown, options))
