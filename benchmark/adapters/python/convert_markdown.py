import markdown
from common import is_gfm, run_adapter

GFM_EXTENSIONS = [
    "tables",
    "footnotes",
    "pymdownx.tilde",
    "pymdownx.tasklist",
    "pymdownx.magiclink",
]

if __name__ == "__main__":
    # a Markdown instance keeps state between documents; reset() is what markdown() does
    converter = markdown.Markdown(extensions=GFM_EXTENSIONS if is_gfm() else [])
    run_adapter(lambda text: converter.reset().convert(text))
