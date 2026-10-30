from textlane import html, md, textlane

from common import run_adapter

PROCESSOR = (
    textlane()
    .use(md.parse())
    .use(md.gfm())
    .use(md.to_html({"allowDangerousHtml": True}))
    .use(html.stringify({"allowDangerousHtml": True}))
    .freeze()
)


def convert(markdown: str) -> str:
    return PROCESSOR.process_sync(markdown)["value"]


if __name__ == "__main__":
    run_adapter(convert)
