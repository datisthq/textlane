import { defineConfig } from "livemark"

export default defineConfig({
  site: "https://textlane.dev",
  title: "Textlane",
  description: "Rust-native text processing pipeline",
  include: ["README.md", "CONTRIBUTING.md", "CHANGELOG.md", "rust/docs/**/*.md"],
  skills: [".skills/*/SKILL.md"],
  sections: [
    {
      title: "GitHub",
      type: "custom",
      url: "https://github.com/datisthq/textlane",
      icon: "github",
    },
  ],
  patches: [
    {
      file: "README.md",
      article: {
        title: "Introduction",
        description: "What textlane is and how to install it.",
        icon: "rocket",
        order: 1,
        path: "/introduction/",
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
