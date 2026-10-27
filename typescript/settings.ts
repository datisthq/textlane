export interface Platform {
  /** The npm package that carries this platform's addon. */
  packageName: string
  /** The Rust target the addon is compiled for. */
  target: string
  /** The addon's file name at the package's root. */
  addon: string
}

/** The npm package users install; it pulls in the one platform package that matches. */
export const PACKAGE_NAME = "textlane"

/** An environment variable that points at an addon to load instead of the installed one. */
export const ADDON_ENV = "TEXTLANE_ADDON"

/** Every published platform, keyed by `${process.platform}-${process.arch}`. */
export const PLATFORMS: Record<string, Platform> = {
  "linux-x64": createPlatform("linux-x64", "x86_64-unknown-linux-gnu"),
  "linux-arm64": createPlatform("linux-arm64", "aarch64-unknown-linux-gnu"),
  "darwin-x64": createPlatform("darwin-x64", "x86_64-apple-darwin"),
  "darwin-arm64": createPlatform("darwin-arm64", "aarch64-apple-darwin"),
  "win32-x64": createPlatform("win32-x64", "x86_64-pc-windows-msvc"),
  "win32-arm64": createPlatform("win32-arm64", "aarch64-pc-windows-msvc"),
}

function createPlatform(key: string, target: string): Platform {
  return { packageName: `@textlane/${key}`, target, addon: "textlane.node" }
}
