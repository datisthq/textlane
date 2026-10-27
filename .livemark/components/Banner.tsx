import { ArrowRight } from "lucide-react"

export function Banner() {
  return (
    <div className="bg-banner text-banner-foreground">
      <div className="mx-auto flex max-w-6xl items-center justify-center gap-2 px-4 py-2 text-center font-mono text-xs tracking-wide sm:px-6">
        <span className="min-w-0 truncate">
          <span className="hidden sm:inline">Textlane is in technical preview</span>
          <span className="sm:hidden">Technical preview</span>
        </span>
        <span aria-hidden className="opacity-60">
          ·
        </span>
        <a
          href="https://github.com/datisthq/textlane/discussions"
          target="_blank"
          rel="noopener noreferrer"
          className="group inline-flex shrink-0 items-center gap-1 no-underline"
        >
          <span className="underline underline-offset-4 group-hover:no-underline">
            Share feedback
          </span>
          <ArrowRight className="size-3 transition-transform group-hover:translate-x-0.5 group-active:translate-x-0.5 motion-reduce:transition-none" />
        </a>
      </div>
    </div>
  )
}
