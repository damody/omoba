# TD 1–100 Reference Policy Decisions

This fixture records the deterministic choices made by `AutoplayController`.
It is descriptive; the executable source remains authoritative.

- Map and state: `TD_GREEN_CROSSROADS`, player 1, generated default seed,
  campaign heroes removed before tick 1, knowledge bonuses disabled.
- Simulation: 15 simulation ticks/game-second (`66.667ms` represented by the
  exact fixed-point remainder schedule), uncapped wall-clock execution, no
  sleep. Measured throughput is informational and never a pass/fail condition.
- Placement: 45 fixed legal candidates in row-major order. Desired tower count
  is `min(round + 2, 45)` and general placement rotates Cake, Boomerang, Dart.
- Upgrade legality: every candidate passes the production three-path validator.
  Preferred primary paths are Dart path 2, Cake path 1, and Boomerang path 1;
  ties use cost, entity id, path, then level.
- `EstablishDefense`: place one Dart when no tower exists.
- `AddCamoDetection`: before a Camo round, advance a Dart on path 0 to level 2.
- `CoverImmunities`: before Black/White/Purple/Zebra/Lead/DDT mixes, ensure a
  Normal-profile Cake tower exists.
- `CounterMoab`: before MOAB-class rounds, ensure a Boomerang exists, then
  advance its path 1 to level 4.
- `CounterRegrowFortified`: buy the cheapest legal preferred upgrade.
- `CounterLeakRisk`: at 25 lives or fewer, spend available cash on the cheapest
  legal preferred upgrade before starting another round.
- `GeneralInvestment`: build toward the round-scaled tower count; afterward buy
  a legal upgrade only while preserving 100 cash.
- Unaffordable selected tower: start the round and preserve cash instead of
  spending it on an upgrade that can permanently starve tower count.
- Rejected placement: keep the formal rejection atomic, advance to the next
  fixed candidate, and retry at most 45 unique positions; never move the tower
  directly or bypass road/overlap validation.
- `CastReadyAbility`: during a running round, cast the first ready ability by
  stable entity order. Otherwise submit no input and wait.
- `StartRound`: only while idle after all higher-priority branches decline.
- Forbidden shortcuts: no debug spawn, direct cash/lives/HP/round/cooldown
  mutation, instant damage, invulnerability, hero combat, or skipped wave.
- Every submitted placement, upgrade, ability, and start-round input is checked
  after the production apply phase; a silently rejected input fails the run.
- Completion requires round 100 combat-path victory, positive lives, zero
  remaining creeps/projectiles after bounded quiescence, zero unattributed layer
  cash, cash conservation, contiguous ledger serials, and deterministic replay
  of round-end ticks, ledger digest, and final state hash.
