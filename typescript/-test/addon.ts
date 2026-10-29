import { execFileSync } from "node:child_process"
import { copyFileSync } from "node:fs"
import { join, resolve } from "node:path"

const RUST = resolve(import.meta.dirname, "..", "..", "rust")

const LIBRARY_NAMES: Record<string, string> = {
  darwin: "libtextlane.dylib",
  linux: "libtextlane.so",
  win32: "textlane.dll",
}

/** Build the addon with the `testing` feature and copy it into `directory`. */
export function buildAddon(directory: string) {
  execFileSync("cargo", ["build", "-p", "textlane", "--features", "node,testing"], {
    cwd: RUST,
    stdio: "inherit",
  })
  const library = LIBRARY_NAMES[process.platform]
  if (!library) throw new Error(`no cargo library name known for ${process.platform}`)
  const addonPath = join(directory, "textlane.node")
  copyFileSync(join(RUST, "target", "debug", library), addonPath)
  return addonPath
}
