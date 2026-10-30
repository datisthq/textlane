import { defineConfig } from "livemark"

export default defineConfig({
  site: "https://textlane.dev",
  title: "Textlane",
  description: "Fast text processing",
  logo: "/logo.svg",
  favicon: "/logo.png",
  include: [
    "python/docs/**/*.md",
    "typescript/docs/**/*.md",
    "rust/docs/**/*.md",
    "python/CONTRIBUTING.md",
    "typescript/CONTRIBUTING.md",
    "rust/CONTRIBUTING.md",
  ],
  skills: [".skills/*/SKILL.md"],
  sections: [
    { title: "Python", prefix: "/python/", icon: "code" },
    { title: "TypeScript", prefix: "/typescript/", icon: "code-xml" },
    { title: "Rust", prefix: "/rust/", icon: "cog" },
    {
      title: "Changelog",
      type: "changelog",
      prefix: "/changelog/",
      source: "CHANGELOG.md",
      version: true,
      icon: "history",
    },
    {
      title: "GitHub",
      type: "custom",
      url: "https://github.com/datisthq/textlane",
      icon: "github",
    },
  ],
  patches: [
    {
      file: "python/CONTRIBUTING.md",
      article: {
        title: "Contributing",
        description:
          "How to report bugs and share ideas, and how team members work on textlane.",
        icon: "heart-handshake",
        order: -1,
        path: "/python/contributing/",
      },
    },
    {
      file: "typescript/CONTRIBUTING.md",
      article: {
        title: "Contributing",
        description:
          "How to report bugs and share ideas, and how team members work on textlane.",
        icon: "heart-handshake",
        order: -1,
        path: "/typescript/contributing/",
      },
    },
    {
      file: "rust/CONTRIBUTING.md",
      article: {
        title: "Contributing",
        description:
          "How to report bugs and share ideas, and how team members work on textlane.",
        icon: "heart-handshake",
        order: -1,
        path: "/rust/contributing/",
      },
    },
  ],
})
