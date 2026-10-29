import mistune
from common import run_adapter

if __name__ == "__main__":
    markdown = mistune.create_markdown(escape=False, plugins=[])
    run_adapter(lambda text: str(markdown(text)))
