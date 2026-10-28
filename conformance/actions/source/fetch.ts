import { execFileSync } from "node:child_process"
import { existsSync, mkdirSync } from "node:fs"
import { join, resolve } from "node:path"

const CACHE = resolve(import.meta.dirname, "..", "..", ".cache")

/** Check out one pinned commit of an upstream repository into the importers' cache. */
export function fetchSource(props: {
  repository: string
  commit: string
  paths?: string[]
}) {
  const name = props.repository.split("/").slice(-2).join("-")
  const path = join(CACHE, `${name}-${props.commit.slice(0, 7)}`)
  if (existsSync(join(path, ".git"))) return path
  mkdirSync(path, { recursive: true })
  const git = (...args: string[]) =>
    execFileSync("git", args, { cwd: path, stdio: "inherit" })
  git("init", "--quiet")
  // A repository too large to check out whole (WPT) is fetched without blobs, sparsely
  if (props.paths) {
    git(
      "fetch",
      "--quiet",
      "--depth",
      "1",
      "--filter=blob:none",
      props.repository,
      props.commit,
    )
    git("sparse-checkout", "set", "--no-cone", ...props.paths)
  } else {
    git("fetch", "--quiet", "--depth", "1", props.repository, props.commit)
  }
  git("checkout", "--quiet", "FETCH_HEAD")
  return path
}
