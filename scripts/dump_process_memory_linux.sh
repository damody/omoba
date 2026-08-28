#!/usr/bin/env bash
set -u
pid="$1"; expected="$2"; output="$3"
actual="$(readlink -f "/proc/$pid/exe" 2>/dev/null || true)"
expected="$(readlink -f "$expected" 2>/dev/null || true)"
status=UNVERIFIED; reason=""
if [[ -z "$actual" || "$actual" != "$expected" ]]; then reason="PID executable mismatch";
elif command -v gcore >/dev/null 2>&1 && gcore -o "$output" "$pid" >/dev/null 2>&1; then output="${output}.${pid}"; status=CAPTURED;
else reason="gcore unavailable or ptrace denied"; fi
sha="$(sha256sum "$expected" 2>/dev/null | awk '{print $1}')"
printf '{"schema_version":1,"pid":%s,"status":"%s","method":"gcore","binary_path":"%s","binary_sha256":"%s","dump_path":"%s","reason":"%s"}\n' "$pid" "$status" "$expected" "$sha" "$output" "$reason" > "${output}.json"
[[ "$status" == CAPTURED ]]

