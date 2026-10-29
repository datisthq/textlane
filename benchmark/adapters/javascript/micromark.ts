import { micromark } from "micromark"
import { runAdapter } from "./common.ts"

runAdapter(markdown =>
  micromark(markdown, { allowDangerousHtml: true, allowDangerousProtocol: true }),
)
