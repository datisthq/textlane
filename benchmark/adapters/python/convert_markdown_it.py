from common import run_adapter
from markdown_it import MarkdownIt
from mdit_py_plugins.footnote import footnote_plugin
from mdit_py_plugins.tasklists import tasklists_plugin

if __name__ == "__main__":
    parser = MarkdownIt("gfm-like").use(footnote_plugin).use(tasklists_plugin)
    run_adapter(parser.render)
