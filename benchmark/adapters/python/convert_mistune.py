import mistune
from common import is_gfm, run_adapter

GFM_PLUGINS = ["table", "strikethrough", "footnotes", "task_lists", "url"]

if __name__ == "__main__":
    plugins = GFM_PLUGINS if is_gfm() else []
    markdown = mistune.create_markdown(escape=False, plugins=plugins)
    run_adapter(lambda text: str(markdown(text)))
