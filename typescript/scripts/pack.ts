import {
  cpSync,
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs"
import { join, resolve } from "node:path"
import { parseArgs } from "node:util"
import { PACKAGE_NAME, PLATFORMS } from "../settings.ts"

// Lays out every package `pnpm release` publishes under compile/: one per platform with
// its addon, and the main package whose optionalDependencies pin them at this version.
// `--addons` holds `<platform>/<addon>`, e.g. `linux-x64/textlane.node`.

const { values } = parseArgs({ options: { addons: { type: "string" } } })
if (!values.addons) throw new Error('usage: pnpm pack:platforms --addons "<dir>"')

const ROOT = resolve(import.meta.dirname, "..")
const COMPILE = join(ROOT, "compile")
const ADDONS = resolve(values.addons)
const manifest = JSON.parse(readFileSync(join(ROOT, "package.json"), "utf8"))
const version: string = manifest.version

rmSync(COMPILE, { recursive: true, force: true })

const missing = Object.entries(PLATFORMS)
  .map(([key, platform]) => join(ADDONS, key, platform.addon))
  .filter(path => !existsSync(path))
if (missing.length > 0) throw new Error(`missing addons:\n  ${missing.join("\n  ")}`)

for (const [key, platform] of Object.entries(PLATFORMS)) {
  const [os, cpu] = key.split("-")
  const target = join(COMPILE, platform.packageName)
  mkdirSync(target, { recursive: true })
  cpSync(join(ADDONS, key, platform.addon), join(target, platform.addon))
  cpSync(join(ROOT, "LICENSE.md"), join(target, "LICENSE.md"))
  writeJson(join(target, "package.json"), {
    name: platform.packageName,
    version,
    description: `The ${key} addon of ${PACKAGE_NAME}`,
    license: manifest.license,
    repository: manifest.repository,
    os: [os],
    cpu: [cpu],
    ...(platform.libc && { libc: [platform.libc] }),
    main: platform.addon,
    files: [platform.addon],
  })
}

const main = join(COMPILE, PACKAGE_NAME)
mkdirSync(main, { recursive: true })
cpSync(join(ROOT, "build"), join(main, "build"), { recursive: true })
for (const file of ["README.md", "LICENSE.md"]) cpSync(join(ROOT, file), join(main, file))
cpSync(join(ROOT, "..", ".skills"), join(main, "skills"), { recursive: true })
const { scripts, devDependencies, ...published } = manifest
writeJson(join(main, "package.json"), {
  ...published,
  optionalDependencies: Object.fromEntries(
    Object.values(PLATFORMS).map(platform => [platform.packageName, version]),
  ),
})

console.log(
  `packed ${Object.keys(PLATFORMS).length + 1} packages at ${version} into ${COMPILE}`,
)

function writeJson(path: string, value: object) {
  writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`)
}
