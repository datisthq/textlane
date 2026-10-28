import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs"
import { dirname } from "node:path"

type Group = { description: string; source?: { repository: string } }

/** Replace one upstream repository's groups in a test file, keeping every other source's. */
export function writeGroups(props: {
  path: string
  repository: string
  groups: Group[]
}) {
  const existing: Group[] = existsSync(props.path)
    ? JSON.parse(readFileSync(props.path, "utf8"))
    : []
  const isOwn = (group: Group) => group.source?.repository === props.repository
  const firstOwn = existing.findIndex(isOwn)
  const others = existing.filter(group => !isOwn(group))
  // Own groups go back where they were, so re-importing one source never reorders a file
  const at = firstOwn === -1 ? others.length : firstOwn
  const merged = [...others.slice(0, at), ...props.groups, ...others.slice(at)]
  mkdirSync(dirname(props.path), { recursive: true })
  writeFileSync(props.path, `${JSON.stringify(merged, null, 2)}\n`)
}
