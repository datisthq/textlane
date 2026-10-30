import * as html from "../../../typescript/generated/html.ts"
import * as md from "../../../typescript/generated/md.ts"
import { textlane } from "../../../typescript/index.ts"
import { isGfm, runAdapter } from "./common.ts"

const parser = textlane().use(md.parse())
const processor = (isGfm ? parser.use(md.gfm()) : parser)
  .use(md.toHtml({ allowDangerousHtml: true }))
  .use(html.stringify({ allowDangerousHtml: true }))
  .freeze()

runAdapter(markdown => processor.processSync(markdown).value)
