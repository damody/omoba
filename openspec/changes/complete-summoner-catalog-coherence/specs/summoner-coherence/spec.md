# Summoner coherence

## ADDED Requirements

### Requirement: Compiled summoner catalog must match runtime authorship
Startup and developer reload MUST reject any summoner row, membership or ordering drift and explain that all peers need rebuilding.

#### Scenario: Rejected reload preserves active snapshot
- WHEN valid Lua changes a summoner value or identity
- THEN reload fails without changing active generation, hash or behavior
- AND subsequent supported ability damage tuning can still reload
