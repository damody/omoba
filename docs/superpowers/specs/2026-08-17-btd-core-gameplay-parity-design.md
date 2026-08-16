# BTD-Style Core Gameplay Parity Design

## Goal

Make Omoba Tower Defense feel strategically similar to modern Bloons Tower
Defense without copying its art, characters, names, live-service breadth, or
content volume. The work focuses on the gameplay grammar that creates the
recognizable decisions: layered enemies, damage immunities, round cadence,
per-layer cash, leak consequences, cross-path upgrades, map rules, and clear
combat information.

This is a master design. It must be implemented as several bounded OpenSpec
changes rather than one monolithic change. The first change covers enemy
layers, properties, damage compatibility, cash, and leak damage.

## Current Baseline

Omoba already has substantial TD infrastructure:

- nine selectable TD maps;
- seven towers with three paths and four tiers per path;
- tower placement, road clearance, ownership, upgrades, selling, target
  priorities, active abilities, pop counters, pause, speed, and auto-start;
- player-owned cash, lives, manual round starts, and a 100-round BTD-shaped
  catalog;
- deterministic lockstep, a two-client development launcher, and a 10,000
  entity stress path;
- seventeen knowledge nodes and two authored heroes.

The principal mismatch is beneath that surface. Generated BTD enemies are
single RPG-style health bars. Camo and Regrow are retained in generated names
but discarded when runtime emitters are built. Fortified only doubles the
flattened health value. Damage-type immunities, child layers, and per-layer
cash do not exist. Every leak removes one life. Every expanded enemy uses the
same 0.18-second interval. Generated enemies fall through to a generic
10-cash bounty, while a table representing round cash is also credited at
round completion. Consequently the visible round list resembles BTD, but the
combat, timing, and economy decisions do not.

## Scope Decisions

### Product Identity

The target is an original Omoba TD with BTD-style systemic depth. Existing
Omoba tower identities, maps, story, Rust deterministic runtime, and Lua
content pipeline remain. New visual art, monetization, seasons, public content
sharing, and matching another game's roster size are outside scope.

UI state, textual combat information, input feedback, and sound-event hooks
are gameplay presentation and remain in scope even though visual art
production does not.

### Deferred Modes and Cooperation

There is no Phase 6 in this program. The following are deferred:

- Sandbox, Reverse, Half Cash, alternate-round, boss, and linked-map modes;
- formal two- or four-player co-op;
- lobby, matchmaking, reconnect, cash transfer, and shared round voting.

The basic 40-, 60-, 80-, and 100-round rule profiles remain because they are
core difficulty configurations, not additional modes. Existing multiplayer
ownership and lockstep code must not be broken, but feature work on co-op is
not part of this design.

## Enemy Model

### Layer Graph

Replace flattened effective health with a data-authored layer graph. Each
enemy archetype defines:

- current-layer health;
- ordered child archetypes and child counts;
- movement speed;
- leak damage;
- cash awarded for removing the layer;
- damage vulnerabilities and immunities;
- Camo, Regrow, Fortified, ordinary-layer, and MOAB-class properties.

Damage first removes the current layer, then carries remaining damage through
children. A one-child transition reuses and transforms the existing entity.
Branching transitions create only the children that survive the same hit.
This avoids creating and immediately deleting every intermediate layer while
preserving the observable child structure.

All child ordering is authored and deterministic. No simulation result may
depend on hash-map iteration order. A single hit produces a single ordered
layer-resolution outcome containing removed layers, awarded cash, surviving
children, and the final entity state.

### Properties

- Camo enemies cannot be selected, tracked, or hit by sources without Camo
  detection.
- Regrow enemies climb back toward their authored maximum layer on fixed
  simulation time. Removing the property stops regeneration.
- Fortified applies only to eligible layers and propagates only to eligible
  children.
- Composite enemies such as DDT-like units are expressed through properties
  and tags rather than unit-specific control flow.

### Damage Compatibility

Add a TD `DamageProfile` with initial tags `Sharp`, `Explosive`, `Energy`,
`Fire`, `Cold`, `Normal`, `Crushing`, and `True`. Enemy layers author accepted
or rejected tags. Tower projectiles, direct-area attacks, damage-over-time,
and active abilities carry the same profile through the normal outcome path.

Lead-like, Black-like, White-like, and Purple-like immunities use these tags.
Upgrades can grant Camo detection or add/replace damage tags. MOBA physical
armor and magic resistance remain available to MOBA mode and are not used as
an approximation for TD immunity.

## Economy, Pops, and Lives

Cash is awarded for each removed layer. A tower's pop count also counts layers
instead of only final entity deaths. An escaped enemy removes lives according
to its remaining layer graph, so different enemies have different leak
consequences.

The generic 10-cash fallback must not apply to generated TD enemies. The
current round-cash table must not be credited as an additional round-clear
payment. Round completion has its own independently authored bonus. All
credits and debits emit deterministic economy-ledger entries so a test can
reconcile starting cash, layer cash, round bonuses, purchases, upgrades,
sales, and ending cash exactly.

Tower sellback becomes one mode-authored multiplier applied consistently to
base and upgrade spend. The present mixed 85-percent base and 75-percent
upgrade refund is removed.

## Rounds and Difficulty

### Group Cadence

A round contains ordered spawn groups. Every group defines enemy archetype,
count, start offset, interval, path-selection policy, and whether it overlaps
the previous group. This supports spaced groups, dense rushes, intentional
gaps, overlapping types, and synchronized multi-lane pressure. Runtime must no
longer flatten every round into a universal 0.18-second cadence.

The existing 100-round catalog is migrated first. After the mechanics are
stable, numbers and timing may be retuned into an original Omoba standard
round curve.

### Selection Dimensions

Separate three concepts:

1. map identity and topology;
2. map difficulty category;
3. gameplay rule profile.

Maps are no longer locked to a gameplay difficulty. Initial rule profiles end
at rounds 40, 60, 80, and 100, with starting round, lives, prices, enemy speed,
sellback, and knowledge availability stored in data. The intended standard
endpoints are 40/60/80/100 rather than the current 40/65/85/100.

## Towers and Upgrades

Deepen the existing seven towers before adding more towers. Each tower gains
three five-tier paths. A primary path may reach tier five, one secondary path
may reach tier two, and the third path locks. Legal completed builds therefore
include 5-2-0 and 5-0-2 forms.

The enforced 25/50/100/250-percent upgrade-price formula is removed. Every
upgrade has an authored cost because detection, immunity bypass, control,
support, and raw damage have different strategic value.

Each path must state its target problem, incompatible enemies, synergy, and
reason to choose it over another damage path. Existing identities remain:

- Sugar Ball Gunner: pierce, multishot, critical damage;
- Hedgehog Shooter: attack speed, burn, radial volleys;
- Macaron Cannon: area damage, blimp specialization, clusters;
- Ice Crystal Teddy: freeze, aura control, vulnerability;
- Churro Artillery: long-range explosive damage, control, barrage;
- Cake Splash Tower: damage over time, support, slow and vulnerability;
- Banana Boomerang: ricochet, rapid volleys, special projectiles.

Tier three establishes a path identity, tier four completes its primary
mechanic, and tier five is an expensive endgame unit. Existing active abilities
remain but must be revalidated against layered enemies and damage profiles.

## TD Heroes

Do not immediately restore freely moving MOBA heroes to standard TD. First add
one TD-specific hero implementation:

- at most one hero per player;
- placed as a special defensive unit and not freely moved;
- automatically gains experience with rounds and reaches twenty levels;
- has at least two strategically important active abilities;
- uses `PlayerEconomy` for TD and never becomes the authoritative TD wallet.

MOBA mode keeps its existing mobile heroes. A second TD hero is deferred until
the first is complete and balanced.

## Maps

Keep all nine maps, but implement mechanics deeply on three representatives
before expanding the rest:

- Green Crossroads: baseline single-route map;
- Twin Gate Outpost: multiple entrances and a merge;
- Mine Corridor: blocked regions and restricted placement.

Later map-rule capabilities include land/water placement classes, sight
blockers, removable obstacles, reversed routes, speed zones, shortcuts, and
multiple exits. Player-facing map descriptions may only advertise mechanics
that runtime actually implements.

## Interaction and Feedback

Without requiring new art, the frontend must expose:

- the next round's important enemy types and properties;
- Camo, Regrow, Fortified, immunity, and MOAB-class state as text or existing
  UI primitives;
- whether the selected tower can damage the inspected threat;
- exact before/after upgrade effects and path-lock state;
- layer-based pop count;
- explicit unaffordable, invalid-placement, immunity, and detection feedback;
- end-of-run round, pops, leaks, spending, and leading-tower statistics.

Production gameplay exposes 1x and 3x speed. Existing broader debug speed
controls remain development-only. Auto-start remains, subject to rule-profile
configuration.

Sound work in this program means event hooks and throttling for layer pops,
immunity, Camo failure, MOAB entry, leaks, upgrades, sales, ability readiness,
round transitions, and game results. High-quality sound assets can follow
separately.

## Tutorial, Save, and Profile

The tutorial teaches placement, round start, range and upgrading, Camo,
damage immunity, selling, and one active ability through a fixed small map and
fixed encounters. Tutorial runs do not affect formal records.

Initial game saving is allowed only while idle between rounds. A save contains
the map and rule profile, round, deterministic seed, lives, economy, tower
ownership and placement, upgrades, target priority, cooldowns, and content
schema version. Mid-round projectile and event-queue saving is deferred.

Profile storage gains a stable user-data location, schema version, migration,
map/mode completion records, and medals. Knowledge data and generated
documentation must be validated from one source. The hardest 100-round profile
disables knowledge. Failed runs award knowledge points only after a configured
minimum amount of progress.

## Automated Round 1-100 Reference Player

### Purpose

Provide a headless integration test that completes rounds 1 through 100 with
no human input. This is a balance canary, a deterministic simulation test, and
an end-to-end validation of enemy layers, economy, towers, upgrades, active
abilities, and round progression. Merely loading all round records or using a
test-only instant-kill defender does not satisfy the requirement.

### Test Conditions

- `TD_GREEN_CROSSROADS`;
- fixed deterministic seed;
- rounds 1 through 100;
- heroes and knowledge disabled;
- real generated Lua content and `base_content` tower scripts;
- no graphical frontend or public network;
- the same authoritative `PlayerInput` path used by lockstep;
- no direct cash, health, spawn, despawn, damage, or round manipulation.

The deterministic rule-based reference player may submit only ordinary tower
placement, upgrade, target-priority, ability, sell, and start-round inputs. Its
policy reacts to round number, cash, current towers and upgrades, and upcoming
properties. It is an authored reference defense, not an optimizer. A balance
change that breaks it is a visible decision requiring either a balance fix or
an explicit reference-strategy update.

### Hardware-Efficient Fast Simulation

The complete round 1-100 test uses a separate coarse fixed-step profile so it
actually reduces work rather than asking ordinary hardware to execute
thousands of complete ECS ticks per second. Its fixed `dt` is 66.666
milliseconds, or 15 simulation ticks per game second. This invokes the full
system pipeline one eighth as often as the production 120 Hz profile.

The headless loop is uncapped and contains no deliberate wall-clock sleep. If
a machine sustains 240 coarse ticks per wall-clock second, it advances sixteen
game seconds per wall-clock second (`240 / 15 = 16x`). Sustaining 300 coarse
ticks per wall-clock second produces 20x. Neither rate is a pass condition:
slower hardware still runs the same test to completion, subject only to a
generous hang watchdog.

Coarse steps require elapsed-time-correct systems. Spawn queues drain every
event due within the elapsed interval in stable order. Attack, pulse, damage-
over-time, regeneration, cooldown, and buff accumulators consume every due
occurrence with bounded loops and retain fractional remainder. Creep and
projectile movement use deterministic swept segments so a 66.666-millisecond
step cannot tunnel through a checkpoint or collision target. These rules are
part of runtime correctness, not test-only instant resolution.

The coarse profile is deterministic with itself, but it is not required to
have the same tick number, target sequence, or state hash as production 120
Hz. Exact cross-rate hash equality is removed because a larger fixed step can
legitimately change the ordering of simultaneous decisions. Instead, short
production-rate milestone tests cover rounds 24, 28, 40, 60, 80, 90, and 100
and the Camo, immunity, Regrow, Fortified, leak, and economy mechanics. Cross-
rate checks compare invariant totals and legal outcomes, not complete state
hashes.

### Components

- `AutoplayController`: emits only normal player inputs.
- `RoundDriver`: advances uncapped 66.666-millisecond coarse ticks, detects
  idle/clear/victory states, and enforces simulation-progress and hang
  watchdogs without imposing a minimum wall-clock speed.
- `InvariantProbe`: validates economy, lives, entities, outcomes, and script
  health throughout the run.
- `AutoplayRunReport`: stores per-round summaries and failure diagnostics.

### Assertions

The test passes only when:

- every round from 1 through 100 starts and clears exactly once;
- the round-100 final enemy actually spawns and is defeated by normal combat;
- victory occurs with positive lives and non-negative cash;
- no generated input is rejected for affordability, ownership, or upgrade
  legality;
- 5-2-0 cross-path rules hold;
- Camo, Regrow, Fortified, Lead-like, Black-like, White-like, and Purple-like
  mechanics are exercised;
- all major blimp classes appear and are defeated;
- the economy ledger reconciles exactly;
- every spawned enemy ends popped or leaked;
- cleared rounds leave no enemy or projectile residue;
- entity and per-round tick watchdogs remain below configured bounds;
- no script panic, invalid numeric state, overflow, or unhandled outcome occurs.

The complete coarse-step test runs twice and compares final state hash,
per-round end tick, cash, lives, tower builds, enemy accounting, and economy-
ledger digest. Production-rate milestone tests compare invariant cash and
enemy totals, legal transitions, and final outcomes against the coarse run;
they do not require equal ticks or complete hashes.

On failure, the test writes an untracked report below `target/td-autoplay/`
containing round, tick, cash, lives, towers and upgrades, remaining enemies,
recent outcomes, rejected inputs, economy ledger, state hash, entity peak, and
tick-time summary.

The complete release-mode test is explicit/ignored and is run for TD-core
acceptance and scheduled CI. Short ordinary tests cover early rounds and the
important 24, 28, 40, 60, 80, 90, and 100 milestones. No new root batch file
is added.

## Determinism and Performance

All new authoritative values use the deterministic fixed-point runtime. At a
given tick profile, the same input stream must have the same outcome on backend
and local replica. Layer resolution must avoid transient entity explosions. At
equal live entity counts, production TD tick p95 may not regress more than
fifteen percent from the Phase 0 baseline. Dense late rounds must have bounded
entities and bounded simulation ticks.

The coarse 15 Hz profile is a hardware-efficient whole-run validation profile,
not the production gameplay rate. Production correctness remains covered by
focused 120 Hz tests.

## Delivery Phases

### Phase 0: Baseline and Test Harness

Capture current cash, lives, pops, round timing, entity peaks, and deterministic
hashes. Add the autoplay controller, fixed-tick round driver, invariant probe,
report format, coarse-run repeatability test, swept-movement checks, elapsed-
time accumulator checks, and focused 120 Hz milestone tests.

### Phase 1: Enemy and Economy Core

Implement layer graphs, properties, damage profiles, compatibility, overkill,
branching, per-layer pops and cash, leak damage, and the economy ledger. Remove
the generic generated-enemy bounty and duplicated round income. This is the
first OpenSpec change.

### Phase 2: Round Cadence and Difficulty Separation

Add spawn groups, migrate round data, separate map category from gameplay rule
profile, implement 40/60/80/100 endpoints, and author rule-specific prices,
speed, lives, start round, sellback, and knowledge availability.

### Phase 3: Tower Depth and Full Autoplay Pass

Give all seven towers damage profiles, add tier five to every path, remove the
fixed upgrade-cost formula, validate cross-path rules and active abilities,
author the reference defense, and make the non-cheating round 1-100 autoplay
test pass deterministically with the uncapped 15 Hz coarse profile.

### Phase 4: Representative Maps and Interaction

Complete the three representative map mechanics, next-round and immunity
information, results, tutorial, production speed controls, and sound-event
hooks.

### Phase 5: One TD Hero, Round-Boundary Save, and Profile

Add one complete TD hero, round-boundary save/resume, versioned profile data,
completion records, medals, and knowledge consistency and balance work.

There is no Phase 6 in this design.

## Acceptance Criteria

1. Enemy types are observable layer graphs rather than renamed large health
   bars.
2. Properties and immunities change viable tower and upgrade choices.
3. Round group timing creates distinct spaced, dense, overlapping, and
   multi-lane pressure.
4. Starting cash, every layer payment, round bonus, purchase, upgrade, sale,
   and ending cash reconcile exactly.
5. Leak damage depends on the remaining enemy layers.
6. All seven towers have three strategically distinct five-tier paths with
   legal primary/secondary cross-pathing.
7. Maps and rule profiles can be selected independently.
8. Lockstep replicas produce identical results and existing multiplayer
   ownership behavior remains intact.
9. The reference player completes rounds 1 through 100 without cheats.
10. Repeated coarse-profile autoplay runs are hash-identical, and focused 120
    Hz milestone tests agree on invariant totals and legal outcomes.
11. The work requires no new visual art to communicate the completed rules.
