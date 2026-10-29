from common import run_adapter
from markdown_it import MarkdownIt

if __name__ == "__main__":
    run_adapter(MarkdownIt("commonmark").render)
