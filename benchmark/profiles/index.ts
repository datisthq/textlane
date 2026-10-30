import { readmeProfile } from "./readme.ts"
import { conformanceProfile } from "./conformance.ts"
import type { Profile } from "./types.ts"

export const PROFILES: Profile[] = [readmeProfile, conformanceProfile]

export function getProfile(name: string) {
  const profile = PROFILES.find(item => item.name === name)
  if (!profile) {
    const known = PROFILES.map(item => item.name).join(", ")
    throw new Error(`Unknown profile ${name}; known: ${known}`)
  }
  return profile
}
