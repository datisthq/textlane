---
title: Memory
description: How much working memory textlane keeps between documents, and how to change it.
path: /python/memory/
icon: memory-stick
order: 11
---

# Memory

Each thread keeps the working memory a document's trees were built in, emptied, for the next
document, up to 256 MB; a document that needs more has its memory freed once it finishes.
Documents of several megabytes need more than that, so raise the cap for them, or lower it
when idle memory matters more than speed. It applies to every processor in the process.

```python
import textlane

textlane.set_retained_memory(512 * 1024**2)
textlane.retained_memory()
# 536870912
```

`textlane.set_retained_memory(0)` keeps nothing between documents.
