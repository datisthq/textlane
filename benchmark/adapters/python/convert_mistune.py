import mistune
from common import run_adapter

PLUGINS = ["table", "strikethrough", "footnotes", "task_lists", "url"]

if __name__ == "__main__":
    markdown = mistune.create_markdown(escape=False, plugins=PLUGINS)
    run_adapter(lambda text: str(markdown(text)))
