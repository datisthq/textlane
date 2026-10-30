import cmarkgfm
from cmarkgfm.cmark import Options
from common import is_gfm, run_adapter

if __name__ == "__main__":
    if is_gfm():
        options = Options.CMARK_OPT_UNSAFE | Options.CMARK_OPT_FOOTNOTES
        convert = cmarkgfm.github_flavored_markdown_to_html
    else:
        options = Options.CMARK_OPT_UNSAFE
        convert = cmarkgfm.markdown_to_html
    run_adapter(lambda text: convert(text, options=options))
