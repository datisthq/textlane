import markdown
from common import run_adapter

if __name__ == "__main__":
    # a Markdown instance keeps state between documents; reset() is what markdown() does
    converter = markdown.Markdown()
    run_adapter(lambda text: converter.reset().convert(text))
