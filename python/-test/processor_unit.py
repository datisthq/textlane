import asyncio

import pytest
import textlane
from textlane import TextlaneError, html, md, mdx, nl


def to_html():
    return textlane.textlane().use(md.parse()).use(md.gfm()).use(md.to_html()).use(html.stringify())


def test_processes_markdown_to_html():
    file = asyncio.run(to_html().aprocess("# hi"))
    assert file == {"value": "<h1>hi</h1>", "messages": [], "data": {}}


def test_processes_synchronously():
    assert to_html().process("~a~")["value"] == "<p><del>a</del></p>"


def test_takes_a_list_tuples_and_names():
    processor = (
        textlane.textlane()
        .use([md.parse(), [md.to_html(), {"allowDangerousHtml": True}], "html.stringify"])
        .use(html.stringify({"allowDangerousHtml": True}))
    )
    assert processor.process("<b>a</b>")["value"] == "<p><b>a</b></p>"


def test_takes_a_preset_its_settings_becoming_data():
    processor = textlane.textlane().use(
        {"plugins": [md.parse(), md.stringify()], "settings": {"bullet": "+"}}
    )
    assert processor.process("- a")["value"] == "* a\n"
    assert processor.data("settings") == {"bullet": "+"}


def test_reconfigures_a_plugin_used_again():
    processor = (
        textlane.textlane()
        .use(md.parse())
        .use(md.stringify({"bullet": "+", "emphasis": "_"}))
        .use(md.stringify({"bullet": "*"}))
    )
    assert processor.process("- *a*")["value"] == "* _a_\n"


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
    assert processor.run(tree) == tree
    assert asyncio.run(processor.arun(tree, "*a*")) == tree
    assert processor.stringify(tree) == "*a*\n"


def test_reports_messages():
    processor = textlane.textlane().use(nl.parse()).use(nl.repeated_words()).use(nl.stringify())
    assert processor.process("the the")["messages"] == [
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
        asyncio.run(textlane.textlane().aprocess("a"))
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
        processor().run(tree)
    assert error.value.code == "async-in-sync"
    assert asyncio.run(processor().arun(tree)) == tree


def test_domain_modules_describe_plugins():
    assert md.to_html() == {"name": "md.toHtml"}
    assert md.to_html({"allowDangerousHtml": True}) == {
        "name": "md.toHtml",
        "options": {"allowDangerousHtml": True},
    }
    assert md.gemoji() == {"name": "md.gemoji"}
    assert nl.indefinite_article() == {"name": "nl.indefiniteArticle"}


def test_domain_modules_list_every_plugin():
    described = [
        getattr(module, name)()["name"]
        for module in (md, html, mdx, nl)
        for name in module.__all__
        if name[0].islower()
    ]
    assert sorted(described) == sorted(textlane.plugin_names())


def test_reports_the_version():
    assert textlane.version().count(".") == 2


def test_reads_back_the_retained_memory():
    assert textlane.retained_memory() == 256 * 1024**2
    textlane.set_retained_memory(0)
    assert textlane.retained_memory() == 0
    textlane.set_retained_memory(256 * 1024**2)
    with pytest.raises(OverflowError):
        textlane.set_retained_memory(-1)


def strong_emphasis(processor, options=None):
    def transformer(tree, file):
        visit(tree, lambda node: node.update(type="strong") if node["type"] == "emphasis" else None)

    return transformer


def visit(tree, visitor):
    visitor(tree)
    for child in tree.get("children", []):
        visit(child, visitor)


def test_runs_a_python_plugin_between_native_stages():
    processor = textlane.textlane().use(md.parse()).use(strong_emphasis).use(md.to_html()).use(html.stringify())
    assert processor.process("*a*")["value"] == "<p><strong>a</strong></p>"


def test_calls_the_attacher_with_the_processor_and_merged_options():
    seen = []

    def record(processor, options=None):
        seen.extend([options, processor.data("key")])

    processor = (
        textlane.textlane()
        .data("key", "value")
        .use(md.parse())
        .use(record, {"a": {"b": 1}})
        .use([[record, {"a": {"c": 2}}]])
        .use(md.stringify())
    )
    processor.process("a")
    assert seen == [{"a": {"b": 1, "c": 2}}, "value"]


def test_awaits_an_async_transformer():
    def replace(processor):
        async def transformer(tree, file):
            paragraph = {"type": "paragraph", "children": [{"type": "text", "value": "b"}]}
            return {"type": "root", "children": [paragraph]}

        return transformer

    processor = textlane.textlane().use([md.parse(), replace, md.stringify()])
    assert asyncio.run(processor.aprocess("a"))["value"] == "b\n"
    with pytest.raises(TextlaneError) as raised:
        processor.process("a")
    assert raised.value.code == "async-in-sync"


def test_keeps_the_messages_and_data_a_python_plugin_reports():
    def warn(processor):
        def transformer(tree, file):
            file.data["seen"] = True
            file.message("look", {"line": 1, "column": 1}, rule_id="rule", source="py")

        return transformer

    file = textlane.textlane().use([md.parse(), warn, md.stringify()]).process("a")
    assert file["data"] == {"seen": True}
    point = {"line": 1, "column": 1, "offset": None}
    place = {"start": point, "end": point}
    assert file["messages"] == [{"reason": "look", "ruleId": "rule", "source": "py", "place": place}]


def test_skips_a_python_plugin_turned_off():
    def fail(processor):
        def transformer(tree, file):
            raise RuntimeError("ran")

        return transformer

    processor = textlane.textlane().use([md.parse(), fail, md.stringify()]).use(fail, False)
    assert processor.process("a")["value"] == "a\n"


def test_fails_the_run_on_an_exception_a_transformer_returns():
    def fail(processor):
        return lambda tree, file: ValueError("returned")

    with pytest.raises(ValueError, match="returned"):
        textlane.textlane().use([md.parse(), fail, md.stringify()]).process("a")


def test_gives_python_plugins_the_files_path_and_keeps_it_in_the_result():
    seen = []

    def record(processor):
        def transformer(tree, file):
            seen.extend([file.path, file.dirname, file.stem, file.extname, str(file)])

        return transformer

    processor = textlane.textlane().use([md.parse(), record, md.stringify()])
    file = processor.process({"value": "a", "path": "docs/intro.md"})
    assert seen == ["docs/intro.md", "docs", "intro", ".md", "a"]
    assert file["path"] == "docs/intro.md"


def test_reports_python_messages_as_vfile_does():
    reported = []

    def report(processor):
        def transformer(tree, file):
            reported.append(file.message("a", {"line": 1, "column": 2}, rule_id="rule"))
            reported.append(file.info(ValueError("b")))
            file.fail("c", {"start": point(1, 1), "end": point(1, 3)})

        return transformer

    processor = textlane.textlane().use([md.parse(), report, md.stringify()])
    with pytest.raises(textlane.HostMessage) as raised:
        processor.process({"value": "abc", "path": "a.md"})
    assert (raised.value.fatal, raised.value.name, raised.value.file) == (True, "1:1-1:3", "a.md")
    assert (reported[0].fatal, reported[0].line, reported[0].column) == (False, 1, 2)
    assert (reported[1].fatal, reported[1].name, reported[1].reason) == (None, "1:1", "b")
    assert isinstance(reported[1].__cause__, ValueError)


def point(line, column):
    return {"line": line, "column": column, "offset": None}


def test_shows_python_plugins_what_native_stages_reported():
    counts = []

    def count(processor):
        def transformer(tree, file):
            counts.append(len(file.messages))
            file.message("from py")

        return transformer

    processor = textlane.textlane().use([md.parse(), ["test.message", {"reason": "native"}], count, md.stringify()])
    file = processor.process("a")
    assert counts == [1]
    assert [message["reason"] for message in file["messages"]] == ["native", "from py"]


def test_parses_a_document():
    assert textlane.textlane().use(md.parse()).parse({"value": "a"})["type"] == "root"
