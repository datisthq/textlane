from common import is_gfm, run_adapter
from markdown_it import MarkdownIt
from mdit_py_plugins.footnote import footnote_plugin
from mdit_py_plugins.tasklists import tasklists_plugin

if __name__ == "__main__":
    if is_gfm():
        parser = MarkdownIt("gfm-like").use(footnote_plugin).use(tasklists_plugin)
    else:
        parser = MarkdownIt("commonmark")
    run_adapter(parser.render)
