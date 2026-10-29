import type { Random } from "../generate/random.ts"
import type { Profile } from "./types.ts"

const WORDS = (
  "the pipeline parser tree node plugin stage text document markdown html syntax " +
  "content render output input value option format source table list item quote " +
  "fast native rust binding module package release version stream buffer string " +
  "every each one many first last small large simple common nested inline block"
).split(" ")

const LANGUAGES = ["js", "rust", "python", "sh", "json", ""]

/** A README-like document: headings, prose with inline markup, lists, quotes and code. */
export const proseProfile: Profile = {
  name: "prose",
  description: "README-like CommonMark: headings, rich paragraphs, lists, quotes, code",
  version: 1,
  createBlock: (random, index) => {
    if (index % 12 === 0)
      return `${"#".repeat(random.integer(1, 3))} ${createWords(random, 4)}\n\n`
    const kind = random.next()
    if (kind < 0.5) return `${createParagraph(random)}\n\n`
    if (kind < 0.7) return createList(random)
    if (kind < 0.8) return `> ${createParagraph(random).replaceAll("\n", "\n> ")}\n\n`
    if (kind < 0.95) return createCode(random)
    return "---\n\n"
  },
}

function createParagraph(random: Random) {
  const sentences: string[] = []
  for (let index = random.integer(2, 6); index > 0; index--) {
    const words = createWords(random, random.integer(6, 16)).split(" ")
    const at = random.integer(0, words.length - 1)
    const word = words[at] ?? ""
    const kind = random.next()
    if (kind < 0.2) words[at] = `*${word}*`
    else if (kind < 0.35) words[at] = `**${word}**`
    else if (kind < 0.5) words[at] = `\`${word}()\``
    else if (kind < 0.65) words[at] = `[${word}](https://example.com/${word}?id=${at})`
    else if (kind < 0.7) words[at] = `${word} &amp; ${word}`
    const sentence = words.join(" ")
    sentences.push(`${sentence.charAt(0).toUpperCase()}${sentence.slice(1)}.`)
  }
  // a hard wrap every few sentences, as hand-written Markdown has
  return sentences.join(random.next() < 0.3 ? "\n" : " ")
}

function createList(random: Random) {
  const ordered = random.next() < 0.3
  const lines: string[] = []
  for (let index = 1; index <= random.integer(2, 6); index++) {
    const marker = ordered ? `${index}.` : "-"
    lines.push(`${marker} ${createWords(random, random.integer(3, 10))}`)
    if (random.next() < 0.2) {
      lines.push(`${" ".repeat(marker.length + 1)}- ${createWords(random, 4)}`)
    }
  }
  return `${lines.join("\n")}\n\n`
}

function createCode(random: Random) {
  const lines: string[] = []
  for (let index = random.integer(2, 8); index > 0; index--) {
    lines.push(
      `${"  ".repeat(random.integer(0, 2))}${createWords(random, 5)} = <${index}>;`,
    )
  }
  return `\`\`\`${random.pick(LANGUAGES)}\n${lines.join("\n")}\n\`\`\`\n\n`
}

function createWords(random: Random, count: number) {
  return Array.from({ length: count }, () => random.pick(WORDS)).join(" ")
}
