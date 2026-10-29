import json
import sys
import time
from collections.abc import Callable


def run_adapter(convert: Callable[[str], str]) -> None:
    """Convert `<input> <repeat>` times and print `{elapsedMs, outputBytes}`; imports are untimed."""
    _, path, repeat = sys.argv
    with open(path, encoding="utf-8") as file:
        markdown = file.read()
    html = ""
    started = time.perf_counter()
    for _ in range(int(repeat)):
        html = convert(markdown)
    elapsed_ms = (time.perf_counter() - started) * 1000
    print(json.dumps({"elapsedMs": elapsed_ms, "outputBytes": len(html.encode())}))
