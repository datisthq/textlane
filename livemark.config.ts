import { defineConfig } from "livemark"

export default defineConfig({
  site: "https://textlane.dev",
  title: "Textlane",
  description: "Fast Markdown pipeline",
  logo: "/logo.svg",
  favicon: "/logo.png",
  include: [
    "python/README.md",
    "python/docs/**/*.md",
    "typescript/README.md",
    "typescript/docs/**/*.md",
    "rust/textlane/README.md",
    "rust/docs/**/*.md",
    "CONTRIBUTING.md",
    "CHANGELOG.md",
  ],
  skills: [".skills/*/SKILL.md"],
  sections: [
    { title: "Python", prefix: "/python/", icon: "code" },
    { title: "TypeScript", prefix: "/typescript/", icon: "code-xml" },
    { title: "Rust", prefix: "/rust/", icon: "cog" },
    {
      title: "Contributing",
      type: "custom",
      url: "/contributing/",
      icon: "heart-handshake",
      position: "sidebar",
    },
    {
      title: "Changelog",
      type: "custom",
      url: "/changelog/",
      icon: "history",
      position: "sidebar",
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
      file: "python/README.md",
      article: {
        title: "Introduction",
        description: "Install textlane for Python.",
        icon: "rocket",
        order: 1,
        path: "/python/introduction/",
      },
    },
    {
      file: "typescript/README.md",
      article: {
        title: "Introduction",
        description: "Install textlane for Node.js.",
        icon: "rocket",
        order: 1,
        path: "/typescript/introduction/",
      },
    },
    {
      file: "rust/textlane/README.md",
      article: {
        title: "Introduction",
        description: "Install textlane for Rust.",
        icon: "rocket",
        order: 1,
        path: "/rust/introduction/",
      },
    },
    {
      file: "CONTRIBUTING.md",
      article: {
        title: "Contributing",
        description:
          "How to report bugs and share ideas, and how team members work on textlane.",
        icon: "heart-handshake",
        order: -2,
        path: "/contributing/",
      },
    },
    {
      file: "CHANGELOG.md",
      article: {
        title: "Changelog",
        description: "What changed in each release.",
        icon: "history",
        order: -1,
        path: "/changelog/",
      },
    },
  ],
})
