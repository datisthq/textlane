export interface Platform {
  /** The npm package that carries this platform's addon. */
  packageName: string
  /** The Rust target the addon is compiled for. */
  target: string
  /** The addon's file name at the package's root. */
  addon: string
  /** The C library a Linux addon links against; npm skips the package on the other one. */
  libc?: "glibc" | "musl"
}

/** The npm package users install; it pulls in the one platform package that matches. */
export const PACKAGE_NAME = "textlane"

/** An environment variable that points at an addon to load instead of the installed one. */
export const ADDON_ENV = "TEXTLANE_ADDON"

/** Every published platform, keyed by `${process.platform}-${process.arch}`, plus `-musl`. */
export const PLATFORMS: Record<string, Platform> = {
  "linux-x64": createPlatform("linux-x64", "x86_64-unknown-linux-gnu"),
  "linux-arm64": createPlatform("linux-arm64", "aarch64-unknown-linux-gnu"),
  "linux-x64-musl": createPlatform("linux-x64-musl", "x86_64-unknown-linux-musl"),
  "linux-arm64-musl": createPlatform("linux-arm64-musl", "aarch64-unknown-linux-musl"),
  "darwin-x64": createPlatform("darwin-x64", "x86_64-apple-darwin"),
  "darwin-arm64": createPlatform("darwin-arm64", "aarch64-apple-darwin"),
  "win32-x64": createPlatform("win32-x64", "x86_64-pc-windows-msvc"),
  "win32-arm64": createPlatform("win32-arm64", "aarch64-pc-windows-msvc"),
}

function createPlatform(key: string, target: string): Platform {
  const platform: Platform = {
    packageName: `@textlane/${key}`,
    target,
    addon: "textlane.node",
  }
  if (target.endsWith("-linux-gnu")) platform.libc = "glibc"
  if (target.endsWith("-linux-musl")) platform.libc = "musl"
  return platform
}
