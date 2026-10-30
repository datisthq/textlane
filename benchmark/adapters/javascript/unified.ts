import rehypeStringify from "rehype-stringify"
import remarkGfm from "remark-gfm"
import remarkParse from "remark-parse"
import remarkRehype from "remark-rehype"
import { unified } from "unified"
import { isGfm, runAdapter } from "./common.ts"

const parser = unified().use(remarkParse)
const processor = (isGfm ? parser.use(remarkGfm) : parser)
  .use(remarkRehype, { allowDangerousHtml: true })
  .use(rehypeStringify, { allowDangerousHtml: true })
  .freeze()

runAdapter(markdown => String(processor.processSync(markdown)))
