# Design

Each drained pulse carries a checked fixed-point offset from this tick's start. For an entry with previous remainder r and interval i, due offsets are i-r, 2i-r, ... within min(dt, remaining). Collect bounded-by-existing-lifetime pulses, sort by due offset then source entity ID/generation and buff key, and expose that order to the existing buff system. Do not resample historical positions or interpolate movement; target sampling still uses the current simulation state. This orders zones relative to other zones, not all unrelated combat subsystems.

Use integer Q10 arithmetic and existing 30-second/quarter-second minimum bounds. Retain right-endpoint pulses, clock remainder, canonical simultaneous ties, nonpositive-dt no-op and source death retirement. Tests compare one catch-up interval with 15/60/120Hz partitions and verify actual ECS/outcome and formal 60Hz first-lethal credit.
