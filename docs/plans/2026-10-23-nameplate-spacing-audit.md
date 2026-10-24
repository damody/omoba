# Whole-game continuation audit

Continue the maintained backend/shared Rust/scripts and Unreal inventory in 2026-10-21-whole-game-completeness-audit.md. Existing diagonal maps, NPC behaviours, control/target/cast feedback, point blasts, swept dashes, normal-hit passives and strict transformations retain regression coverage.

This batch addresses a visible completeness gap observed in previous real native map captures: dense nonhero healthplates overlap despite a count cap. Add a common per-view bounded screen rectangle policy with selected/hovered/building priority, actual status-dependent Slate sizes, fresh projected anchors and recovery after suppression. Hero visibility and ground relation markers keep their existing policies.

Offline geometry, real Slate and current map captures validate this presentation change; no new real network match is needed. Sustained channels/zones, timed motion/residue, more specialized skill geometry, summon-group atomic commit, full combat replica lifecycle acceptance, wider content/art/audio and balancing remain pending. Existing MVP 33/44 and 11 external ACK/LAN/physical DPI/fixed-performance items stay pending. No omfx maintenance or commit/push.

Final native captures also show the next concrete presentation gap: hero rows keep their existing visibility and do not reserve space in this nonhero layout, so crossing hero/nonhero labels can remain. A future shared hero-obstacle/reservation policy should preserve critical hero health rather than hiding it, remain bounded under stress and use the same real-layout and ownership rules. This batch does not claim all nameplate collisions are solved.
