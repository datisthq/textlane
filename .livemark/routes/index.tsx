import { Link, createFileRoute } from "@tanstack/react-router"
import {
  ArrowRight,
  BookOpen,
  CircleCheck,
  Bot,
  Boxes,
  Braces,
  Gauge,
  Github,
  Rocket,
  ShieldCheck,
  Workflow,
  Zap,
} from "lucide-react"
import {
  type ComponentType,
  type ReactNode,
  type SVGProps,
  useRef,
  useState,
} from "react"
import { BackToTop } from "livemark/components/BackToTop"
import { Banner } from "livemark/components/Banner"
import { Header } from "livemark/components/Header"
import { buttonVariants } from "livemark/elements/button"
import { useInView } from "livemark/hooks/in-view"
import { cn } from "cn"

export const Route = createFileRoute("/")({
  component: Landing,
})

function Landing() {
  return (
    <>
      <Banner />
      <Header />
      <main className="flex flex-col">
        <Hero />
        <Pipeline />
        <Plugins />
        <Features />
        <ConformanceStripe />
        <Benchmark />
        <FinalCta />
        <Datist />
      </main>
      <BackToTop />
    </>
  )
}

const tk = {
  comment: "text-[#9ca0b0] dark:text-[#6c7086]",
  keyword: "text-[#8839ef] dark:text-[#cba6f7]",
  type: "text-[#df8e1d] dark:text-[#f9e2af]",
  string: "text-[#40a02b] dark:text-[#a6e3a1]",
  func: "text-[#1e66f5] dark:text-[#89b4fa]",
  punct: "text-[#7c7f93] dark:text-[#9399b2]",
  body: "text-[#4c4f69] dark:text-[#cdd6f4]",
  dim: "text-[#9ca0b0] dark:text-[#6c7086]",
  key: "text-[#1e66f5] dark:text-[#89b4fa]",
}

function Hero() {
  return (
    <section className="relative flex min-h-[calc(100svh-6rem)] items-center overflow-hidden border-b border-border">
      <BackgroundLanes />
      <div className="relative mx-auto w-full max-w-6xl px-6 py-20 md:py-28 lg:pt-8 lg:pb-14 lg:grid lg:grid-cols-[27rem_minmax(0,1fr)] lg:items-center lg:gap-14">
        <div className="animate-in fade-in-0 slide-in-from-bottom-4 duration-700 ease-out">
          <p className="text-primary text-xs font-semibold tracking-[0.18em] uppercase">
            Open source · MIT License
          </p>
          <h1 className="mt-5 text-4xl font-semibold tracking-tight text-foreground sm:text-5xl md:text-6xl lg:text-5xl">
            Fast text pipeline for{" "}
            <span className="box-decoration-clone bg-linear-to-r from-primary/20 to-primary/20 bg-size-[100%_0.3em] bg-position-[0_88%] bg-no-repeat">
              Python, TypeScript, and Rust
            </span>
          </h1>
          <p className="mt-6 max-w-[52ch] text-lg leading-relaxed text-muted-foreground md:text-xl">
            Textlane parses Markdown, HTML and natural language into syntax trees,
            transforms them with plugins and writes them back out. One native Rust core
            does the work, with the same API in every language.
          </p>
          <HeroActions />
        </div>
        <div className="mt-12 lg:mt-0 xl:w-[60rem] animate-in fade-in-0 slide-in-from-bottom-4 duration-700 ease-out">
          <HeroCarousel slides={slides} />
        </div>
      </div>
    </section>
  )
}

function HeroActions() {
  return (
    <>
      <div className="mt-10 flex flex-wrap gap-3">
        <Link
          to="/$"
          params={{ _splat: "python/introduction" }}
          className={cn(
            buttonVariants({ variant: "default", size: "lg" }),
            "flex-1 px-3 no-underline sm:flex-none sm:px-5",
          )}
        >
          <Rocket className="size-4" />
          Get started
        </Link>
        <a
          href="https://github.com/datisthq/textlane"
          target="_blank"
          rel="noopener noreferrer"
          className={cn(
            buttonVariants({ variant: "outline", size: "lg" }),
            "flex-1 px-3 no-underline sm:flex-none sm:px-5",
          )}
        >
          <Github className="size-4" />
          View source
        </a>
      </div>
      <InstallCard className="mt-6 hidden sm:inline-grid" />
    </>
  )
}

function InstallCard(props: { className?: string }) {
  return (
    <div
      className={cn(
        "grid-cols-[auto_auto] gap-x-3 gap-y-1 rounded-[1.25rem] border border-border bg-card/50 backdrop-blur px-4 py-2 text-left font-mono text-sm text-muted-foreground",
        props.className,
      )}
    >
      <span className="text-primary select-none">$</span>
      <span>
        <span className="text-foreground">pip install</span> textlane
      </span>
      <span className="text-primary select-none">$</span>
      <span>
        <span className="text-foreground">npm install</span> textlane
      </span>
      <span className="text-primary select-none">$</span>
      <span>
        <span className="text-foreground">cargo add</span> textlane
      </span>
    </div>
  )
}

interface Slide {
  id: string
  title: string
  stages: string[]
  ports: string[]
  input: { label: string; text: string }
  output: { label: string; text: string }
}

// Outputs are what the pipeline returns for these inputs, as its conformance cases expect
const slides: Slide[] = [
  {
    id: "md-html",
    title: "MD → HTML",
    stages: ["md.parse", "md.gfm", "md.toHtml", "html.slug", "html.stringify"],
    ports: [
      "remark-parse",
      "remark-gfm",
      "remark-rehype",
      "rehype-slug",
      "rehype-stringify",
    ],
    input: {
      label: "input.md",
      text: "# Hello, *lane*\n\n- [x] parse\n- [ ] ~~rewrite~~",
    },
    output: {
      label: "output.html",
      text: '<h1 id="hello-lane">Hello, <em>lane</em></h1>\n<ul class="contains-task-list">\n<li class="task-list-item"><input type="checkbox" checked disabled> parse</li>\n<li class="task-list-item"><input type="checkbox" disabled> <del>rewrite</del></li>\n</ul>',
    },
  },
  {
    id: "html-md",
    title: "HTML → MD",
    stages: ["html.parse", "html.toMarkdown", "md.stringify"],
    ports: ["rehype-parse", "rehype-remark", "remark-stringify"],
    input: {
      label: "input.html",
      text: '<h2>Notes</h2>\n<p>Read <a href="https://unifiedjs.com">unified</a> <em>first</em>.</p>',
    },
    output: {
      label: "output.md",
      text: "## Notes\n\nRead [unified](https://unifiedjs.com) *first*.",
    },
  },
  {
    id: "smartypants",
    title: "Typography",
    stages: ["nl.parse", "nl.smartypants", "nl.stringify"],
    ports: ["retext-english", "retext-smartypants", "retext-stringify"],
    input: { label: "input.txt", text: "Alfred \"bertrand\" 'cees'." },
    output: { label: "output.txt", text: "Alfred “bertrand” ‘cees’." },
  },
  {
    id: "lint",
    title: "Lint",
    stages: ["nl.parse", "nl.repeatedWords", "nl.stringify"],
    ports: ["retext-english", "retext-repeated-words", "retext-stringify"],
    input: { label: "input.txt", text: "Well, it it doesn’t have to be." },
    output: {
      label: "messages.json",
      text: '[\n  {\n    "ruleId": "it",\n    "source": "retext-repeated-words",\n    "place": {\n      "start": { "line": 1, "column": 7 },\n      "end": { "line": 1, "column": 12 }\n    }\n  }\n]',
    },
  },
]

function HeroCarousel(props: { slides: Slide[] }) {
  const [active, setActive] = useState(0)
  const [paused, setPaused] = useState(false)
  const [stopped, setStopped] = useState(false)
  const stoppedRef = useRef(false)

  return (
    <div
      className="relative"
      onMouseEnter={() => setPaused(true)}
      onMouseLeave={() => setPaused(false)}
      onFocus={() => setPaused(true)}
      onBlur={() => setPaused(false)}
    >
      <figure className="overflow-hidden rounded-card border border-border bg-card shadow-xl">
        <div className="flex items-center gap-5 border-b border-border bg-muted/50 px-4 py-2">
          <span aria-hidden className="hidden gap-1.5 sm:flex">
            <span className="size-2.5 rounded-full bg-border" />
            <span className="size-2.5 rounded-full bg-border" />
            <span className="size-2.5 rounded-full bg-border" />
          </span>
          <div role="tablist" className="flex min-w-0 gap-1 overflow-x-auto">
            {props.slides.map((slide, index) => (
              <button
                key={slide.id}
                type="button"
                role="tab"
                aria-selected={index === active}
                aria-controls={`hero-slide-${slide.id}`}
                onClick={() => {
                  stoppedRef.current = true
                  setActive(index)
                  setStopped(true)
                }}
                className={cn(
                  "relative flex cursor-pointer items-center gap-2 overflow-hidden rounded-control px-3 py-1.5 text-sm whitespace-nowrap transition-colors",
                  index === active
                    ? "bg-card font-medium text-foreground shadow-sm"
                    : "text-muted-foreground hover:text-foreground",
                )}
              >
                <span
                  className={cn("font-mono text-xs", index === active && "text-primary")}
                >
                  {String(index + 1).padStart(2, "0")}
                </span>
                <span className={index === active ? undefined : "hidden md:inline"}>
                  {slide.title}
                </span>
                {index === active ? (
                  <span
                    key={stopped ? "stopped" : active}
                    aria-hidden
                    className={cn(
                      "absolute inset-x-3 bottom-0.5 h-0.5 rounded-full bg-primary/70",
                      !stopped && "hero-carousel-progress",
                    )}
                    style={{ animationPlayState: paused ? "paused" : "running" }}
                    onAnimationEnd={() => {
                      if (stoppedRef.current) return
                      setActive(current => (current + 1) % props.slides.length)
                    }}
                  />
                ) : null}
              </button>
            ))}
          </div>
        </div>
        <div className="grid">
          {props.slides.map((slide, index) => (
            <div
              key={slide.id}
              id={`hero-slide-${slide.id}`}
              role="tabpanel"
              aria-hidden={index !== active}
              inert={index !== active}
              className={cn(
                "[grid-area:1/1] transition-opacity duration-700 motion-reduce:transition-none",
                index === active ? "opacity-100" : "pointer-events-none opacity-0",
              )}
            >
              <SlideBody slide={slide} />
            </div>
          ))}
        </div>
      </figure>
      <div className="mt-4 lg:absolute lg:-bottom-16 lg:-left-12 lg:mt-0 lg:w-[30rem]">
        <PortsCard slides={props.slides} active={active} />
      </div>
    </div>
  )
}

function SlideBody(props: { slide: Slide }) {
  return (
    <div className="flex h-full flex-col gap-4 p-4 sm:p-5 lg:pb-36">
      <ol className="flex flex-wrap items-center gap-1.5">
        {props.slide.stages.map((stage, index) => (
          <li key={stage} className="flex items-center gap-1.5">
            {index > 0 && (
              <ArrowRight aria-hidden className="size-3.5 text-muted-foreground/60" />
            )}
            <code className="rounded-md border border-primary/25 bg-primary/10 px-2 py-0.5 font-mono text-xs text-primary">
              {stage}
            </code>
          </li>
        ))}
      </ol>
      <div className="grid flex-1 grid-cols-1 gap-3">
        <Pane label={props.slide.input.label} text={props.slide.input.text} />
        <Pane label={props.slide.output.label} text={props.slide.output.text} />
      </div>
    </div>
  )
}

function Pane(props: { label: string; text: string }) {
  return (
    <div className="flex min-w-0 flex-col overflow-hidden rounded-lg border border-border bg-muted/40">
      <p className="border-b border-border px-3 py-1.5 font-mono text-xs text-muted-foreground">
        {props.label}
      </p>
      <pre className="flex-1 overflow-x-auto p-3 font-mono text-xs leading-relaxed">
        <code className={tk.body}>{props.text}</code>
      </pre>
    </div>
  )
}

function PortsCard(props: { slides: Slide[]; active: number }) {
  return (
    <figure className="rounded-card border border-primary/25 bg-card p-4 shadow-lg">
      <div className="flex items-start gap-3">
        <span className="flex size-10 shrink-0 items-center justify-center rounded-lg bg-primary text-primary-foreground">
          <Workflow className="size-5" />
        </span>
        <div className="min-w-0 flex-1">
          <p className="text-xs font-semibold tracking-[0.12em] text-primary uppercase">
            unified-compatible
          </p>
          <p className="mt-0.5 text-base font-semibold text-foreground">
            Built by AI, in Rust
          </p>
        </div>
        <p className="shrink-0 text-right leading-none">
          <span className="bg-gradient-to-br from-primary to-primary/60 bg-clip-text font-mono text-3xl font-bold tracking-tight text-transparent tabular-nums">
            100×
          </span>
          <span className="mt-1 block text-[0.625rem] tracking-wide text-muted-foreground uppercase">
            faster with Rust core
          </span>
        </p>
      </div>
      <p className="mt-3 text-sm leading-relaxed text-muted-foreground">
        All credit goes to the{" "}
        <a
          href="https://unifiedjs.com/"
          target="_blank"
          rel="noopener noreferrer"
          className="font-medium text-primary underline underline-offset-4 hover:no-underline"
        >
          unified community
        </a>{" "}
        for creating the project: its plugins, syntax trees and pipeline design are
        theirs.
      </p>
      <div className="mt-4 grid">
        {props.slides.map((slide, index) => (
          <figcaption
            key={slide.id}
            aria-hidden={index !== props.active}
            className={cn(
              "[grid-area:1/1] flex flex-wrap content-start gap-1.5 transition-opacity duration-700 motion-reduce:transition-none",
              index === props.active ? "opacity-100" : "opacity-0",
            )}
          >
            {slide.ports.map(port => (
              <code
                key={port}
                className="rounded-md border border-border bg-muted/60 px-2 py-0.5 font-mono text-xs text-foreground"
              >
                {port}
              </code>
            ))}
          </figcaption>
        ))}
      </div>
    </figure>
  )
}

function BackgroundLanes() {
  return (
    <>
      <style href="textlane-hero-background" precedence="default">
        {BACKGROUND_CSS}
      </style>
      <div aria-hidden className="pointer-events-none absolute inset-0 overflow-hidden">
        <div className="absolute inset-0 [mask-image:linear-gradient(to_bottom,black_55%,transparent_98%)]">
          <div className="lanes lanes-drift absolute -inset-y-11 inset-x-0 opacity-50" />
          <div className="hero-scan-x from-primary/0 via-primary/16 to-primary/0 absolute inset-y-0 w-[40%] bg-gradient-to-r" />
          <div className="hero-scan-y from-secondary/0 via-secondary/16 to-secondary/0 absolute inset-x-0 h-[40%] bg-gradient-to-b" />
        </div>
      </div>
      <div
        aria-hidden
        className="hero-drift bg-primary/15 pointer-events-none absolute -top-40 -right-40 size-[32rem] rounded-full blur-3xl"
      />
    </>
  )
}

const BACKGROUND_CSS = `
@keyframes hero-scan-x {
  0% {
    transform: translateX(-55%);
    opacity: 0;
  }
  5%,
  45% {
    opacity: 1;
  }
  50%,
  100% {
    transform: translateX(155%);
    opacity: 0;
  }
}

@keyframes hero-scan-y {
  0%,
  50% {
    transform: translateY(-55%);
    opacity: 0;
  }
  55%,
  95% {
    opacity: 1;
  }
  100% {
    transform: translateY(155%);
    opacity: 0;
  }
}

@keyframes lanes-drift {
  to {
    transform: translateY(44px);
  }
}

@keyframes hero-drift {
  0%,
  100% {
    transform: translate3d(0, 0, 0) scale(1);
  }
  50% {
    transform: translate3d(6%, -8%, 0) scale(1.08);
  }
}

.lanes {
  background-image: repeating-linear-gradient(
    0deg,
    var(--color-border) 0,
    var(--color-border) 1px,
    transparent 1px,
    transparent 44px
  );
}

.lanes-drift {
  animation: lanes-drift 14s linear infinite;
}

.hero-scan-x {
  animation: hero-scan-x 18s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}

.hero-scan-y {
  animation: hero-scan-y 18s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}

.hero-drift {
  animation: hero-drift 18s ease-in-out infinite;
}

@keyframes hero-carousel-progress {
  from {
    transform: scaleX(0);
  }
  to {
    transform: scaleX(1);
  }
}

.hero-carousel-progress {
  transform-origin: left;
  animation: hero-carousel-progress 6s linear;
}

@media (prefers-reduced-motion: reduce) {
  .hero-scan-x,
  .hero-scan-y {
    animation: none;
    opacity: 0;
  }
  .lanes-drift,
  .hero-drift,
  .hero-carousel-progress {
    animation: none;
  }
}
`

interface Example {
  id: string
  label: string
  file: string
  keywords: Set<string>
  source: string
}

const examples: Example[] = [
  {
    id: "python",
    label: "Python",
    file: "main.py",
    keywords: new Set(["from", "import"]),
    source: `from textlane import textlane, md, html

processor = (
    textlane()
    .use(md.parse())
    .use(md.gfm())
    .use(md.to_html())
    .use(html.slug())
    .use(html.stringify())
)

file = processor.process(markdown)`,
  },
  {
    id: "typescript",
    label: "TypeScript",
    file: "main.ts",
    keywords: new Set(["import", "from", "const", "await", "as"]),
    source: `import { textlane } from "textlane"
import * as md from "textlane/md"
import * as html from "textlane/html"

const file = await textlane()
  .use(md.parse())
  .use(md.gfm())
  .use(md.toHtml())
  .use(html.slug())
  .use(html.stringify())
  .process(markdown)`,
  },
  {
    id: "rust",
    label: "Rust",
    file: "main.rs",
    keywords: new Set(["use", "let", "mut", "fn"]),
    source: `use textlane::{Processor, html, md};

let processor = Processor::new()
    .with(md::Parse::default())
    .with(md::Gfm::default())
    .with(md::ToHtml::default())
    .with(html::Slug::default())
    .with(html::Stringify::default())
    .freeze()?;

let output = processor.process(markdown)?;`,
  },
]

const CODE_TOKEN =
  /(\/\/.*)|("(?:[^"\\]|\\.)*")|(\b\d+\b)|([A-Za-z_]\w*)|([^\sA-Za-z_\d"]+)|(\s+)|([\s\S])/g

function highlightCode(code: string, keywords: Set<string>) {
  return [...code.matchAll(CODE_TOKEN)].map(match => {
    const [text, comment, string, number, word, punct] = match
    const next = code[match.index + text.length]
    const className = comment
      ? tk.comment
      : string
        ? tk.string
        : number
          ? tk.type
          : word
            ? keywords.has(word)
              ? tk.keyword
              : next === "("
                ? tk.func
                : /^[A-Z]/.test(word)
                  ? tk.type
                  : undefined
            : punct
              ? tk.punct
              : undefined
    return className ? (
      <span key={match.index} className={className}>
        {text}
      </span>
    ) : (
      text
    )
  })
}

function Pipeline() {
  return (
    <section className="border-b border-border bg-primary/5">
      <div className="mx-auto max-w-6xl px-6 pt-16 md:pt-24">
        <Reveal>
          <div className="text-center max-w-2xl mx-auto mb-10 md:mb-16">
            <h2 className="text-3xl md:text-4xl font-bold tracking-tight">Languages</h2>
            <p className="mt-4 text-muted-foreground text-lg">
              One pipeline in Python, TypeScript and Rust; each stage ports the unified
              plugin of the same name.
            </p>
          </div>
        </Reveal>
      </div>
      <div className="mx-auto max-w-7xl px-6 pb-16 md:pb-24">
        <div className="grid grid-cols-1 gap-4 lg:grid-cols-3">
          {examples.map((example, i) => (
            <Reveal key={example.id} delayMs={i * 60}>
              <WindowCard label={`${example.label} · ${example.file}`}>
                <pre className="overflow-x-auto p-4 font-mono text-xs leading-relaxed sm:text-sm lg:text-xs">
                  <code className={tk.body}>
                    {highlightCode(example.source, example.keywords)}
                  </code>
                </pre>
              </WindowCard>
            </Reveal>
          ))}
        </div>
      </div>
    </section>
  )
}

interface Domain {
  name: string
  title: string
  description: string
  plugins: [string, string][]
}

const domains: Domain[] = [
  {
    name: "md/mdx",
    title: "Markdown",
    description: "A port of micromark and the mdast utilities.",
    plugins: [
      ["md.parse", "remark-parse"],
      ["md.stringify", "remark-stringify"],
      ["md.toHtml", "remark-rehype"],
      ["md.toNaturalLanguage", "remark-retext"],
      ["md.gfm", "remark-gfm"],
      ["md.frontmatter", "remark-frontmatter"],
      ["md.math", "remark-math"],
      ["md.directive", "remark-directive"],
      ["md.gemoji", "remark-gemoji"],
      ["md.definitionList", "remark-definition-list"],
      ["md.mdx", "remark-mdx"],
      ["mdx.compile", "@mdx-js/mdx"],
    ],
  },
  {
    name: "html",
    title: "HTML",
    description: "A port of parse5 and the hast utilities.",
    plugins: [
      ["html.parse", "rehype-parse"],
      ["html.stringify", "rehype-stringify"],
      ["html.toMarkdown", "rehype-remark"],
      ["html.toNaturalLanguage", "rehype-retext"],
      ["html.raw", "rehype-raw"],
      ["html.sanitize", "rehype-sanitize"],
      ["html.slug", "rehype-slug"],
      ["html.format", "rehype-format"],
      ["html.highlight", "@shikijs/rehype"],
    ],
  },
  {
    name: "nl",
    title: "Natural Language",
    description: "A port of parse-english and the retext plugins.",
    plugins: [
      ["nl.parse", "retext-english"],
      ["nl.stringify", "retext-stringify"],
      ["nl.smartypants", "retext-smartypants"],
      ["nl.quotes", "retext-quotes"],
      ["nl.contractions", "retext-contractions"],
      ["nl.repeatedWords", "retext-repeated-words"],
      ["nl.indefiniteArticle", "retext-indefinite-article"],
      ["nl.passive", "retext-passive"],
      ["nl.readability", "retext-readability"],
      ["nl.equality", "retext-equality"],
      ["nl.sentenceSpacing", "retext-sentence-spacing"],
    ],
  },
]

function Plugins() {
  return (
    <section className="border-b border-border">
      <div className="mx-auto max-w-6xl px-6 py-16 md:py-24">
        <Reveal>
          <div className="text-center max-w-2xl mx-auto mb-10 md:mb-16">
            <h2 className="text-3xl md:text-4xl font-bold tracking-tight">Plugins</h2>
            <p className="mt-4 text-muted-foreground text-lg">
              Markdown, HTML and natural language, each a crate of its own, bridged by the
              same converters unified uses.
            </p>
          </div>
        </Reveal>
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
          {domains.map((domain, i) => (
            <Reveal key={domain.name} delayMs={i * 60}>
              <WindowCard label={`textlane-${domain.name}`}>
                <div className="p-5">
                  <h3 className="text-lg font-semibold text-foreground">
                    {domain.title}
                  </h3>
                  <p className="mt-1 text-sm text-muted-foreground">
                    {domain.description}
                  </p>
                  <ul className="mt-4 space-y-1.5 font-mono text-xs sm:text-sm">
                    {domain.plugins.map(([plugin, port]) => (
                      <li
                        key={plugin}
                        className="flex items-baseline justify-between gap-3"
                      >
                        <span className="text-primary">{plugin}</span>
                        <span className="truncate text-xs text-muted-foreground">
                          {port}
                        </span>
                      </li>
                    ))}
                  </ul>
                </div>
              </WindowCard>
            </Reveal>
          ))}
        </div>
      </div>
    </section>
  )
}

interface Feature {
  icon: ComponentType<SVGProps<SVGSVGElement>>
  title: string
  description: string
}

const features: Feature[] = [
  {
    icon: Zap,
    title: "Native execution",
    description:
      "Parsing, transforms and serialization run in Rust over trees held in a per-document arena, not a heap of objects.",
  },
  {
    icon: Boxes,
    title: "One crate, three languages",
    description:
      "crates.io gets the crate, npm a prebuilt addon per platform, PyPI a PyO3 wheel: the same code at the same version.",
  },
  {
    icon: Workflow,
    title: "unified's semantics",
    description:
      "use, data and freeze; parse, run and stringify; presets, reconfiguring a plugin, sync and async.",
  },
  {
    icon: Braces,
    title: "Compatible trees",
    description:
      "mdast, hast and nlcst nodes with unified's types, fields and positions, read and written as unified's JSON.",
  },
  {
    icon: ShieldCheck,
    title: "10k+ conformance tests",
    description:
      "Ported from CommonMark, html5lib and WPT, remark, rehype and retext. Every implementation runs the same JSON cases.",
  },
  {
    icon: Bot,
    title: "Agent skill",
    description:
      "The npm and PyPI packages ship a skill that teaches coding agents to install and call textlane; this site publishes it too.",
  },
]

function Features() {
  return (
    <section className="border-b border-border">
      <div className="mx-auto max-w-6xl px-6 py-16 md:py-24">
        <Reveal>
          <div className="text-center max-w-2xl mx-auto mb-10 md:mb-16">
            <h2 className="text-3xl md:text-4xl font-bold tracking-tight">
              Compatibility
            </h2>
            <p className="mt-4 text-muted-foreground text-lg">
              Each port is checked case by case against its upstream test suite.
            </p>
          </div>
        </Reveal>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {features.map((feature, i) => (
            <Reveal key={feature.title} delayMs={i * 60}>
              <FeatureCard feature={feature} />
            </Reveal>
          ))}
        </div>
      </div>
    </section>
  )
}

function FeatureCard(props: { feature: Feature }) {
  return (
    <div className="h-full group relative rounded-card border border-border bg-card p-6 transition-all duration-200 hover:border-primary/40 hover:-translate-y-0.5 hover:shadow-lg">
      <div className="inline-flex items-center justify-center size-10 rounded-lg bg-primary/10 text-primary mb-4 group-hover:bg-primary/15 transition-colors">
        <props.feature.icon className="size-5" />
      </div>
      <h3 className="text-lg font-semibold text-foreground">{props.feature.title}</h3>
      <p className="mt-2 text-sm text-muted-foreground leading-relaxed">
        {props.feature.description}
      </p>
    </div>
  )
}

const SUITE = "https://github.com/datisthq/textlane/tree/main/conformance"

const conformanceStats = [
  { value: "10k+", label: "tests" },
  { value: "30", label: "plugins" },
  { value: "3", label: "languages" },
]

const conformanceCase = `{
  "description": "micromark-extension-gfm-strikethrough",
  "pipeline": [
    ["md.parse"],
    ["md.gfm"],
    ["md.toHtml"],
    ["html.stringify"]
  ],
  "tests": [{
    "description": "should support nested strikethrough",
    "input": "a ~~b ~~c~~ d~~ e",
    "expected": {
      "html": "<p>a <del>b <del>c</del> d</del> e</p>"
    }
  }]
}`

const JSON_TOKEN =
  /("(?:[^"\\]|\\.)*"(?=:))|("(?:[^"\\]|\\.)*")|(\b\d+\b)|([{}[\],:])|(\s+|[^\s"{}[\],:]+)/g

function highlightJson(code: string) {
  return [...code.matchAll(JSON_TOKEN)].map(match => {
    const [text, key, string, number, punct] = match
    const className = key
      ? tk.key
      : string
        ? tk.string
        : number
          ? tk.type
          : punct
            ? tk.punct
            : undefined
    return className ? (
      <span key={match.index} className={className}>
        {text}
      </span>
    ) : (
      text
    )
  })
}

function ConformanceStripe() {
  return (
    <section className="border-b border-border bg-primary/5">
      <div className="mx-auto grid max-w-6xl items-center gap-10 px-6 py-16 *:min-w-0 md:grid-cols-2 md:gap-12 md:py-20">
        <Reveal>
          <p className="text-xs font-semibold tracking-[0.12em] text-primary uppercase">
            Conformance suite
          </p>
          <h2 className="mt-3 text-3xl font-bold tracking-tight md:text-4xl">
            Shared across languages
          </h2>
          <p className="mt-4 text-lg text-muted-foreground">
            Python, TypeScript and Rust run the same language-neutral cases, ported from
            CommonMark, html5lib, remark, rehype and retext: the same pipeline, the same
            input, the same expected output.
          </p>
          <dl className="mt-8 grid grid-cols-3 gap-4">
            {conformanceStats.map(stat => (
              <div
                key={stat.label}
                className="flex flex-col border-l-2 border-primary/40 pl-3"
              >
                <dt className="text-xs text-muted-foreground">{stat.label}</dt>
                <dd className="order-first font-mono text-2xl font-bold tracking-tight text-foreground tabular-nums">
                  {stat.value}
                </dd>
              </div>
            ))}
          </dl>
          <a
            href={SUITE}
            target="_blank"
            rel="noopener noreferrer"
            className="mt-8 inline-block font-medium text-primary underline-offset-4 hover:underline"
          >
            Browse the suite →
          </a>
        </Reveal>
        <Reveal delayMs={100}>
          <WindowCard label="conformance/tests/md/gfm-strikethrough.json">
            <pre className="overflow-x-auto p-5 font-mono text-xs leading-relaxed sm:text-sm">
              <code className={tk.body}>{highlightJson(conformanceCase)}</code>
            </pre>
            <div className="flex flex-wrap gap-2 border-t border-primary/20 bg-muted/50 px-4 py-3">
              {["Python", "TypeScript", "Rust"].map(language => (
                <span
                  key={language}
                  className="inline-flex items-center gap-1.5 rounded-full border border-border bg-card px-2.5 py-1 font-mono text-xs"
                >
                  <CircleCheck className="size-3.5 text-[#40a02b] dark:text-[#a6e3a1]" />
                  {language}
                </span>
              ))}
            </div>
          </WindowCard>
        </Reveal>
      </div>
    </section>
  )
}

interface Measure {
  tool: string
  mbps: number
  isTextlane?: boolean
}

// The 1 MB `realistic` case of benchmark/, GFM to HTML, input MB/s at the median
const measures: Measure[] = [
  { tool: "pulldown-cmark", mbps: 218.2 },
  { tool: "comrak", mbps: 53.8 },
  { tool: "textlane (Rust)", mbps: 33.0, isTextlane: true },
  { tool: "cmark (C, via cmarkgfm)", mbps: 32.1 },
  { tool: "textlane (Node.js)", mbps: 30.0, isTextlane: true },
  { tool: "textlane (Python)", mbps: 26.8, isTextlane: true },
  { tool: "marked", mbps: 10.1 },
  { tool: "mistune", mbps: 3.4 },
  { tool: "markdown-it", mbps: 2.8 },
  { tool: "Python-Markdown", mbps: 1.5 },
  { tool: "markdown-rs", mbps: 1.5 },
  { tool: "markdown-it-py", mbps: 1.4 },
  { tool: "micromark", mbps: 0.6 },
  { tool: "unified (remark + rehype)", mbps: 0.5 },
]

function Benchmark() {
  const fastest = Math.max(...measures.map(measure => measure.mbps))
  return (
    <section className="border-b border-border">
      <div className="mx-auto max-w-6xl px-6 py-16 md:py-24">
        <Reveal>
          <div className="text-center max-w-2xl mx-auto mb-10 md:mb-16">
            <h2 className="text-3xl md:text-4xl font-bold tracking-tight">Performance</h2>
            <p className="mt-4 text-muted-foreground text-lg">
              Markdown-to-HTML throughput from Rust, Node.js and Python, against other
              libraries. textlane builds two trees, mdast and hast, for plugins to
              transform; comrak and cmark build one, pulldown-cmark none.
            </p>
          </div>
        </Reveal>
        <Reveal>
          <WindowCard label="benchmark · throughput on a 1 MB GFM README">
            <div className="p-5 sm:p-6">
              <ul className="space-y-3">
                {measures.map(measure => (
                  <li
                    key={measure.tool}
                    className="grid grid-cols-1 gap-1 sm:grid-cols-[13rem_minmax(0,1fr)_6rem] sm:items-center sm:gap-4"
                  >
                    <span
                      className={cn(
                        "text-sm",
                        measure.isTextlane
                          ? "font-semibold text-foreground"
                          : "text-muted-foreground",
                      )}
                    >
                      {measure.tool}
                    </span>
                    <span className="h-3 rounded-full bg-muted">
                      <span
                        className={cn(
                          "block h-full min-w-1 rounded-full",
                          measure.isTextlane ? "bg-primary" : "bg-muted-foreground/40",
                        )}
                        style={{ width: `${(measure.mbps / fastest) * 100}%` }}
                      />
                    </span>
                    <span className="font-mono text-xs whitespace-nowrap text-muted-foreground sm:text-right">
                      {measure.mbps} MB/s
                    </span>
                  </li>
                ))}
              </ul>
              <p className="mt-6 flex items-start gap-2 text-xs leading-relaxed text-muted-foreground">
                <Gauge className="mt-0.5 size-3.5 shrink-0" />
                <span>
                  GFM in every library, input megabytes per second at the median, on an
                  AMD Ryzen 5 PRO 7545U. Reproduce with{" "}
                  <code className="font-mono text-foreground">
                    pnpm -C benchmark measure
                  </code>
                  .
                </span>
              </p>
            </div>
          </WindowCard>
        </Reveal>
      </div>
    </section>
  )
}

function WindowCard(props: { label: string; children: ReactNode }) {
  return (
    <div className="h-full rounded-card border border-primary/20 bg-card overflow-hidden">
      <div className="flex items-center gap-2 border-b border-primary/20 px-4 py-2 bg-muted/50">
        <div className="size-2.5 shrink-0 rounded-full bg-red-400/60" />
        <div className="size-2.5 shrink-0 rounded-full bg-yellow-400/60" />
        <div className="size-2.5 shrink-0 rounded-full bg-green-400/60" />
        <span className="ml-2 truncate text-xs font-mono text-muted-foreground">
          {props.label}
        </span>
      </div>
      {props.children}
    </div>
  )
}

function FinalCta() {
  return (
    <section className="relative overflow-hidden">
      <div className="relative mx-auto max-w-3xl px-6 py-16 md:py-24 text-center">
        <Reveal>
          <h2 className="text-3xl md:text-4xl font-bold tracking-tight">Get started</h2>
          <p className="mt-6 text-lg text-muted-foreground">
            Textlane is in technical preview: the crate and its plugins are here, the
            Node.js and Python pipeline API is on its way.
          </p>
          <InstallCard className="mt-8 inline-grid" />
          <div className="mt-8 flex flex-col sm:flex-row items-center justify-center gap-3">
            <Link
              to="/$"
              params={{ _splat: "python/introduction" }}
              className={cn(
                buttonVariants({ variant: "default", size: "lg" }),
                "px-5 no-underline",
              )}
            >
              <BookOpen className="size-4" />
              Read the docs
            </Link>
            <a
              href="https://github.com/datisthq/textlane/tree/main/conformance"
              target="_blank"
              rel="noopener noreferrer"
              className={cn(
                buttonVariants({ variant: "ghost", size: "lg" }),
                "px-5 no-underline",
              )}
            >
              Browse the conformance suite
            </a>
          </div>
        </Reveal>
      </div>
    </section>
  )
}

function Datist() {
  return (
    <section className="border-t border-border bg-muted">
      <div className="mx-auto max-w-5xl px-6 py-10">
        <Reveal>
          <div className="flex flex-col items-center gap-3 text-center">
            <p className="text-xs font-semibold uppercase tracking-widest text-muted-foreground">
              Built by
            </p>
            <a
              href="https://datist.io"
              target="_blank"
              rel="noopener"
              className="text-sm font-medium text-foreground/70 hover:text-foreground transition-colors no-underline"
            >
              Datist — Software development, end to end.
            </a>
          </div>
        </Reveal>
      </div>
    </section>
  )
}

function Reveal(props: { children: ReactNode; delayMs?: number }) {
  const { ref, isVisible } = useInView()
  return (
    <div
      ref={ref as (node: HTMLDivElement | null) => void}
      style={{ transitionDelay: `${props.delayMs ?? 0}ms` }}
      className={cn(
        "transition-all duration-700 ease-out",
        isVisible ? "opacity-100 translate-y-0" : "opacity-0 translate-y-4",
      )}
    >
      {props.children}
    </div>
  )
}
