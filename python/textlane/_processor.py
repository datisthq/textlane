"""unified's processor over the native pipeline; every call is the crate's."""

import json
from typing import Any, Dict, Optional, Union, overload

from . import _native
from ._models import Compatible, VFile

_UNSET: Any = object()


class Processor:
    """A pipeline: add plugins with `use`, then `process` text."""

    def __init__(self) -> None:
        self._native = _native.Pipeline()

    def use(self, value: Any, options: Any = None) -> "Processor":
        """Add a plugin, a list of them or a preset; a plugin used again is reconfigured."""
        own = None if options is None else json.dumps(options)
        self._native.use(json.dumps(value), own)
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
        self._native.freeze()
        return self

    def parse(self, text: str) -> Dict[str, Any]:
        """Parse text into a tree."""
        return json.loads(self._native.parse(text))

    async def run(self, tree: Dict[str, Any], file: Compatible = None) -> Dict[str, Any]:
        """Run the transforms, finishing asynchronous stages.

        `file`, the text `tree` was parsed from, lets bridges read it and keeps positions in
        the result.
        """
        return json.loads(self._native.run(json.dumps(tree), _read_value(file)))

    def run_sync(self, tree: Dict[str, Any], file: Compatible = None) -> Dict[str, Any]:
        """Run the transforms; fails with `async-in-sync` on an asynchronous stage."""
        return json.loads(self._native.run_sync(json.dumps(tree), _read_value(file)))

    def stringify(self, tree: Dict[str, Any]) -> str:
        """Compile a tree, running no transform."""
        return self._native.stringify(json.dumps(tree))

    async def process(self, text: str) -> VFile:
        """Parse, run and compile, finishing asynchronous stages."""
        return json.loads(self._native.process(text))

    def process_sync(self, text: str) -> VFile:
        """Parse, run and compile; fails with `async-in-sync` on an asynchronous stage."""
        return json.loads(self._native.process_sync(text))


def textlane() -> Processor:
    """A new processor."""
    return Processor()


def _read_value(file: Compatible) -> Optional[str]:
    if isinstance(file, dict):
        return file["value"]
    return file
