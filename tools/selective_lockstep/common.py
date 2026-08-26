from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any


def load_json(path: str) -> Any:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def write_result(path: str | None, value: Any) -> None:
    text = json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n"
    if path:
        Path(path).write_text(text, encoding="utf-8", newline="\n")
    else:
        print(text, end="")


def sha256_file(path: str) -> str:
    digest = hashlib.sha256()
    with Path(path).open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()
