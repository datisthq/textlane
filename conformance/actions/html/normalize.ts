import { fromHtml } from "hast-util-from-html"
import { toHtml } from "hast-util-to-html"

/** Reparse HTML so two renderings compare by meaning, as upstream's own tests compare them. */
export function normalizeHtml(html: string) {
  return toHtml(fromHtml(html.replace(/\n$/, ""), { fragment: true }))
}
