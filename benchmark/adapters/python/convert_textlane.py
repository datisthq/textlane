import os

from textlane import html, md, textlane

from common import is_gfm, run_adapter


def create_processor():
    processor = textlane().use(md.parse())
    if is_gfm():
        processor = processor.use(md.gfm())
    stringify = {"allowDangerousHtml": True}
    # What any HTML plugin forces, as a no-op: a non-default option takes the hast path
    if os.environ.get("TEXTLANE_HAST"):
        stringify["quote"] = '"'
    return (
        processor.use(md.to_html({"allowDangerousHtml": True}))
        .use(html.stringify(stringify))
        .freeze()
    )


if __name__ == "__main__":
    processor = create_processor()
    run_adapter(lambda markdown: processor.process(markdown)["value"])
