"""The Python stand-ins for the suite's `host:<behaviour>[:<label>]` plugins."""

import asyncio


def create_hosts():
    """One function per name, so a name used again is one plugin reconfigured."""
    hosts = {}

    def host_for(name):
        if name not in hosts:
            hosts[name] = create_host(name[len("host:") :].split(":")[0])
        return hosts[name]

    return host_for


def create_host(behaviour):
    def attacher(processor, options=None):
        if behaviour == "recordOptions":
            recorded = processor.data("recorded")
            processor.data("recorded", [*(recorded if isinstance(recorded, list) else []), options])
            return None

        def transformer(tree, file):
            if (options or {}).get("mode") == "async":
                return run_later(behaviour, options, tree, file)
            return run_host(behaviour, options, tree, file)

        return transformer

    return attacher


async def run_later(behaviour, options, tree, file):
    await asyncio.sleep(0)
    return run_host(behaviour, options, tree, file)


def run_host(behaviour, options, tree, file):
    options = options or {}
    children = tree.setdefault("children", [])
    if behaviour == "appendNode":
        children.append(options["node"])
    elif behaviour == "replaceTree":
        return options["tree"]
    elif behaviour == "message":
        file.message(options["reason"], options.get("place"), options.get("ruleId"), options.get("source"))
    elif behaviour == "countMessages":
        children.append(paragraph(str(len(file.messages))))
    elif behaviour == "readValue":
        children.append(paragraph(file.value or ""))
    elif behaviour == "fail":
        raise failure(options)
    elif behaviour == "returnError":
        return failure(options)
    else:
        raise ValueError(f"no host behaviour `{behaviour}`")
    return tree


class PluginFailed(Exception):
    code = "plugin-failed"


def failure(options):
    return PluginFailed(options.get("message", ""))


def paragraph(value):
    return {"type": "paragraph", "children": [{"type": "text", "value": value}]}
