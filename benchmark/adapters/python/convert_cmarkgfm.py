import cmarkgfm
from cmarkgfm.cmark import Options
from common import run_adapter

if __name__ == "__main__":
    run_adapter(lambda text: cmarkgfm.markdown_to_html(text, options=Options.CMARK_OPT_UNSAFE))
