---
title: Agent Skills
description: Teach your coding agent to install textlane and call it from Python.
path: /python/agent-skills/
icon: sparkles
order: 13
---

# Agent Skills

Textlane ships an [Agent Skill](https://agentskills.io/) that tells coding agents such as
Claude Code, Cursor and Codex which package each language uses and what the API offers today.
Once installed, your agent knows not to invent pipeline calls that are not shipped yet.

| Skill            | Your agent uses it when                                         |
| ---------------- | --------------------------------------------------------------- |
| `textlane-usage` | installing textlane and calling it from Rust, Node.js or Python |

## Install from the website

textlane.dev publishes its skills under `/.well-known/agent-skills/`, so the
[`skills` CLI](https://github.com/vercel-labs/skills) can install them from the site URL. Run
this at your project root:

```bash
npx skills add https://textlane.dev
```

To skip the prompts, pass the skill and your agent as flags:

```bash
npx skills add https://textlane.dev --skill textlane-usage --agent claude-code
```

Add `--global` to install for your user instead of the project. Run `npx skills update` later
to pull newer versions.

## Install from the package

The skill also ships inside the wheel, installed to `share/textlane/skills` under your
environment's data directory. Installing it from there keeps it matched to the textlane
version you have, while the website always serves the latest release:

```bash
npx skills add "$(python -c "import sysconfig; print(sysconfig.get_path('data'))")/share/textlane/skills"
```

## Use it

You don't need to invoke the skill yourself. Agents read its description and load it when a
task matches, such as adding textlane to a project. If your agent supports invoking skills by
name, you can also call it directly, as with `/textlane-usage` in Claude Code.
