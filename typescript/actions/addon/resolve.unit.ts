import { describe, expect, it } from "vite-plus/test"
import { resolveAddon } from "./resolve.ts"

describe("resolveAddon", () => {
  it("resolves the platform package's addon", () => {
    const addon = resolveAddon({
      platform: "linux",
      arch: "x64",
      env: {},
      resolvePackage: specifier => `/node_modules/${specifier}`,
    })
    expect(addon).toBe("/node_modules/@textlane/linux-x64/textlane.node")
  })

  it("resolves the same file name on Windows", () => {
    const addon = resolveAddon({
      platform: "win32",
      arch: "arm64",
      env: {},
      resolvePackage: specifier => specifier,
    })
    expect(addon).toBe("@textlane/win32-arm64/textlane.node")
  })

  it("prefers the env override", () => {
    const addon = resolveAddon({
      platform: "plan9",
      arch: "mips",
      env: { TEXTLANE_ADDON: "/opt/textlane.node" },
      resolvePackage: () => "",
    })
    expect(addon).toBe("/opt/textlane.node")
  })

  it("names the supported platforms for an unknown one", () => {
    const run = () =>
      resolveAddon({
        platform: "plan9",
        arch: "mips",
        env: {},
        resolvePackage: () => "",
      })
    expect(run).toThrow(/no addon for plan9-mips; supported: linux-x64/)
  })

  it("explains a missing optional dependency", () => {
    const run = () =>
      resolveAddon({
        platform: "darwin",
        arch: "arm64",
        env: {},
        resolvePackage: () => {
          throw new Error("Cannot find module")
        },
      })
    expect(run).toThrow(/@textlane\/darwin-arm64 is not installed/)
  })
})
