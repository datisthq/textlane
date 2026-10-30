// neither plugin ships types for markdown-it 15, and @types/markdown-it-footnote binds 14's
declare module "markdown-it-footnote" {
  import type { MarkdownIt } from "markdown-it"
  export default function markdownItFootnote(md: MarkdownIt): void
}

declare module "markdown-it-task-lists" {
  import type { MarkdownIt } from "markdown-it"
  export default function markdownItTaskLists(md: MarkdownIt): void
}
