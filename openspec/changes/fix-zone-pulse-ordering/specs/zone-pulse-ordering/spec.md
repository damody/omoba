# Zone pulse ordering

## Requirement: actual due offset precedes source identity
When multiple valid zones pulse in one drain, emit earliest due pulses first. Equal times use source ID, generation and buff key. Offset must be positive and within the active interval; lifetime expiry includes a pulse exactly at its endpoint. Repartitioning time preserves absolute due times and ordering for stationary state.

## Requirement: existing combat admission remains authoritative
The production buff tick consumes this order without reordering. Existing live source checks, hostile live target sampling, defenses, shields, first-lethal guards and kill attribution apply. Death/removal cancels future source-bound zones. No hidden movement reconstruction, new network match or global cross-subsystem chronology is introduced.
