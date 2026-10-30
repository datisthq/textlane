import * as html from "../../../typescript/generated/html.ts"
import * as md from "../../../typescript/generated/md.ts"
import { textlane } from "../../../typescript/index.ts"
import { runAdapter } from "./common.ts"

const processor = textlane()
  .use(md.parse())
  .use(md.gfm())
  .use(md.toHtml({ allowDangerousHtml: true }))
  .use(html.stringify({ allowDangerousHtml: true }))
  .freeze()

runAdapter(markdown => processor.processSync(markdown).value)
