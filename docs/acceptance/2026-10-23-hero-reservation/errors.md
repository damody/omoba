# Errors and prevention

- Carry-forward: the previous nonhero-only layout left ordinary healthplates crossing hero labels. Reserve actual visible hero footprints with bounded work; do not infer hero visibility from prior desktop geometry or only entity kind.
- Retain the pure inner Slate layout factory from E1498. A temporary TakeWidget wrapper can release layout before measurement; do not reintroduce that initialization pattern or direct UMG calls from OmGenerated.
- Focused targets deliberately bypass hero reservations, but still obey nonhero spacing. Hero/hero collisions and overflow beyond the bounded 64 hero obstacles remain outside this batch; do not claim every crossing label is solved.
- Existing 48-hour heartbeat omoba-48 remains active until 2026-10-25 00:33:38 Taipei. Do not create another automation or reset its deadline.
- OpenSpec CLI unavailable in preceding work; use explicit proposal/design/spec/tasks under the user's authorized unattended scope.
- Diagnostic mistake: attempted to read hero-reservation-health.log before enumerating the actual filenames; the monitor writes hero-reservation-health.jsonl. No worker or production failure occurred. Used rg --files to locate the real file. Prevention: derive paths from the runner or file inventory before reading; do not guess log suffixes.

- Consolidated acceptance: Rust V1 1966 passed / 10 existing ignored / zero failed; four Lua contracts, backend V1, UE5.8 Build V1 and World V1 59 successes. Final 3407 source/assets set/SHA/size/mtime are identical before/after. No production changes during acceptance.
