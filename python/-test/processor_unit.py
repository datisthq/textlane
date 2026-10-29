import asyncio

import pytest
import textlane
from textlane import TextlaneError, html, md, mdx, prose


def to_html():
    return textlane.textlane().use(md.parse()).use(md.gfm()).use(md.to_html()).use(html.stringify())


def test_processes_markdown_to_html():
    file = asyncio.run(to_html().process("# hi"))
    assert file == {"value": "<h1>hi</h1>", "messages": [], "data": {}}


def test_processes_synchronously():
    assert to_html().process_sync("~a~")["value"] == "<p><del>a</del></p>"


def test_takes_a_list_tuples_and_names():
    processor = (
        textlane.textlane()
        .use([md.parse(), [md.to_html(), {"allowDangerousHtml": True}], "html.stringify"])
        .use(html.stringify({"allowDangerousHtml": True}))
    )
    assert processor.process_sync("<b>a</b>")["value"] == "<p><b>a</b></p>"


def test_takes_a_preset_its_settings_becoming_data():
    processor = textlane.textlane().use(
        {"plugins": [md.parse(), md.stringify()], "settings": {"bullet": "+"}}
    )
    assert processor.process_sync("- a")["value"] == "* a\n"
    assert processor.data("settings") == {"bullet": "+"}


def test_reconfigures_a_plugin_used_again():
    processor = (
        textlane.textlane()
        .use(md.parse())
        .use(md.stringify({"bullet": "+", "emphasis": "_"}))
        .use(md.stringify({"bullet": "*"}))
    )
    assert processor.process_sync("- *a*")["value"] == "* _a_\n"


def test_ignores_none():
    textlane.textlane().use(None)


def test_gets_sets_and_replaces_data():
    processor = textlane.textlane().data("a", 1)
    assert processor.data("a") == 1
    assert processor.data() == {"a": 1}
    processor.data({"b": [2]})
    assert processor.data() == {"b": [2]}
    assert processor.data("missing") is None


def test_fails_to_change_a_frozen_processor():
    processor = textlane.textlane().use(md.parse()).freeze()
    with pytest.raises(TextlaneError) as error:
        processor.use(md.gfm())
    assert error.value.code == "frozen-processor"
    with pytest.raises(TextlaneError) as error:
        processor.data("a", 1)
    assert error.value.code == "frozen-processor"


def test_parses_runs_and_stringifies():
    processor = textlane.textlane().use(md.parse()).use(md.stringify())
    tree = processor.parse("*a*")
    assert tree["type"] == "root"
    assert processor.run_sync(tree) == tree
    assert asyncio.run(processor.run(tree, "*a*")) == tree
    assert processor.stringify(tree) == "*a*\n"


def test_reports_messages():
    processor = textlane.textlane().use(prose.parse()).use(prose.repeated_words()).use(prose.stringify())
    assert processor.process_sync("the the")["messages"] == [
        {
            "reason": "Unexpected repeated `the`, remove one occurrence",
            "ruleId": "the",
            "source": "retext-repeated-words",
            "place": {
                "start": {"line": 1, "column": 1, "offset": 0},
                "end": {"line": 1, "column": 8, "offset": 7},
            },
        }
    ]


def test_raises_errors_carrying_the_crates_code():
    with pytest.raises(TextlaneError) as error:
        textlane.textlane().use("md.missing")
    assert error.value.code == "unknown-plugin"
    with pytest.raises(TextlaneError) as error:
        asyncio.run(textlane.textlane().process("a"))
    assert error.value.code == "missing-parser"
    assert str(error.value) == "Cannot `process` without `parser`"
    with pytest.raises(TextlaneError) as error:
        textlane.textlane().use(md.parse()).use("md.gfm", {"singleTilde": "no"}).freeze()
    assert error.value.code == "invalid-options"


def test_finishes_asynchronous_stages_only_in_the_async_api():
    def processor():
        return textlane.textlane().use("test.parseText").use("test.noop", {"mode": "async"})

    tree = {"type": "root", "children": [{"type": "text", "value": "a"}]}
    with pytest.raises(TextlaneError) as error:
        processor().run_sync(tree)
    assert error.value.code == "async-in-sync"
    assert asyncio.run(processor().run(tree)) == tree


def test_domain_modules_describe_plugins():
    assert md.to_html() == {"name": "md.toHtml"}
    assert md.to_html({"allowDangerousHtml": True}) == {
        "name": "md.toHtml",
        "options": {"allowDangerousHtml": True},
    }
    assert md.gemoji() == {"name": "md.gemoji"}
    assert prose.indefinite_article() == {"name": "prose.indefiniteArticle"}


def test_domain_modules_list_every_plugin():
    described = [
        getattr(module, name)()["name"]
        for module in (md, html, mdx, prose)
        for name in module.__all__
        if name[0].islower()
    ]
    assert sorted(described) == sorted(textlane.plugin_names())


def test_reports_the_version():
    assert textlane.version().count(".") == 2
