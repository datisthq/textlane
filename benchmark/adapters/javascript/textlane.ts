import * as html from "../../../typescript/generated/html.ts"
import * as md from "../../../typescript/generated/md.ts"
import { textlane } from "../../../typescript/index.ts"
import { isGfm, runAdapter } from "./common.ts"

// What any HTML plugin forces, as a no-op: a non-default option takes the hast path
const quote = process.env.TEXTLANE_HAST ? { quote: '"' as const } : {}
const parser = textlane().use(md.parse())
const processor = (isGfm ? parser.use(md.gfm()) : parser)
  .use(md.toHtml({ allowDangerousHtml: true }))
  .use(html.stringify({ allowDangerousHtml: true, ...quote }))
  .freeze()

runAdapter(markdown => processor.processSync(markdown).value)
