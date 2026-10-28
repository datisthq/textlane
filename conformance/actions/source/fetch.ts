import { execFileSync } from "node:child_process"
import { existsSync, mkdirSync } from "node:fs"
import { join, resolve } from "node:path"

const CACHE = resolve(import.meta.dirname, "..", "..", ".cache")

/** Check out one pinned commit of an upstream repository into the importers' cache. */
export function fetchSource(props: { repository: string; commit: string }) {
  const name = props.repository.split("/").slice(-2).join("-")
  const path = join(CACHE, `${name}-${props.commit.slice(0, 7)}`)
  if (existsSync(join(path, ".git"))) return path
  mkdirSync(path, { recursive: true })
  const git = (...args: string[]) =>
    execFileSync("git", args, { cwd: path, stdio: "inherit" })
  git("init", "--quiet")
  git("fetch", "--quiet", "--depth", "1", props.repository, props.commit)
  git("checkout", "--quiet", "FETCH_HEAD")
  return path
}
