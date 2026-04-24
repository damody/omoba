#!/usr/bin/env python3
"""
migrate_sk_callers.py

解析 scripts/script-abi/src/stat_keys.rs 的 `StatKey::Foo => "foo_bar"` match arm，
建立 SNAKE (const 舊名，全大寫) → PascalCase (新 variant 名) 對照，
再對 D:/omoba/omb/src/ 裡的 .rs 批次把 `sk::SNAKE` 換成 `StatKey::Pascal`。

設計：
- Import 不自動補。檔頭已經有 `use omb_script_abi::stat_keys as sk;` 的檔案
  另外由主流程 sed 換成 `use omb_script_abi::stat_keys::StatKey;`。
- 僅替換 `sk::XXX` + `stat_keys::XXX`（全大寫 SNAKE）兩種 pattern。
- 保留 `sk::BUFF_ID_*` / `stat_keys::BUFF_ID_*` / `stat_keys::StatKey`。
"""

import re
import sys
from pathlib import Path

STAT_KEYS_RS = Path("D:/omoba/scripts/script-abi/src/stat_keys.rs")
OMB_SRC = Path("D:/omoba/omb/src")

MATCH_ARM_RE = re.compile(
    r'StatKey::(?P<pascal>\w+)\s*=>\s*"(?P<wire>[a-z0-9_]+)"'
)


def build_mapping():
    """回傳 {SNAKE_UPPER: PascalCase}"""
    text = STAT_KEYS_RS.read_text(encoding="utf-8")
    mapping = {}
    for m in MATCH_ARM_RE.finditer(text):
        pascal = m.group("pascal")
        wire = m.group("wire")
        snake_upper = wire.upper()
        mapping[snake_upper] = pascal
    return mapping


# 兩種 caller 型態：
#   sk::FOO_BAR / stat_keys::FOO_BAR        →  StatKey::FooBar
#   sk::FOO_BAR.into() / stat_keys::FOO_BAR.into()  →  StatKey::FooBar.as_str().into()
# 先處理 .into() 尾綴（更長 pattern 優先）
SK_CALLER_INTO_RE = re.compile(r"\bsk::([A-Z][A-Z0-9_]+)\.into\(\)")
STATKEYS_CALLER_INTO_RE = re.compile(r"\bstat_keys::([A-Z][A-Z0-9_]+)\.into\(\)")
SK_CALLER_RE = re.compile(r"\bsk::([A-Z][A-Z0-9_]+)\b")
STATKEYS_CALLER_RE = re.compile(r"\bstat_keys::([A-Z][A-Z0-9_]+)\b")

# 不要動的白名單
SKIP_NAMES = {
    "BUFF_ID_STUN",
    "BUFF_ID_ROOT",
    "BUFF_ID_SILENCE",
    "BUFF_ID_INVISIBLE",
    "BUFF_ID_INVULNERABLE",
    "StatKey",  # regex 已排除（大寫起頭的 SNAKE only）
}


def migrate_file(path: Path, mapping: dict) -> int:
    """回傳此檔替換了幾處。"""
    original = path.read_text(encoding="utf-8")
    count = 0
    missing = set()

    def sk_into_repl(m):
        nonlocal count
        name = m.group(1)
        if name in SKIP_NAMES:
            return m.group(0)
        if name not in mapping:
            missing.add(name)
            return m.group(0)
        count += 1
        return f"StatKey::{mapping[name]}.as_str().into()"

    def sk_repl(m):
        nonlocal count
        name = m.group(1)
        if name in SKIP_NAMES:
            return m.group(0)
        if name not in mapping:
            missing.add(name)
            return m.group(0)
        count += 1
        return f"StatKey::{mapping[name]}"

    # 先 .into() 尾綴
    new = SK_CALLER_INTO_RE.sub(sk_into_repl, original)
    new = STATKEYS_CALLER_INTO_RE.sub(sk_into_repl, new)
    # 再處理裸 const 引用
    new = SK_CALLER_RE.sub(sk_repl, new)
    new = STATKEYS_CALLER_RE.sub(sk_repl, new)

    # 換 import — 僅當本檔確實產生過替換
    if count > 0:
        new = re.sub(
            r"use\s+omb_script_abi::stat_keys\s+as\s+sk\s*;",
            "use omb_script_abi::stat_keys::StatKey;",
            new,
        )

    if new != original:
        path.write_text(new, encoding="utf-8")

    if missing:
        print(f"  WARN {path}: unmapped names: {sorted(missing)}")

    return count


def main():
    mapping = build_mapping()
    print(f"Loaded {len(mapping)} StatKey mappings.")

    total = 0
    changed_files = 0
    for rs in OMB_SRC.rglob("*.rs"):
        n = migrate_file(rs, mapping)
        if n:
            rel = rs.relative_to(OMB_SRC)
            print(f"  {rel}: {n} replacements")
            total += n
            changed_files += 1

    print(f"Done: {total} replacements across {changed_files} files.")


if __name__ == "__main__":
    main()
