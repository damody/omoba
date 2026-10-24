# Shield presentation errors and prevention

S01: Existing `__damage_shield` was intentionally internal and absent from the public catalog. Renamed the sole stored identity to `damage_shield` and append-only registered it; retain combat amount/duration bounds and existing snapshot visibility. Do not invent shield amounts from HP changes.

S02: One read-only PowerShell search containing nested double-quote escapes failed to parse. Reissued using simple single-quoted patterns; no mutations occurred. Avoid shell-dependent escaping around search expressions.

S03: Decode fixed-point raw units explicitly (1024), require integral bounded values and authoritative finite duration. UI width caps at max HP but text preserves full amount; no loss of combat shield value.

S04: A fixed-height Slate box still reserves height when only its child is collapsed. Collapse the shield bar's containing box too, so units without shields retain their existing nameplate dimensions.

S05: V1 world export and bridge test compilation reported E0599: the new test used `OmBuffEventKind::OwnerRemoved`; the actual ABI enum is `BuffOwnerRemoved`. This is a test symbol error; UE/backend production builds succeeded. Preserve V1 logs and wait for every V1 consumer, source capture and watcher to retire before editing the test. Verify enum declarations before copying event terminology into code.
