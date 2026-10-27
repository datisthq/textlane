import { createRequire } from "node:module"
import { resolveAddon } from "./actions/addon/resolve.ts"

const require = createRequire(import.meta.url)

const addon: { version: () => string } = require(
  resolveAddon({
    platform: process.platform,
    arch: process.arch,
    env: process.env,
    resolvePackage: specifier => require.resolve(specifier),
  }),
)

/** The package version, as the native crate reports it. */
export const version = addon.version
