# Zone pulse retirement

## Requirement: late pulses recheck exact living source
A pulse whose source has died, been removed, lost faction/health, or appears in the current removal set must emit no damage or success cue. Cancel its exact-generation source-bound zones. A reused numeric ID cannot revive old queued pulses or cancel a replacement generation's zones.

## Requirement: serial sampling and ordinary settlement
Each valid pulse samples current live hostile non-building targets in its anchored circle, excluding pending removals, in canonical entity order. Raw damage children settle before the next scheduled pulse. Existing defenses/shields and first-lethal attribution apply once. Invalid payloads do not mutate healthy zone owners. Managed script casts cannot insert host-only deferred pulse envelopes. Paused scheduling, due-time ordering and source-bound duration semantics remain unchanged.
