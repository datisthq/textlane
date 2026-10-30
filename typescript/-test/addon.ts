import { execFileSync } from "node:child_process"
import { copyFileSync, rmSync } from "node:fs"
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

/** Remove a directory `buildAddon` filled, leaving it where the addon is still loaded. */
export function removeAddon(directory: string) {
  try {
    rmSync(directory, { recursive: true, force: true })
  } catch (error) {
    // Windows locks a loaded `.node` until the process exits, so the temp dir stays behind
    if (process.platform !== "win32") throw error
  }
}
