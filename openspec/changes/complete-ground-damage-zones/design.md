# Design

A checked DamageZoneSpec payload anchors a circle in world coordinates. Its BuffStore entry owns a pulse clock; first pulse occurs after one interval and the final exact-boundary pulse precedes expiry. Refresh replaces the anchor and restarts the clock. Removing the entry stops pulses. Pause supplies no game time. Source death/retirement cancels its zone, rather than relying on a stale entity generation. Distinct caster/ability keys coexist.

Every pulse samples current living enemy positions, excludes allies/self/buildings, sorts by full entity identity, and emits RawDamage with the authored profile. Existing defenses, shields and credit remain the single sink. Catch-up pulses use current tick geometry, with bounded duration/interval (maximum 120 pulses per active interval); this does not promise historical sub-tick movement reconstruction. Repeated explosions provide the existing public circle cue; no new dedicated zone art is claimed.

The scheduler uses the BuffStore reverse index, avoiding a per-frame scan of every entity when no zones exist. No hero/camp name checks. Flame assault uses shared point-blast admission plus optional checked zone extras, and the transaction captures all outputs before success.
