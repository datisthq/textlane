---
title: Memory
description: How much working memory textlane keeps between documents, and how to change it.
path: /typescript/memory/
icon: memory-stick
order: 11
---

# Memory

Each thread keeps the working memory a document's trees were built in, emptied, for the next
document, up to 256 MB; a document that needs more has its memory freed once it finishes.
Documents of several megabytes need more than that, so raise the cap for them, or lower it
when idle memory matters more than speed. It applies to every processor in the process.

```js
import { retainedMemory, setRetainedMemory } from "textlane"

setRetainedMemory(512 * 1024 ** 2)
retainedMemory()
// 536870912
```

`setRetainedMemory(0)` keeps nothing between documents.
