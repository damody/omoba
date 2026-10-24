# Healing reduction errors and prevention

H01: HealReceivedMultiplier was declared but ignored by direct/native/script healing. Use one checked recipient policy; regen applies it once, not again during writeback.

H02: Existing generic product aggregation iterates HashMap and may overflow on hostile values. Healing uses bounded factors, canonical source/family ordering and checked wide intermediates. Invalid or overflowing policy yields zero healing.

H03: Full camp reset and fountain recovery are restoration. Do not apply combat antiheal to completed camp reset or infer a new effect from item identity. The authored optional component is the only trigger.

H04: A read-only search used an unexpanded Windows path wildcard. Reissued searches against concrete directories; no mutations occurred. Use rg directory roots and -g filters.

H05: One combined exact patch placed an earlier codegen hunk after a later one and was rejected before mutation. Reapplied each observed block separately. Checked the targeted-use parameter name (`active`) before freezing sources.

H06: V1 core test compilation found a missing BuffStore import in the extended jungle fixture. Production UE/backend builds are separate. Preserve the failure logs and wait for every V1 consumer/watcher to retire before adding the explicit import.

H07: Inspection found summoner self-heal commits HP directly and caps to missing HP during planning. Applying reduction only in the sink would incorrectly reduce already-capped healing. Include the checked recipient multiplier in the authoritative cast context and plan identity, apply it before the HP cap, and revalidate it at commit. A repeated Windows wildcard read also failed; subsequent searches use concrete runtime roots.

H08: V2 core passed897 and failed one legacy assertion that explicitly expected ScriptDirectDamage50 against30 HP followed by ScriptHeal15 to resurrect the unit to15. The new documented rule forbids resurrection via ordinary healing. After all V2 consumers/watchers retire, update that assertion to0 and add an alive-target damage/heal control, preserving the intended adapter test rather than weakening the new rule. Preserve the V2 failure and all successful production results.

H09: V2 base test compilation reported E0422 because the newly added formal summoner input used an unimported protobuf SummonerCast. Use the fully qualified generated type rather than relying on the fixture's narrow import list. No production type or wire change is needed.

H10: The inventory helper used a literal50% in a Lua gsub replacement string; Lua rejected it before writing the inventory. Use a replacement function for arbitrary prose, and escape literal percent signs inside string.format report templates before finalization.

H11: V3 Rust passed1782/10. WorldV1 completed40 tests but the new tooltip test assumed integer strings50% and4 seconds. Existing OmPlayerItemText::Number intentionally renders at least one localized fractional digit (50.0%／4.0 seconds). Metadata and authority were correct. After source capture/check and watchers retire, make the test's expected numeric tokens use the same documented localized number format; retain V3 Rust/backend results because only the C++ test changes, and rerun UE build/world plus fresh source/staging guards. Preserve failed WorldV1 report and logs.
