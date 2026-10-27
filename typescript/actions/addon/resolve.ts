import { ADDON_ENV, PACKAGE_NAME, PLATFORMS } from "../../settings.ts"

/** Find the textlane addon: the env override first, else the matching platform package. */
export function resolveAddon(props: {
  platform: string
  arch: string
  env: Record<string, string | undefined>
  resolvePackage: (specifier: string) => string
}) {
  const override = props.env[ADDON_ENV]
  if (override) return override

  const key = `${props.platform}-${props.arch}`
  const platform = PLATFORMS[key]
  if (!platform) {
    const supported = Object.keys(PLATFORMS).join(", ")
    throw new Error(
      `${PACKAGE_NAME} has no addon for ${key}; supported: ${supported}. ` +
        `Set ${ADDON_ENV} to an addon built with cargo to load it anyway.`,
    )
  }
  try {
    return props.resolvePackage(`${platform.packageName}/${platform.addon}`)
  } catch {
    throw new Error(
      `${platform.packageName} is not installed. Reinstall ${PACKAGE_NAME} without ` +
        `--no-optional / --omit=optional so the addon for ${key} comes along.`,
    )
  }
}
