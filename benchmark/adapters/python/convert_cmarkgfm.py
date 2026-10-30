import cmarkgfm
from cmarkgfm.cmark import Options
from common import run_adapter

OPTIONS = Options.CMARK_OPT_UNSAFE | Options.CMARK_OPT_FOOTNOTES

if __name__ == "__main__":
    run_adapter(
        lambda text: cmarkgfm.github_flavored_markdown_to_html(text, options=OPTIONS)
    )
