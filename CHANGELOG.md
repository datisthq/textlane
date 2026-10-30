# Changelog

## [0.2.0](https://github.com/datisthq/textlane/compare/v0.1.0...v0.2.0) (2026-10-30)


### Features

* add heading markers to md.headingId and a skip list to html.extractToc ([58f9233](https://github.com/datisthq/textlane/commit/58f92334ca856807467f97961aba86e895398199))
* **html:** run transformerMetaWordHighlight natively in html.highlight, on a port of Shiki's decorations ([903cf4e](https://github.com/datisthq/textlane/commit/903cf4e1968fbdbba9e392ade022f3e3d5287dfc))
* **md:** add md.pathRewrite, resolving asset and page URLs from the document's path, and list the new plugins on the landing ([c2f2383](https://github.com/datisthq/textlane/commit/c2f23835cabdb490c641ab156b55608683dff2d5))
* **md:** give task-list checkboxes an aria-label with md.toHtml's taskListLabels ([a459be5](https://github.com/datisthq/textlane/commit/a459be587badbf03defb65680a98af042ba05ac0))
* **md:** read a container directive's lists as a file tree with md.component's treeAs ([e954fc1](https://github.com/datisthq/textlane/commit/e954fc1d01c9f78430575d622cafcb68072cc772))


### Bug Fixes

* **html:** apply meta-word decorations from running lengths, at most 1,000 per line ([b291cbd](https://github.com/datisthq/textlane/commit/b291cbd41a441fb56a3521ea7b42967dd31b0028))
* **html:** leave lines of 2,000 characters or more plain in html.highlight, with Shiki's tokenizeMaxLineLength ([9570ce3](https://github.com/datisthq/textlane/commit/9570ce3efaef88fa783122094619b0d0f72d1b07))


### Performance Improvements

* write HTML straight from mdast when md.toHtml and html.stringify end a pipeline ([a357233](https://github.com/datisthq/textlane/commit/a357233f028d7f051bfd80e15e6cba1919071b29))

## 0.1.0 (2026-10-30)


### Features

* add html.externalLinks and attribute lists for md.headingId ([473b5ec](https://github.com/datisthq/textlane/commit/473b5ecef7bbc3ea6c735e65bba21fe8d5621060))
* add md.headingId, md.strip, html.unwrapImages, html.extractToc, code rules in md.component, file data and ANSI highlighting ([f0b05ae](https://github.com/datisthq/textlane/commit/f0b05aebfb445883c35aac5fcf3f79b8aaec3b38))
* add the core processor pipeline with unified's use, data and freeze ([4a65027](https://github.com/datisthq/textlane/commit/4a65027c44d998e600e06f148938e452d912096f))
* align the bindings' host plugin API and hold it to shared conformance cases ([4b8f1ae](https://github.com/datisthq/textlane/commit/4b8f1ae210c4163f67008b5d44072d41cd2ed85d))
* bootstrap the arena tree, the processor and Markdown to HTML ([45afd1e](https://github.com/datisthq/textlane/commit/45afd1eb2f7f5e98908eeb5f4e418c8b02a93094))
* convert HTML to Markdown and add GFM ([b708ce7](https://github.com/datisthq/textlane/commit/b708ce78360cd51b28cfb3afb61ba1611558d893))
* expose the processor in the Node and Python bindings ([400a7ab](https://github.com/datisthq/textlane/commit/400a7aba1531b1662acbc75782adbac4af586afd))
* give JavaScript and Python plugins vfile's file and fail runs on a returned error ([13a95f4](https://github.com/datisthq/textlane/commit/13a95f4c72985e86dab75e5c42d57b6df9c2e771))
* **html:** add html.highlight on giallo ([36501e7](https://github.com/datisthq/textlane/commit/36501e72f0a5e1385e1811b6cb8b4cc9d97aff8e))
* **html:** add html.katex, rendering math as rehype-katex does, natively on katex-rs ([ba1264f](https://github.com/datisthq/textlane/commit/ba1264f4366b42086eabd0eddacd0b326ea79749))
* **html:** add html.raw and html.format ([33700b3](https://github.com/datisthq/textlane/commit/33700b331d6da82013e556c1bfac80d65e01c32b))
* **html:** parse HTML as parse5 and hast-util-from-html do ([f0afe97](https://github.com/datisthq/textlane/commit/f0afe97db3bda438d09a4e0d776b991415a8a85e))
* **html:** run Shiki's notation, meta-highlight and line-number transformers, inline code and color replacements natively in html.highlight ([3716429](https://github.com/datisthq/textlane/commit/3716429538f0bbfaa45aedd534c144a521576722))
* **html:** sanitize and slug hast ([eb083fb](https://github.com/datisthq/textlane/commit/eb083fbd3050be71948f2c14eba97e8dd2ba5369))
* **html:** serialize every hast-util-to-html option ([b6c4083](https://github.com/datisthq/textlane/commit/b6c4083a9e7a82e7d8bc654cd40e3ceff88eaed0))
* let each thread keep up to 256 MB between documents, configurable with setRetainedMemory ([5f0e90c](https://github.com/datisthq/textlane/commit/5f0e90ccf1eaffffc0c5218b72a9bd121d7d9fa6))
* let JavaScript and Python plugins join a pipeline as transformers ([03244b1](https://github.com/datisthq/textlane/commit/03244b1fd6de460571f2ef3d642ea1ef50bea668))
* make each Rust plugin a struct of its options, replacing the constructor functions ([fcf4eb3](https://github.com/datisthq/textlane/commit/fcf4eb3ff38323bd0cdb49ca93bfb14e26195004))
* **md:** add math, directive, frontmatter and definition list extensions ([739178b](https://github.com/datisthq/textlane/commit/739178b77a6bc019d2e66f255d47b745f51cc832))
* **md:** add md.component, turning directives, alerts and nodes into MDX components by rules ([3aeebcd](https://github.com/datisthq/textlane/commit/3aeebcdaa8e8f4eb3547a64be167ed28c82a56e7))
* **md:** add md.toc, md.packageCommand and frontmatter data, with github-slugger in core ([0581318](https://github.com/datisthq/textlane/commit/05813185c9b6cce5650f9baa472229986e8cd633))
* **md:** apply unist data and passThrough in md.toHtml and fix parser gaps ([3a58dc6](https://github.com/datisthq/textlane/commit/3a58dc6559195c1ace0aa816e7bbb50c7af2330c))
* **md:** parse block quotes and lists ([2ea6249](https://github.com/datisthq/textlane/commit/2ea624935ba2502b078c7da4920e3533596b2e9f))
* **md:** parse character escapes, references, emphasis and hard breaks ([912462e](https://github.com/datisthq/textlane/commit/912462eb74c47ab8d05a8262c122a8d0c9a4096a))
* **md:** parse code spans, fenced code and indented code ([3770324](https://github.com/datisthq/textlane/commit/3770324ab87bdbc521c401ff932c57af8c011ef0))
* **md:** parse setext headings, links, definitions, autolinks and raw HTML ([28abcfe](https://github.com/datisthq/textlane/commit/28abcfe4ca5a78d5faa77d1bc3d9fda503b33a9c))
* **md:** serialize mdast to Markdown and replace gemoji shortcodes ([51d7831](https://github.com/datisthq/textlane/commit/51d78319b87f2f782a834223bbd0d14d07bf199b))
* **mdx:** add md.mdx and mdx.compile ([e5ad62d](https://github.com/datisthq/textlane/commit/e5ad62d00b5aa3da10c89ae51af23754821e8d60))
* **prose:** add prose parsing, retext plugins and the md and html bridges ([8ae05e0](https://github.com/datisthq/textlane/commit/8ae05e07f29a64a90ebf217f5a0c911fa00fd5f5))
* **python:** make process and run synchronous, with aprocess and arun as their coroutines ([2a6a963](https://github.com/datisthq/textlane/commit/2a6a96300cf44f84e137bccff32a88a99ce5734d))
* rename the prose domain to nl, with md.toNaturalLanguage and html.toNaturalLanguage ([95042ba](https://github.com/datisthq/textlane/commit/95042ba5ead3411d29bf4e76a1c5596c1d1e5028))


### Bug Fixes

* **html:** export ToNaturalLanguage without the highlight feature ([d9d9c84](https://github.com/datisthq/textlane/commit/d9d9c84cb80c976287c7414844a2bfc5b40ef925))
* **md:** match micromark on tabs, laziness, list ends and code spans ([4adf21f](https://github.com/datisthq/textlane/commit/4adf21fdff4c27428130830fc3dc46653996d248))
* **mdx:** add the mdx.compile plugin an ignore rule kept out ([1726e83](https://github.com/datisthq/textlane/commit/1726e83e1491894e8eb7e945a693ef8c2120aadc))


### Performance Improvements

* **md:** build trees at exact size and scan escapes with memchr ([23e3f93](https://github.com/datisthq/textlane/commit/23e3f930eedb1092213d8a4e1281b5b0b6751790))
* **md:** halve memory and speed up parsing 1.25–2× ([a87fa5b](https://github.com/datisthq/textlane/commit/a87fa5b29cefee61e4221a41b8113afebd892301))
* **md:** parse Markdown in one pass, MDX included ([1d05eee](https://github.com/datisthq/textlane/commit/1d05eee4f8914dbc08d07de6bed2ab507a85ca62))
* **md:** skip plain text and texts without literals under GFM ([fd73efe](https://github.com/datisthq/textlane/commit/fd73efec4b2c74a50f90f46ef49d41fa732eb64f))
* **md:** skip plain text in one run and find line ends with memchr ([d72f312](https://github.com/datisthq/textlane/commit/d72f312982f7a976b8d5f7d001fb58c9d09b1913))
* **md:** speed up parsing and HTML output 2–3× ([6a30a3f](https://github.com/datisthq/textlane/commit/6a30a3f295644badb8b855344047576a9f03a56e))

## Changelog
