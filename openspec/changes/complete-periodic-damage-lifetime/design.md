# Design

Use the existing stat-buff transport, explicit source handle/profile and normal Outcome::Damage settlement. Integrate only min(positive dt, remaining lifetime), before timer retirement. Retain sub-quantum damage in each buff entry, sort target/source-key traversal and validate source integers without narrowing casts. Removing a buff removes its remainder. Refresh retains the accumulated fraction; changing rate affects only future elapsed time. Source retirement does not reattribute damage to a replacement generation. Existing tower magical DoT payloads remain supported.

Add one unit-target damage_over_time_enemy effect with bounded per-rank DPS/duration and explicit damage kind. Generate generic effect dispatch from Lua without hero-name branches. Same ability/source refreshes; different sources stack. Instant damage and bleed retain independent damage events. Sustained channels and standalone ground zones remain separate future work.
