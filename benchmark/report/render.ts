import { formatBytes } from "../generate/size.ts"
import type { Result } from "../harness/measure.ts"
import { getProfile } from "../profiles/index.ts"

export type Machine = {
  cpu: string
  cpus: number
  memory: string
  os: string
  memoryMax?: string
}

export type Report = {
  measuredAt: string
  machine: Machine
  results: Result[]
}

type Group = {
  caseName: string
  profile: string
  bytes: number
  results: Result[]
}

const LANGUAGES = { rust: "Rust", typescript: "Node.js", python: "Python" }

export function renderReport(report: Report) {
  const groups = groupResults(report.results)
  const rivals = new Set(
    report.results.filter(result => !result.isTextlane).map(result => result.tool),
  ).size

  return `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Markdown Benchmark</title>
<style>${STYLE}</style>
</head>
<body>
<main>
<header>
  <p class="eyebrow">Textlane benchmark</p>
  <h1>How fast does Markdown become HTML?</h1>
  <p class="lede">Textlane from Rust, Node.js and Python against ${rivals} other Markdown
  libraries, each converting the same generated documents to HTML. Measured
  ${escapeHtml(report.measuredAt.slice(0, 10))} on ${escapeHtml(describeMachine(report.machine))}.</p>
</header>
${groups.map(renderGroup).join("\n")}
${renderTools(report.results)}
${renderMethod()}
</main>
</body>
</html>
`
}

function groupResults(results: Result[]) {
  const groups = new Map<string, Group>()
  for (const result of results) {
    const key = `${result.case} ${result.size}`
    const group = groups.get(key) ?? {
      caseName: result.case,
      profile: result.profile,
      bytes: result.bytes,
      results: [],
    }
    group.results.push(result)
    groups.set(key, group)
  }
  return [...groups.values()].toSorted(
    (left, right) =>
      left.caseName.localeCompare(right.caseName) || left.bytes - right.bytes,
  )
}

function renderGroup(group: Group) {
  const results = group.results.toSorted(compareResults)
  const maxTime = Math.max(
    ...results.map(result => result.documentMs?.median ?? 0),
    0.001,
  )
  const maxMemory = Math.max(...results.map(result => result.peakMemoryMb ?? 0), 1)
  const description = getProfile(group.profile).description

  return `<section>
  <h2>${escapeHtml(group.caseName)} · ${formatBytes(group.bytes)}</h2>
  <p class="caption">${escapeHtml(description)} · median of ${results[0]?.runs || 1} run(s) · shorter is better</p>
  <div class="legend"><span><i class="swatch textlane"></i>Textlane</span><span><i class="swatch rival"></i>Other libraries</span></div>
  <div class="charts">
    ${renderBars("Time per document", results, result => result.documentMs?.median, maxTime, formatTime)}
    ${renderBars("Peak memory", results, result => result.peakMemoryMb, maxMemory, formatMemory)}
  </div>
  <div class="table-wrap"><table>
    <thead><tr><th>Library</th><th>Language</th><th class="num">Time</th><th class="num">Throughput</th><th class="num">Memory</th><th class="num">HTML out</th></tr></thead>
    <tbody>${results.map(renderRow).join("")}</tbody>
  </table></div>
</section>`
}

function renderRow(result: Result) {
  const cells = [
    escapeHtml(result.title),
    LANGUAGES[result.language],
    escapeHtml(formatOutcome(result)),
    result.throughputMbs ? `${result.throughputMbs.toFixed(1)} MB/s` : "–",
    result.peakMemoryMb === undefined ? "–" : formatMemory(result.peakMemoryMb),
    result.outputBytes === undefined ? "–" : formatBytes(result.outputBytes),
  ]
  return `<tr>${cells.map((cell, index) => `<td${index > 1 ? ' class="num"' : ""}>${cell}</td>`).join("")}</tr>`
}

function renderBars(
  title: string,
  results: Result[],
  measure: (result: Result) => number | undefined,
  max: number,
  format: (value: number) => string,
) {
  const rows = results.map(result => {
    const value = measure(result)
    const kind = result.isTextlane ? "textlane" : "rival"
    const label = `${result.title} ${result.version}`
    const bar =
      value === undefined
        ? `<span class="missing">${escapeHtml(formatOutcome(result))}</span>`
        : `<span class="bar ${kind}" style="width:${Math.max((value / max) * 100, 0.5).toFixed(1)}%"></span><span class="value">${escapeHtml(format(value))}</span>`
    return `<div class="row" data-tip="${escapeHtml(label)}"><span class="name">${escapeHtml(result.title)}</span><span class="track">${bar}</span></div>`
  })
  return `<figure><figcaption>${title}</figcaption>${rows.join("")}</figure>`
}

function renderTools(results: Result[]) {
  const tools = new Map(results.map(result => [result.tool, result]))
  return `<section>
  <h2>What each library does</h2>
  <p class="caption">Every library converts CommonMark, or GFM in the <code>gfm</code> case, to HTML with
  raw HTML passed through.
  Differences from textlane, which builds an mdast tree and writes HTML from it, or converts it to hast and serializes that once an HTML plugin needs the tree:</p>
  <div class="table-wrap"><table>
    <thead><tr><th>Library</th><th>Version</th><th>Caveats</th></tr></thead>
    <tbody>${[...tools.values()]
      .map(
        result =>
          `<tr><td>${escapeHtml(result.title)}</td><td>${escapeHtml(result.version)}</td><td>${result.caveats.map(escapeHtml).join("<br>") || "–"}</td></tr>`,
      )
      .join("")}</tbody>
  </table></div>
</section>`
}

function renderMethod() {
  return `<section>
  <h2>Method</h2>
  <ul>
    <li>Documents are synthetic and seeded, so every library reads the same bytes. The
    <code>cmark</code> case reads like a README in CommonMark, <code>gfm</code> like one in
    GFM with GFM on in every library; <code>cmark-full</code> strings together the
    CommonMark edge cases of textlane's conformance suite.</li>
    <li>Each library runs as its own process and converts the document repeatedly, at least
    8 MB of input per run, so a small document is timed over many conversions. Time is clocked
    inside the process after imports and reading the file, and divided by the repeat count.</li>
    <li>Memory is the peak anonymous memory of the process, sampled every 50 ms, and so
    includes the runtime: a Node.js or Python process starts tens of megabytes above a Rust one.</li>
    <li><em>HTML out</em> is the size of the last conversion's output: one far from the others
    renders the document differently, and its time is not like for like.</li>
    <li>Reproduce: <code>pnpm -C benchmark setup:tools</code>, then
    <code>pnpm -C benchmark measure</code> and <code>pnpm -C benchmark show:report</code>.</li>
  </ul>
</section>`
}

function compareResults(left: Result, right: Result) {
  const leftTime = left.documentMs?.median ?? Number.POSITIVE_INFINITY
  const rightTime = right.documentMs?.median ?? Number.POSITIVE_INFINITY
  return leftTime - rightTime || left.title.localeCompare(right.title)
}

function formatOutcome(result: Result) {
  if (result.documentMs) return formatTime(result.documentMs.median)
  if (result.outcome === "oom") return "out of memory"
  if (result.outcome === "timeout") return "timed out"
  return "crashed"
}

function formatMemory(megabytes: number) {
  return megabytes >= 1024
    ? `${(megabytes / 1024).toFixed(1)} GB`
    : `${Math.round(megabytes)} MB`
}

function formatTime(milliseconds: number) {
  if (milliseconds < 1) return `${(milliseconds * 1000).toFixed(0)} µs`
  if (milliseconds < 1000) return `${milliseconds.toFixed(milliseconds < 10 ? 2 : 1)} ms`
  return `${(milliseconds / 1000).toFixed(2)} s`
}

function describeMachine(machine: Machine) {
  const cap = machine.memoryMax ? `, capped at ${machine.memoryMax} per library` : ""
  return `${machine.cpu} (${machine.cpus} threads), ${machine.memory} RAM${cap}, ${machine.os}`
}

export function escapeHtml(text: string) {
  return text
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
}

const STYLE = `
:root {
  color-scheme: light;
  --ground: #f6f5f2; --panel: #fcfcfb; --ink: #0b0b0b; --muted: #52514e; --rule: #e2e1dc;
  --textlane: #2a78d6; --rival: #b4b3ab; 
  --good: #0ca30c; --serious: #ec835a; --critical: #d03b3b;
}
@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) {
    color-scheme: dark;
    --ground: #121211; --panel: #1a1a19; --ink: #f0efec; --muted: #c3c2b7; --rule: #383835;
    --textlane: #3987e5; --rival: #6b6a64; 
  }
}
:root[data-theme="dark"] {
  color-scheme: dark;
  --ground: #121211; --panel: #1a1a19; --ink: #f0efec; --muted: #c3c2b7; --rule: #383835;
  --textlane: #3987e5; --rival: #6b6a64; 
}
* { box-sizing: border-box; }
body { margin: 0; background: var(--ground); color: var(--ink);
  font: 16px/1.55 system-ui, -apple-system, "Segoe UI", sans-serif; }
main { max-width: 980px; margin: 0 auto; padding: 40px 16px 64px; }
h1 { font-size: clamp(1.6rem, 4vw, 2.3rem); line-height: 1.15; margin: 4px 0 12px; }
h2 { font-size: 1.25rem; margin: 0 0 4px; }
.eyebrow { margin: 0; color: var(--muted); text-transform: uppercase; letter-spacing: .08em; font-size: .78rem; }
.lede { color: var(--muted); max-width: 70ch; margin: 0; }
section { background: var(--panel); border: 1px solid var(--rule); border-radius: 12px; padding: 20px; margin-top: 20px; }
.caption { color: var(--muted); font-size: .9rem; margin: 0 0 12px; }
.tiles { display: grid; grid-template-columns: repeat(auto-fit, minmax(min(190px, 100%), 1fr)); gap: 12px; }
.tile { border: 1px solid var(--rule); border-radius: 10px; padding: 14px; display: flex; flex-direction: column; gap: 2px; }
.tile .label { color: var(--muted); font-size: .85rem; }
.tile .value { font-size: 1.6rem; font-weight: 650; font-variant-numeric: tabular-nums; }
.tile .note { color: var(--muted); font-size: .85rem; }
.legend { display: flex; gap: 16px; font-size: .85rem; color: var(--muted); margin-bottom: 8px; }
.swatch { display: inline-block; width: 10px; height: 10px; border-radius: 2px; margin-right: 6px; vertical-align: -1px; }
.swatch.textlane, .bar.textlane { background: var(--textlane); }
.swatch.rival, .bar.rival { background: var(--rival); }
.charts { display: grid; grid-template-columns: repeat(auto-fit, minmax(min(300px, 100%), 1fr)); gap: 24px; margin-bottom: 16px; }
figure { margin: 0; min-width: 0; }
figcaption { font-weight: 600; font-size: .9rem; margin-bottom: 6px; }
.row { display: grid; grid-template-columns: minmax(0, min(11rem, 38%)) minmax(0, 1fr); gap: 8px; align-items: center; padding: 3px 0; position: relative; }
.row:hover { background: color-mix(in srgb, var(--rule) 45%, transparent); border-radius: 4px; }
.row:hover::after { content: attr(data-tip); position: absolute; left: 0; bottom: 100%; z-index: 1;
  background: var(--ink); color: var(--panel); font-size: .8rem; padding: 4px 8px; border-radius: 6px; white-space: nowrap; pointer-events: none; }
.name { font-size: .85rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.track { display: flex; align-items: center; gap: 6px; min-width: 0; }
.bar { height: 14px; border-radius: 0 4px 4px 0; flex: none; max-width: calc(100% - 4.5rem); }
.value, .missing { font-size: .8rem; color: var(--muted); font-variant-numeric: tabular-nums; white-space: nowrap; }
.missing { font-style: italic; }
.table-wrap { overflow-x: auto; }
table { width: 100%; border-collapse: collapse; font-size: .88rem; }
th, td { text-align: left; padding: 7px 8px; border-bottom: 1px solid var(--rule); vertical-align: top; }
th { color: var(--muted); font-weight: 600; }
.num { text-align: right; font-variant-numeric: tabular-nums; white-space: nowrap; }
.chip { display: inline-block; padding: 1px 8px; border-radius: 999px; font-size: .8rem; white-space: nowrap; border: 1px solid; }
.chip.good { color: var(--good); border-color: var(--good); }
.chip.critical { color: var(--critical); border-color: var(--critical); }
code { font-size: .88em; }
ul { padding-left: 1.2rem; margin: 0; }
li + li { margin-top: 6px; }
`
