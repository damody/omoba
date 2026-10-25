# Plan

1. Add a distinct typed summon ABI payload (ABI36) and renderer intent (presentation12, selective14 unchanged), carrying both opaque source lives plus connection/view scope. Never reuse hero target, direction axes or inventory fields.
2. Client input gateway validates exact current hero/summon disclosures and commands capability; bridge validates current published live own hero/summon and connection before allocating/queueing.
3. UE model builds clean typed events; selected state marks only current exact owned live summons commandable. Feed current selection to ActionOwner, clear it on retirement/context reset and leave ordinary hero clicks unchanged.
4. Add optional configurable Alt+RightMouse move and Alt+S release rows using existing conflict-preserving default migration. Route through existing owner focus/modal/world sampling and show actual binding guidance for commandable selection. Do not cancel hero cast/channel.
5. Port active bridge build/codegen/header/staging logic to fixed clone-local Lua; current UE build still calls historical batch/PowerShell logic, which must not be used for this batch. Keep old launcher files untouched, replace active Lua entry calls with Lua implementation.
6. Add Rust roundtrip/current-scope/foreign/stale/alias/old-ABI tests and native typed input/selection/chord/Slate coverage; run concentrated maintained builds/native captures/source checks, archive all failures and update tasks/audit/handoff.
