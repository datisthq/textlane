from typing import Any, Iterator, List, Mapping, Optional, Union

import textlane
from textlane import hast, html, md, mdast, nl, nlcst

MARKDOWN = """---
title: a
...

# A *b* `c` ![d](e.png)

```py title="f.py"
g = 1
```

- [ ] h
  - i

| j | k |
| - | :-: |
| l | m |

Term
: Description

:::note{#n .o}
p :q[r] $s$
:::

$$
t
$$

<div>u</div>

[v]: https://w.example "x"

[^y]

[^y]: z
"""

MATTERS: md.FrontmatterOptions = [
    {"type": "yaml", "fence": "---"},
    {"type": "yaml", "fence": {"open": "---", "close": "..."}},
]


def parse_markdown():
    processor = (
        textlane.textlane()
        .use(md.parse())
        .use(md.frontmatter(MATTERS))
        .use(md.gfm())
        .use(md.math())
        .use(md.directive())
        .use(md.definition_list())
    )
    return processor.parse(MARKDOWN)


def test_parse_is_typed_as_the_parsers_root_and_walks_without_casts():
    tree: mdast.Root = parse_markdown()
    headings: List[str] = []
    blocks: List[Any] = []
    matter: Optional[str] = None
    for node in tree["children"]:
        if node["type"] == "heading":
            headings.append(to_string(node))
        elif node["type"] == "code":
            position = node.get("position")
            line = position["start"]["line"] if position else None
            blocks.append((node.get("lang"), node.get("meta"), node["value"], line))
        elif node["type"] == "yaml":
            matter = node["value"]
    assert matter == "title: a"
    assert headings == ["A b c d"]
    assert blocks == [("py", 'title="f.py"', "g = 1", 7)]


def test_trees_match_their_typeddicts():
    trees: List[Mapping[str, Any]] = [
        parse_markdown(),
        textlane.textlane().use(md.parse()).use(md.mdx()).parse(
            'import a from "b"\n\n<C d="e" {...f}>{g}</C>\n'
        ),
        textlane.textlane().use(html.parse()).parse("<!doctype html><p>a<!--b--></p>"),
        textlane.textlane().use(nl.parse()).parse("A b. C!"),
    ]
    modules = [mdast, mdast, hast, nlcst]
    for tree, module in zip(trees, modules):
        for node in walk(tree):
            check_node(node, module)


# mdast-util-to-string over the nodes a heading holds, narrowed by `type` alone
def to_string(node: Union[mdast.Heading, mdast.PhrasingContent]) -> str:
    if node["type"] == "image":
        return node.get("alt") or ""
    if (
        node["type"] == "text"
        or node["type"] == "inlineCode"
        or node["type"] == "inlineMath"
        or node["type"] == "html"
        or node["type"] == "mdxTextExpression"
    ):
        return node["value"]
    if (
        node["type"] == "break"
        or node["type"] == "footnoteReference"
        or node["type"] == "imageReference"
    ):
        return ""
    return "".join(to_string(child) for child in node["children"])


def walk(node: Mapping[str, Any]) -> Iterator[Mapping[str, Any]]:
    yield node
    for child in node.get("children", []):
        yield from walk(child)


# Every key a node has is one its TypedDict declares, and every key it requires is there
def check_node(node: Mapping[str, Any], module: Any) -> None:
    kind = node["type"]
    name = kind[0].upper() + kind[1:]
    if kind.startswith("defList"):
        name += "Node"
    if module is nlcst:
        name = kind[: -len("Node")]
    typed = getattr(module, name)
    keys = set(node)
    assert typed.__required_keys__ <= keys, (kind, typed.__required_keys__ - keys)
    assert keys <= typed.__required_keys__ | typed.__optional_keys__, (
        kind,
        keys - typed.__required_keys__ - typed.__optional_keys__,
    )
