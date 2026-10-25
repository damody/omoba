# Summon arrival movement ownership
The accepted active-destination guard ends when summon_tick removes MoveTarget on arrival. Preserve host movement ownership until the end of that simulation tick so a second script chase cannot consume another step after arriving.
