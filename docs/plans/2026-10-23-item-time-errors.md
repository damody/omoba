# Item cooldown readability errors and prevention

Preparation: attempted guessed OmEditor plugin/test and prep filenames; reads failed with path-not-found and changed no files. Corrected by rg --files discovery. Prevention: use discovered paths exactly, including the OmRuntime/Source/OmEditor nesting.

First native patch failed verification on a whitespace-different summary-derived line; apply_patch was atomic and changed no files. Read actual function and used assert-checked literal Lua replacements prepared before writing. Prevention: never substitute summary pseudocode for exact source text. No readers were running during edits.

OpenSpec CLI absent in existing environment; following documented direct artifact fallback, no toolchain installation. No runtime/build failure accepted.

WorldV1:82tests executed; new six-slot cases passed, ResponsiveHudLayout failed retained exact 2.0秒 /2.0/5.0秒 expectations. Root cause: intended common scalar trims trailing.0, existing fixture encoded previous formatting. After all4readers retired/reaped, preserved original fixture SHA bound to source-before-v1 and changed only expected literals to2秒 /2/5秒. All semantic/tooltips/geometry/font assertions retained. New3inputs-v2/fullbuild/fullnative/source required; original failed evidence kept. Prevention: search presentation fixtures for exact old numeric strings when intentionally changing shared formatting; never accept an earlier passing cohort after a repair.

V2 helper audit before launch caught prefix-relative v1 references left by full-path-only replacement. Corrected all version references in unlaunched V2 readers and checked Lua syntax. BuildV2 already uses exactV2 self/log; formal source unchanged. Prevention: inspect both literal full paths and prefix-relative suffixes before launching derived helpers.

During read-only next-gap inventory, discovered nameplate header but also searched a guessed companion.cpp that does not exist. No source changes. This repeated the preparation path mistake; prevention tightened: only pass rg --files returned exact paths to subsequent content searches, never infer a .cpp beside a discovered header.
