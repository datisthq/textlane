import markdown
from common import run_adapter

EXTENSIONS = [
    "tables",
    "footnotes",
    "pymdownx.tilde",
    "pymdownx.tasklist",
    "pymdownx.magiclink",
]

if __name__ == "__main__":
    # a Markdown instance keeps state between documents; reset() is what markdown() does
    converter = markdown.Markdown(extensions=EXTENSIONS)
    run_adapter(lambda text: converter.reset().convert(text))
