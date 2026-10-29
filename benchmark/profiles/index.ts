import { proseProfile } from "./prose.ts"
import { specProfile } from "./spec.ts"
import type { Profile } from "./types.ts"

export const PROFILES: Profile[] = [proseProfile, specProfile]

export function getProfile(name: string) {
  const profile = PROFILES.find(item => item.name === name)
  if (!profile) {
    const known = PROFILES.map(item => item.name).join(", ")
    throw new Error(`Unknown profile ${name}; known: ${known}`)
  }
  return profile
}
