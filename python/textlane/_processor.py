"""unified's processor over the native pipeline; every call is the crate's, except that
Python plugins run here when a run pauses before them."""

import json
from typing import Any, Callable, Dict, List, Optional, Union, cast, overload

from . import _native
from ._host import Host, HostFile, finish, finish_sync, merge_options
from ._models import Compatible, VFile

_UNSET: Any = object()


class Processor:
    """A pipeline: add plugins with `use`, then `process` text."""

    def __init__(self) -> None:
        self._native = _native.Pipeline()
        self._ids: Dict[Callable[..., Any], int] = {}
        self._hosts: Dict[int, Host] = {}
        self._attached = False

    def use(self, value: Any, *options: Any) -> "Processor":
        """Add a plugin, a list of them or a preset; a plugin used again is reconfigured.

        A Python plugin is an attacher, called with the processor and its options at freeze:
        it returns a transformer taking the tree and the file.
        """
        if callable(value):
            self._native.use(json.dumps([self._register(value, list(options))]))
            return self
        own = None if not options or options[0] is None else json.dumps(options[0])
        self._native.use(json.dumps(self._map(value)), own)
        return self

    @overload
    def data(self) -> Dict[str, Any]: ...
    @overload
    def data(self, key: str) -> Any: ...
    @overload
    def data(self, key: str, value: Any) -> "Processor": ...
    @overload
    def data(self, key: Dict[str, Any]) -> "Processor": ...
    def data(self, key: Optional[Union[str, Dict[str, Any]]] = None, value: Any = _UNSET) -> Any:
        """All data or the value under `key`; with a value, set it; with a dict, replace all."""
        if isinstance(key, dict):
            self._native.replace_data(json.dumps(key))
            return self
        if key is not None and value is not _UNSET:
            self._native.set_data(key, json.dumps(value))
            return self
        return json.loads(self._native.data(key))

    def freeze(self) -> "Processor":
        """Attach every plugin; later `use` and `data` setters fail."""
        self._attach()
        self._native.freeze()
        return self

    def parse(self, document: Compatible) -> Dict[str, Any]:
        """Parse a document into a tree."""
        self._attach()
        return json.loads(self._native.parse(_read_value(document) or ""))

    async def arun(self, tree: Dict[str, Any], file: Compatible = None) -> Dict[str, Any]:
        """Run the transforms, finishing asynchronous stages.

        `file`, the text `tree` was parsed from, lets bridges read it and keeps positions in
        the result.
        """
        value = _read_value(file)
        if not self._attach():
            return json.loads(self._native.run(json.dumps(tree), value))
        run = self._native.start_run(json.dumps(tree), value, False)
        await finish(run, self._hosts, HostFile(value, _read_path(file)))
        return json.loads(run.result())

    def run(self, tree: Dict[str, Any], file: Compatible = None) -> Dict[str, Any]:
        """Run the transforms; fails with `async-in-sync` on an asynchronous stage."""
        value = _read_value(file)
        if not self._attach():
            return json.loads(self._native.run_sync(json.dumps(tree), value))
        run = self._native.start_run(json.dumps(tree), value, True)
        finish_sync(run, self._hosts, HostFile(value, _read_path(file)), "run")
        return json.loads(run.result())

    def stringify(self, tree: Dict[str, Any]) -> str:
        """Compile a tree, running no transform."""
        self._attach()
        return self._native.stringify(json.dumps(tree))

    async def aprocess(self, document: Compatible) -> VFile:
        """Parse, run and compile a document, finishing asynchronous stages."""
        text = _read_value(document) or ""
        if not self._attach():
            return _with_path(_read_file(*self._native.process(text, _read_path(document))), document)
        run = self._native.start_process(text, False, _read_path(document))
        file = HostFile(run.value() or text, _read_path(document))
        await finish(run, self._hosts, file)
        return _with_path(_merge_data(json.loads(run.result()), file.data), document)

    def process(self, document: Compatible) -> VFile:
        """Parse, run and compile a document; fails with `async-in-sync` on an asynchronous
        stage."""
        text = _read_value(document) or ""
        if not self._attach():
            return _with_path(_read_file(*self._native.process_sync(text, _read_path(document))), document)
        run = self._native.start_process(text, True, _read_path(document))
        file = HostFile(run.value() or text, _read_path(document))
        finish_sync(run, self._hosts, file, "process")
        return _with_path(_merge_data(json.loads(run.result()), file.data), document)

    # unified's addPlugin: the same attacher is one plugin, its dict options merged
    def _register(self, attacher: Callable[..., Any], parameters: List[Any]) -> List[Any]:
        id = self._ids.get(attacher)
        if id is None:
            id = len(self._ids)
            self._ids[attacher] = id
            self._hosts[id] = Host(attacher, parameters)
            return [{"host": id}, not parameters or parameters[0] is not False]
        host = self._hosts[id]
        if not parameters:
            return [{"host": id}]
        primary, *rest = parameters
        current = host.parameters[0] if host.parameters else None
        if isinstance(current, dict) and isinstance(primary, dict):
            primary = merge_options(current, primary)
        host.parameters = [primary, *rest]
        return [{"host": id}, primary is not False]

    def _map(self, value: Any) -> Any:
        if callable(value):
            return self._register(value, [])
        if isinstance(value, list):
            return [self._map_entry(entry) for entry in value]
        return self._map_preset(value)

    def _map_entry(self, entry: Any) -> Any:
        if callable(entry):
            return self._register(entry, [])
        if isinstance(entry, (list, tuple)) and entry and callable(entry[0]):
            return self._register(entry[0], list(entry[1:]))
        return self._map_preset(entry)

    def _map_preset(self, value: Any) -> Any:
        if isinstance(value, dict) and isinstance(value.get("plugins"), list):
            return {**value, "plugins": [self._map_entry(entry) for entry in value["plugins"]]}
        return value

    # Attachers run once, before the native pipeline freezes, so they can still set data;
    # whether any Python plugin is on decides if runs pause for them
    def _attach(self) -> bool:
        if not self._attached:
            self._attached = True
            for id in self._native.hosts():
                host = self._hosts.get(id)
                if host is None:
                    continue
                parameters = list(host.parameters)
                if parameters and parameters[0] is True:
                    parameters[0] = None
                transformer = host.attacher(self, *parameters)
                if callable(transformer):
                    host.transformer = transformer
        return len(self._native.hosts()) > 0


def textlane() -> Processor:
    """A new processor."""
    return Processor()


def _read_value(file: Compatible) -> Optional[str]:
    if isinstance(file, dict):
        return file["value"]
    return file


def _read_path(file: Compatible) -> Optional[str]:
    if isinstance(file, dict):
        return file.get("path")
    return None


# Native stages' data first, then what Python plugins stored over it
def _merge_data(result: VFile, data: Dict[str, Any]) -> VFile:
    return {**result, "data": {**result["data"], **data}}


# The extension hands back the text apart from the file's messages and data
def _read_file(value: str, rest: str) -> VFile:
    return cast(VFile, {"value": value, **json.loads(rest)})


def _with_path(file: VFile, document: Compatible) -> VFile:
    path = _read_path(document)
    return file if path is None else {**file, "path": path}
