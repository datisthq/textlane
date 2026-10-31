/** The C library Node runs on under Linux: glibc when its report names a glibc, else musl. */
export function detectLibc(props: { platform: string; report: () => object }) {
  if (props.platform !== "linux") return undefined
  const report = props.report()
  const header = "header" in report ? report.header : undefined
  if (typeof header !== "object" || header === null) return "glibc"
  return "glibcVersionRuntime" in header && header.glibcVersionRuntime ? "glibc" : "musl"
}
