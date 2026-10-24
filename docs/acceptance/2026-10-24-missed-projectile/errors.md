# Errors and prevention

- Edit V1 preserved all 10 original inputs, then refused the first formal write because path.write defaults to no overwrite. No formal input changed. V2 explicitly enables overwrite only for the named formal files; preserved originals remain immutable. Read the helper contract before invoking it.
- Edit V2 wrote only ProjectileOrigin and Outcome, then stopped before writing game_processor because its test constructor uses a newline after ScriptProjectile. V3 resumes only the eight unwritten files, matching the actual basic_attack field. Original helpers are retained. Prevention: read all constructor layouts before building exact replacement scripts; never rerun a partial edit over already modified inputs.

No acceptance reader was active during these edits. Original source bytes and SHA-256 are retained in missed-projectile-old/manifest.json.

- Read-only progress probes requested health.json instead of the actual health.jsonl and attempted to read source-before before capture published it. No source or reader changed. Discover receipt/log names with rg --files; inspect the current worker CPU/log progress while the snapshot is still being built, and parse the complete snapshot only after successful capture.
