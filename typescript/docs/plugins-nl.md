---
title: Natural language plugins
description: Every natural language plugin, its options and the unified package it follows.
path: /typescript/natural-language-plugins/
icon: languages
order: 9
---

# Natural language plugins

Natural-language plugins parse prose into nlcst and check or rewrite it. The checks report messages on the file and leave the text as it is. Reach them from Markdown through `md.toNaturalLanguage` or from HTML through `html.toNaturalLanguage`.

## nl.parse

Parses natural language into an nlcst tree of paragraphs, sentences, words and punctuation.

```js
import { textlane } from "textlane"
import * as nl from "textlane/nl"

const file = textlane()
  .use(nl.parse())
  .use(nl.smartypants())
  .use(nl.stringify())
  .processSync('"Hi" -- she said...')
file.value
// “Hi” — she said…
```

<!-- generated:options nl.parse -->

Follows [retext-english](https://www.npmjs.com/package/retext-english).

| Option     | Type       | Default | Description                                 |
| ---------- | ---------- | ------- | ------------------------------------------- |
| `language` | `Language` |         | The rules sentences and words are found by. |

**`Language`**

The language `nl.parse` reads: parse-latin's rules, or parse-english's on top of them.

Takes `"latin" | "english"`.

<!-- /generated -->

## nl.equality

Warns about insensitive, inconsiderate wording.

```js
import { textlane } from "textlane"
import * as nl from "textlane/nl"

const file = textlane()
  .use(nl.parse())
  .use(nl.equality())
  .use(nl.stringify())
  .processSync("Hey guys, the chairman is here.")
file.messages[0].reason
// Unexpected potentially insensitive use of `guys`, in somes cases `people`, ` …
```

<!-- generated:options nl.equality -->

Follows [retext-equality](https://www.npmjs.com/package/retext-equality).

| Option   | Type               | Default | Description                                                          |
| -------- | ------------------ | ------- | -------------------------------------------------------------------- |
| `ignore` | `string[] \| null` | `null`  | Phrases not to warn about.                                           |
| `binary` | `boolean \| null`  | `null`  | Whether a phrase used next to its counterpart (`he or she`) is fine. |

<!-- /generated -->

## nl.passive

Warns about the passive voice.

```js
import { textlane } from "textlane"
import * as nl from "textlane/nl"

const file = textlane()
  .use(nl.parse())
  .use(nl.passive())
  .use(nl.stringify())
  .processSync("The cake was eaten.")
file.messages[0].reason
// Unexpected use of the passive voice
```

<!-- generated:options nl.passive -->

Follows [retext-passive](https://www.npmjs.com/package/retext-passive).

| Option   | Type               | Default | Description                    |
| -------- | ------------------ | ------- | ------------------------------ |
| `ignore` | `string[] \| null` | `null`  | Participles not to warn about. |

<!-- /generated -->

## nl.readability

Warns about sentences too hard to read for a target age.

```js
import { textlane } from "textlane"
import * as nl from "textlane/nl"

const file = textlane()
  .use(nl.parse())
  .use(nl.readability({ age: 12 }))
  .use(nl.stringify())
  .processSync(
    "The constitutionally mandated intergovernmental reconciliation procedures necessitate comprehensive multilateral deliberation.",
  )
file.messages[0].reason
// Unexpected hard to read sentence, according to 6 out of 7 algorithms
```

<!-- generated:options nl.readability -->

Follows [retext-readability](https://www.npmjs.com/package/retext-readability).

| Option      | Type             | Default | Description                                                                 |
| ----------- | ---------------- | ------- | --------------------------------------------------------------------------- |
| `age`       | `number \| null` | `null`  | The age of the target audience: 16 by default.                              |
| `threshold` | `number \| null` | `null`  | The share of algorithms that must find a sentence too hard: 4/7 by default. |
| `minWords`  | `number \| null` | `null`  | The fewest words a sentence needs to be checked: 5 by default.              |

<!-- /generated -->

## nl.repeatedWords

Warns about a word written twice in a row.

```js
import { textlane } from "textlane"
import * as nl from "textlane/nl"

const file = textlane()
  .use(nl.parse())
  .use(nl.repeatedWords())
  .use(nl.stringify())
  .processSync("It is the the best.")
file.messages[0].reason
// Unexpected repeated `the`, remove one occurrence
```

<!-- generated:options nl.repeatedWords -->

Follows [retext-repeated-words](https://www.npmjs.com/package/retext-repeated-words).

No options.

<!-- /generated -->

## nl.sentenceSpacing

Warns about the spaces between sentences: one by default.

```js
import { textlane } from "textlane"
import * as nl from "textlane/nl"

const file = textlane()
  .use(nl.parse())
  .use(nl.sentenceSpacing())
  .use(nl.stringify())
  .processSync("One.  Two.")
file.messages[0].reason
// Unexpected 2 spaces between sentence, expected 1 space
```

<!-- generated:options nl.sentenceSpacing -->

Follows [retext-sentence-spacing](https://www.npmjs.com/package/retext-sentence-spacing).

| Option      | Type              | Default | Description                                                                                                          |
| ----------- | ----------------- | ------- | -------------------------------------------------------------------------------------------------------------------- |
| `preferred` | `Spacing \| null` |         | The spacing expected between sentences: `1` or `"space"` (the default), `2` or `"double-space"`, `0` or `"newline"`. |

**`Spacing`**

A spacing between sentences, as a count of spaces or by name.

Takes `number | string`.

<!-- /generated -->

## nl.stringify

Serializes nlcst back into text.

```js
import { textlane } from "textlane"
import * as nl from "textlane/nl"

const file = textlane().use(nl.parse()).use(nl.stringify()).processSync("Same text out.")
file.value
// Same text out.
```

<!-- generated:options nl.stringify -->

Follows [retext-stringify](https://www.npmjs.com/package/retext-stringify).

No options.

<!-- /generated -->

## nl.smartypants

Turns straight quotes, dashes, dots and backticks into their typographic forms.

```js
import { textlane } from "textlane"
import * as nl from "textlane/nl"

const file = textlane()
  .use(nl.parse())
  .use(nl.smartypants())
  .use(nl.stringify())
  .processSync('"It\'s" -- fine...')
file.value
// “It’s” — fine…
```

<!-- generated:options nl.smartypants -->

Follows [retext-smartypants](https://www.npmjs.com/package/retext-smartypants).

| Option          | Type                      | Default | Description                                                                                                                                            |
| --------------- | ------------------------- | ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `backticks`     | `Backticks`               |         | Turn double backticks and double straight single quotes into double quotes; `all` turns single ones into single quotes too, and needs `quotes: false`. |
| `closingQuotes` | `QuoteCharacters \| null` |         | The closing quotes to use: `”` and `’` by default.                                                                                                     |
| `dashes`        | `Dashes`                  |         | Turn two dashes into an em dash; `oldschool` and `inverted` treat three too.                                                                           |
| `ellipses`      | `Ellipses`                |         | Turn three dots into an ellipsis, with spaces between them or without.                                                                                 |
| `openingQuotes` | `QuoteCharacters \| null` |         | The opening quotes to use: `“` and `‘` by default.                                                                                                     |
| `quotes`        | `boolean \| null`         | `true`  | Turn straight quotes into smart ones.                                                                                                                  |

**`Backticks`**

Takes `boolean | "all"`.

**`QuoteCharacters`**

The quote characters to use for double and single quotes.

| Option   | Type     | Default | Description |
| -------- | -------- | ------- | ----------- |
| `double` | `string` |         |             |
| `single` | `string` |         |             |

**`Dashes`**

Takes `boolean | "oldschool" | "inverted"`.

**`Ellipses`**

Takes `boolean | "spaced" | "unspaced"`.

<!-- /generated -->

## nl.quotes

Warns about quotes and apostrophes that are not the preferred kind, smart by default.

```js
import { textlane } from "textlane"
import * as nl from "textlane/nl"

const file = textlane()
  .use(nl.parse())
  .use(nl.quotes())
  .use(nl.stringify())
  .processSync('He said "hi".')
file.messages[0].reason
// Unexpected straight quote `"`, expected `“`
```

<!-- generated:options nl.quotes -->

Follows [retext-quotes](https://www.npmjs.com/package/retext-quotes).

| Option      | Type                 | Default | Description                                                                                                  |
| ----------- | -------------------- | ------- | ------------------------------------------------------------------------------------------------------------ |
| `preferred` | `QuoteStyle \| null` |         | The style of quotes and apostrophes to prefer: `smart` by default.                                           |
| `smart`     | `string[] \| null`   | `null`  | Smart quotes by nesting level, each one character or an opening and closing pair: `["“”", "‘’"]` by default. |
| `straight`  | `string[] \| null`   | `null`  | Straight quotes by nesting level: `["\"", "'"]` by default.                                                  |

**`QuoteStyle`**

A style of quotes: straight (`"`) or smart (`“”`).

Takes `"smart" | "straight"`.

<!-- /generated -->

## nl.contractions

Warns about missing or misplaced apostrophes in contractions.

```js
import { textlane } from "textlane"
import * as nl from "textlane/nl"

const file = textlane()
  .use(nl.parse())
  .use(nl.contractions())
  .use(nl.stringify())
  .processSync("Dont do that.")
file.messages[0].reason
// Unexpected missing apostrophe in `Dont`, expected `Don’t`
```

<!-- generated:options nl.contractions -->

Follows [retext-contractions](https://www.npmjs.com/package/retext-contractions).

| Option          | Type              | Default | Description                                                     |
| --------------- | ----------------- | ------- | --------------------------------------------------------------- |
| `allowLiterals` | `boolean \| null` | `false` | Also check contractions written as literals, such as `“hasnt”`. |
| `straight`      | `boolean \| null` | `false` | Expect straight apostrophes (`'`) instead of smart ones (`’`).  |

<!-- /generated -->

## nl.indefiniteArticle

Warns about `a` and `an` used before the wrong sound.

```js
import { textlane } from "textlane"
import * as nl from "textlane/nl"

const file = textlane()
  .use(nl.parse())
  .use(nl.indefiniteArticle())
  .use(nl.stringify())
  .processSync("It is a apple.")
file.messages[0].reason
// Unexpected article `a` before `apple`, expected `an`
```

<!-- generated:options nl.indefiniteArticle -->

Follows [retext-indefinite-article](https://www.npmjs.com/package/retext-indefinite-article).

No options.

<!-- /generated -->
