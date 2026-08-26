# Server-Authoritative Selective Lockstep Design

## Summary

`omb` will replace its global-world lockstep stream with a server-authoritative selective lockstep architecture. The server remains the only process that owns the complete simulation. Each team receives and simulates only the deterministic subset of state that the team is authorized to know. Visibility is shared by the whole team, supports deterministic automatic vision plus explicit gameplay overrides, and never depends on a client viewport.

The client-side simulation is subordinate to the server. When client and server state conflict, the server result always wins through an authoritative component repair, entity replacement, or filtered team-view rebase.

Every active team also has a non-blocking observer replica inside the server process. A validation worker consumes the same encoded team stream as real clients and runs the same selective replica runtime. It detects divergence without delaying outbound traffic.

## Goals

- Prevent hidden units, state, inputs, RNG state, identities, and events from entering an unauthorized player's packets or memory.
- Preserve deterministic local stepping for the disclosed team world.
- Share visibility across all players on the same team.
- Support automatic range/obstacle/detection vision and script-driven overrides.
- Reveal entities at a scheduled tick without rollback.
- Support per-entity `Forget`, `LastKnown`, and custom remembered presentation policies.
- Make the server authoritative for inputs, transitions, outcomes, repair, and rebase.
- Validate each team stream asynchronously with a server-local observer replica.
- Preserve the existing steady-state lockstep bandwidth target of less than 5 KB/s per player.
- Pass the 10,000-entity stress scenario with the authoritative world and all active team observer replicas enabled.

## Non-goals

- Cryptographically protecting information that gameplay intentionally discloses to a team.
- Continuing a secure fog match by downgrading to the old global snapshot or global TickBatch protocol.
- Keeping the existing camera viewport AOI as the authority for gameplay vision.
- Sending the full world to a client and hiding entities only in the renderer.
- Giving clients the global master RNG seed or a PRNG state that remains usable after an entity becomes hidden.
- Making remembered ghosts participate in simulation, targeting, collision, or state hash.
- Maintaining permanent server-side replicas per player; replicas are per team.

## Current Architecture and Gaps

The current lockstep path broadcasts one `TickBatch` and one global `StateHash` to every session whose `lockstep_joined` flag is set. `SnapshotResp` contains a global `WorldSnapshot`, and `GameStart` exposes the global `master_seed`. This means renderer-only hiding cannot provide information security.

The legacy transport has viewport/AOI filtering and unused `VisSet` state. That code is camera-oriented, uses player names and raw ECS identifiers, and is bypassed by lockstep frames. It is not a suitable authority boundary.

The existing `vision_ecs` implementation is also not suitable as-is. It uses wall-clock timestamps and floating-point visibility geometry, caches by player name, and is not wired into the deterministic runtime dispatcher. Its geometry algorithms can be reused only after deterministic fixed-point conversion and explicit tick semantics.

The current snapshot and hash cover only selected global ECS fields. They are insufficient as a complete replica bootstrap and are unsafe for a partial-information client.

## Terminology

- **Authoritative World**: the only complete gameplay world, owned by the server.
- **Team View**: the state and events that one team is currently authorized to know.
- **Disclosed Entity**: an entity present in a team's deterministic replica.
- **Remembered Record**: sanitized last-known render data stored outside the replica simulation.
- **Canonical Entity**: the server-only ECS entity identity.
- **Replica Entity ID**: a team-scoped opaque identity used on the wire and in a team replica.
- **Observable Fact**: a deterministic gameplay result with enough metadata for later team projection.
- **External Effect**: a sanitized authoritative result caused by a dependency that cannot safely be disclosed.
- **Team Observer Replica**: a server-local replica that consumes only one team's encoded stream.
- **D**: visibility commitment delay.
- **L**: client replica buffering lead behind the authoritative server.

## Authority and Security Invariants

1. The server is the final authority for all accepted inputs, visibility, RNG results, spawns, deaths, damage, buffs, repairs, and snapshots.
2. A player session is bound to exactly one team view for the duration of a secure match.
3. A team frame never includes canonical ECS identifiers, another team's visibility mask, the global RNG seed, or server-only component data.
4. Two authoritative worlds that differ only in hidden state must emit byte-identical frames for a team until that difference causes an intentionally public effect.
5. Remembered records are renderer-only and excluded from input validation, simulation, collision, targeting, and hash calculation.
6. A secure match never falls back to a global-world protocol after it starts.
7. Observer validation is non-blocking. Its absence or lag is reported as a coverage gap, never as successful verification.
8. Repairs and rebases pass through the same team projection and redaction boundary as ordinary frames.

## High-Level Architecture

The server runs these logical components:

1. `AuthoritativeSimulation`: owns the complete Specs world and runs fixed ticks.
2. `VisibilityResolver`: computes deterministic team visibility from committed state.
3. `VisibilityTransitionScheduler`: converts raw visibility into scheduled reveal/hide/forget transitions.
4. `TeamViewProjector`: converts observable facts and authoritative state into one team's safe representation.
5. `TeamFrameBuilder`: builds and encodes ordered team-specific frames.
6. `TeamStreamRouter`: immediately enqueues each encoded frame to sessions on that team.
7. `ObserverValidationWorker`: taps the same encoded frames, decodes them, advances team observer replicas, and reports divergence through a control channel.
8. `AuthorityRepairCoordinator`: produces subsequent repair or filtered rebase frames when a replica reports divergence.

Shared `omoba-core` code owns the wire types, canonical team-view serialization, transition application, and `SelectiveReplicaRuntime`. Both omfx and the server validation worker call the same runtime. These types do not belong in `scripts/script-abi`.

## Deterministic Specs Tick Pipeline

Steps previously described as "step world", "collect public effects", and "calculate visibility" are integrated into one Rust Specs tick pipeline rather than three serial full-world scans.

### Tick start

At the beginning of tick `T`, `State[T]` and the committed visibility view `V[T]` are stable. Inputs are validated against ownership, the session's team, the referenced view epoch, and visibility history at the input tick.

### Wave A: parallel gameplay evaluation

Gameplay systems run in the existing Specs dispatcher according to storage conflicts and explicit dependencies. Each gameplay system emits two deterministic side outputs while calculating its authoritative work:

- `Outcome`: authoritative mutation to apply at the commit barrier.
- `ObservableFact`: a projection-ready fact describing what occurred without deciding which teams may receive it.

Movement, combat, skills, scripts, spawn, and death therefore produce public-projection facts during their normal computation. The server does not perform a later full-world scan to infer effects.

Parallel writers must not depend on shared `Vec` arrival order. Each output carries a stable ordering key:

```text
(tick, phase, canonical_source_order, local_ordinal, fact_kind)
```

Thread-local or sharded buffers are merged, sorted, deduplicated, and validated at the barrier. Scripts continue to use the Outcome contract rather than directly mutating unrelated ECS state.

### Deterministic commit barrier

The server applies sorted outcomes, runs `World::maintain`, and establishes committed `State[T+1]`. This barrier is required because visibility for `T+1` must observe final post-step positions, stealth, death, ownership, and vision-source changes.

### Wave B: parallel team projection

After `State[T+1]` exists, read-only jobs run in parallel by team:

- resolve raw `V[T+1]`;
- update visibility candidates and scheduled transitions;
- project tick facts for the team;
- capture baselines for transitions becoming effective;
- construct, encode, and enqueue the team frame.

Wave A and Wave B cannot be made a single barrier-free operation without either calculating stale `V[T]` or duplicating next-state logic. The two-wave pipeline preserves parallel execution while keeping post-step visibility correct.

## Visibility Model

### ECS components and resources

- `ReplicationScope`: `ServerOnly`, `Public`, `OwnerTeam`, or `TeamVision`.
- `VisionSource`: owning team, range, height/detection tags, and enabled state.
- `StealthProfile`: stealth layers and detector requirements.
- `VisibilityOverride`: force-show or force-hide grants with priority and expiration tick.
- `RememberPolicy`: `Forget`, `LastKnown`, or a registered custom renderer policy.
- `TeamVisibilityIndex`: resolved visible canonical entities per team, visibility epoch, and transition state.
- `TeamReplicaIdMap`: canonical-to-replica identity mapping private to one team.

Gameplay and scripts change these through explicit outcomes such as `GrantVisibility`, `RevokeVisibility`, `SetReplicationScope`, and `SetRememberPolicy`.

### Team sharing

Vision sources owned by any player or unit on a team contribute to one shared team view. The final authorization boundary remains per session: sessions inherit only the stream of their bound team.

### Automatic and override resolution

Resolution order is deterministic:

1. `ServerOnly` denies disclosure.
2. an unexpired force-hide override denies disclosure unless a higher-priority rule explicitly supersedes it;
3. `Public` or an applicable force-show grant discloses the entity;
4. `OwnerTeam` discloses to its owner team;
5. `TeamVision` requires automatic geometry and detection rules to succeed;
6. ties use stable rule identifiers, never insertion order.

### Scheduled transitions

The default visibility commitment delay is three ticks. A raw visibility change creates a candidate. If the required condition still holds when the candidate matures, the transition is committed. At the effective tick the server captures a fresh baseline from current authoritative state; it never applies a stale baseline captured when the candidate was created.

The client replica normally buffers twelve ticks at 120 Hz, equivalent to 100 ms. Both values are authoritative match configuration announced at handshake. Supported bounds are:

- `visibility_commit_delay_ticks`: 2 through 4;
- `replica_buffer_ticks`: at least the visibility delay and from 3 through 24.

Changing these values requires protocol-compatible match negotiation and performance/latency evidence.

### Visibility state machine

```text
Hidden -> RevealCandidate -> Disclosed
Disclosed -> HideCandidate -> Remembered | Hidden
Remembered -> RevealCandidate -> Disclosed
Remembered -> Forget -> Hidden
```

Candidate cancellation is explicit and deterministic. Re-reveal can reuse the existing team-scoped replica ID so a renderer may associate it with a remembered record. IDs are never shared across teams.

## Team-Scoped Identity

Raw `specs::Entity::id()` and generation values are server-only. Each team receives opaque `ReplicaEntityId` values allocated from that team's monotonic, non-reused match-local namespace. A mapping entry contains a disclosure epoch so stale transitions and inputs cannot affect a later incarnation.

Replica IDs remain stable across a permitted remembered interval. They are retired permanently after an authoritative forget or after the canonical entity is destroyed and that destruction becomes known to the team.

## Wire Protocol V2

A secure match negotiates protocol V2 before joining. All player sessions in that match use V2.

### Team game start

`TeamGameStart` contains:

- protocol and snapshot schema versions;
- player ID and team ID;
- authoritative server tick;
- replica start tick;
- configured tick rate, visibility delay, and replica buffer;
- a verified filtered team snapshot;
- public match metadata and team-private deterministic resources.

It does not contain the global master seed.

### Team tick frame

`TeamTickFrame` contains:

- `server_tick` and `replica_tick`;
- monotonic `team_sequence`;
- `view_epoch`;
- `PreStep` transitions;
- `Step` accepted inputs, public server events, random tape entries, and external effects;
- `PostStep` authoritative repairs and optional hash checkpoint;
- content/schema compatibility metadata.

Frame ordering is canonical by phase, event kind, replica entity ID, and stable sub-index.

### Transitions

- `RevealEntity`: replica ID, disclosure epoch, effective tick, kind, complete safe baseline, and disclosed dependency records.
- `HideEntity`: replica ID, disclosure epoch, effective tick, and sanitized remembered presentation if allowed.
- `ForgetEntity`: replica ID and effective tick.
- `ReplaceEntity`: complete authority replacement for one disclosed entity.

### Repair and rebase

Authority correction has three levels:

1. `ComponentRepair` overwrites explicit disclosed fields at a server-specified barrier.
2. `EntityReplace` atomically replaces one disclosed entity.
3. `TeamViewRebase` replaces the entire deterministic team replica with a filtered snapshot and resumes from a specified frame sequence.

All corrections carry newer authority revisions. A client never wins a revision conflict against the server.

## Randomness

The global master seed and reusable global PRNG state remain server-only. Disclosed local simulation obtains either:

- already-decided authoritative random outcomes; or
- a short, bounded random tape scoped to a disclosure epoch and a limited tick window.

A tape cannot derive values outside its declared window or across a later hidden period. Hidden-dependent random behavior is projected as an authoritative external effect.

## Cross-Visibility Dependencies

The simulation closure rule is:

> A client locally simulates an action only when every dependency required for deterministic evaluation can be safely disclosed. Otherwise the server projects the result as a sanitized external effect.

Examples:

- A hidden attacker damages a visible hero: disclose target, amount, damage class, tick, and only the attribution allowed by gameplay rules. Do not disclose attacker ID or position.
- A hidden projectile enters vision: reveal its current baseline. Use an anonymous public source surrogate when the owner remains hidden.
- A visible projectile's target becomes hidden: remove the private target reference and switch to a disclosed trajectory, hide the projectile, or send a later authoritative impact outcome according to projectile policy.
- An AOE crosses the boundary: disclose effects only for disclosed targets. Do not disclose the hidden target count.
- A visible buff from a hidden caster: disclose the public buff effect without the caster identity.
- A remembered enemy dies in fog: keep the last-known record until death becomes team-known or another policy forgets it.

Every gameplay system and script-visible action must declare a projection policy covering visible-visible, hidden-visible, visible-hidden, and hidden-hidden cases. Missing policy is a blocking integration error.

## Client and Observer Replica Runtime

For replica tick `T`, `SelectiveReplicaRuntime` performs:

1. require the expected sequence and tick frame;
2. atomically apply `PreStep` transitions;
3. inject accepted inputs, events, random tape, and external effects;
4. execute one fixed deterministic tick over disclosed entities and resources;
5. apply `PostStep` authority revisions;
6. calculate the canonical team-view hash when requested;
7. emit a render snapshot.

Remembered records live in a separate render cache. A missing or late frame stops the replica at its barrier. The runtime does not guess through an authoritative gap.

## Non-blocking Server Observer Validation

Each encoded team frame is immediately enqueued to the team's network sessions. Validation is not on the send critical path.

The same encoded `Arc<[u8]>` is tapped to a bounded channel owned by a separate validation worker thread in the server process. The worker maintains an isolated observer replica for each active team. Each observer:

- bootstraps through the same filtered snapshot path as a remote team observer;
- decodes the actual wire bytes;
- runs the shared `SelectiveReplicaRuntime`;
- cannot access the authoritative Specs world, canonical IDs, or another team's state;
- compares its checkpoint hash with the server's canonical projected hash for that team and tick.

On mismatch, the worker records the first divergent tick, team, frame sequence, hashes, transition epoch, and safe component path information. It reports through a control channel. The authoritative coordinator emits a repair or filtered rebase in a later frame. Real clients and the local observer consume the same correction.

If the validation channel fills, outbound traffic continues. The server records a verification coverage gap, discards the stale observer instance, obtains a new filtered snapshot through the same bootstrap path, and resumes from the newest retained frame. A coverage gap is an alertable failure and cannot be counted as verification success.

## Input Validation and Anti-Probing

Client commands reference team-scoped replica IDs and the view epoch observed when the command was issued. The server validates:

- player/session/team binding;
- ownership and command permission;
- replica ID mapping and disclosure epoch;
- visibility at the command tick when visibility is required;
- input timing and deduplication ID.

Invalid target commands return generalized rejection classes with uniform processing timing. Responses must not reveal whether an undisclosed canonical entity exists. Rate limits apply to repeated invalid references.

## State Hashes

The global state hash is replaced for player sessions by a canonical team-view hash. It covers every deterministic disclosed component and team-visible deterministic resource, ordered by replica ID and schema-defined field order. It excludes remembered render records, server-only data, diagnostics, queues that do not affect simulation, and other teams' state.

The authoritative server computes the expected hash from its team projection. Client and observer hashes are evidence, not authority. Mismatch triggers correction and diagnostics before any disconnect policy.

## Network Recovery and Rejoin

- Duplicate frames are idempotently ignored by team sequence and authority revision.
- A sequence gap requests replay from a bounded per-team encoded-frame ring.
- If the requested sequence expired, the server sends a verified filtered rebase and subsequent catch-up frames.
- Rejoining players receive only their team's snapshot and stream.
- A late transition stops the client at the relevant tick barrier.
- An interrupted rebase is discarded unless its snapshot ID, chunk hashes, and final manifest all verify.

## Side-Channel Controls

Frames retain a fixed tick cadence even when they contain no inputs or transitions. Sensitive payloads use configured size buckets and padding so common hidden activity cannot be inferred directly from exact payload length. Mass reveal and rebase traffic is chunked and rate-limited independently from steady-state frames.

Logs, replay files, crash bundles, and performance traces use the same team redaction rules. Full authoritative diagnostics require an explicit server-admin capability and must never travel over a player session.

## Observability

Metrics are labeled by opaque match/team identifiers and include:

- raw and committed visible entity counts;
- reveal, hide, forget, and canceled-candidate counts;
- frame bytes before and after padding;
- frame build, encode, enqueue, and replica-step duration;
- outbound queue depth and validation queue depth;
- observer audit lag and verification coverage gaps;
- hash mismatches, component repairs, entity replacements, and rebases;
- client barrier stalls, gap replays, and rejoin duration;
- projection-policy failures and redaction violations.

Diagnostics must avoid canonical entity identity when exported outside the authoritative server.

## Performance Gates

The blocking stress target is the existing 10,000-entity scenario at the configured production tick rate with the authoritative world, two team projection jobs, two team observer replicas, transport encoding, and visibility churn enabled.

Required gates:

- p99 authoritative tick plus required commit work stays within 80% of the tick period;
- p99 projection and enqueue finish before their configured client buffer deadline;
- outbound delivery never waits for observer validation;
- a 30-minute stress soak has zero unintended rebases and zero authoritative tick deadline misses;
- the observer has zero unreported coverage gaps and catches up after every injected gap;
- steady-state network usage remains below 5 KB/s per player, measured separately from bounded reveal/rebase bursts;
- memory is stable during repeated reveal/hide churn and replica rebootstrap;
- no blocking security or hidden-data finding remains.

These gates cannot be weakened or bypassed by disabling observer validation. A threshold change is a material design change requiring explicit approval.

## Test Strategy

The complete test and inspection suite runs once, after the server, shared runtime, observer validator, protocol, and omfx client are integrated. Implementation phases do not each repeat the complete suite. During implementation, developers may use only the minimum compile check or focused smoke needed to keep the branch usable; those checks are not acceptance evidence and do not replace final verification.

### Unit and property tests

- visibility precedence, team sharing, stealth/detection, override expiry, and candidate cancellation;
- transition state machine and disclosure epochs;
- team-scoped ID allocation and stale-ID rejection;
- canonical output ordering under parallel system scheduling;
- projection policy completeness;
- remembered records excluded from simulation and hashes;
- repair revision ordering and idempotence;
- random tape window and epoch isolation;
- non-interference: hidden-only changes produce byte-identical team frames until an allowed public effect.

### Differential and integration tests

- authoritative team projection, server observer replica, and omfx replica hashes match;
- Windows and Linux determinism pins match;
- reveal, hide, canceled transition, re-reveal, and forget;
- hidden damage, projectile boundary crossings, AOE, buffs, fog death, and team-shared vision;
- accepted/rejected inputs at visibility boundaries;
- component repair, entity replacement, and full rebase convergence;
- observer validation remains non-blocking under deliberate validator slowdown.

### Fault and adversarial tests

- late, duplicate, reordered, missing, corrupt, and oversized frames;
- reconnect, ring expiry, interrupted rebase, and validation channel overflow;
- hidden-target probing, replica-ID enumeration, malformed disclosure epochs, and replay attacks;
- packet capture scans for canonical IDs, global seeds, hidden component values, and unpadded sensitive size patterns;
- fuzzed protocol transitions and snapshot decoding.

### Stress tests

- 10,000 entities with two or more teams;
- rapid vision-source movement and repeated mass reveal/hide;
- mass projectile and AOE boundary crossings;
- 30-minute soak with CPU, memory, bandwidth, queue, and audit-lag reports.

## Delivery Program

This program is too broad for one implementation cycle. Work is divided into implementation changes that preserve the approved contracts, followed by one consolidated final verification phase. Complete testing, security inspection, stress testing, and release checks are intentionally not repeated after each implementation phase.

### Phase 0: contracts, threat model, and baselines

- inventory every deterministic component, resource, input, event, script outcome, snapshot field, and hash field;
- classify each as `Public`, `TeamPrivate`, `VisibilityBound`, or `ServerOnly`;
- define projection policy coverage for existing gameplay systems;
- pin protocol V2, snapshot, transition, repair, and canonical hash schemas;
- capture current 10,000-entity CPU, memory, and bandwidth baselines;
- add non-interference and redaction test harnesses.

Phase deliverable: the inventory, classifications, schemas, harnesses, and baseline data are available to all later phases.

### Phase 1: shared selective replica foundation

- extract `SelectiveReplicaRuntime` and canonical team hash into `omoba-core`;
- add team-scoped identity, transition application, random tape, repair, and rebase primitives;
- generate protocol V2 types from `proto/game.proto`;
- build filtered snapshot encode/decode and compatibility guards;
- provide synthetic server-observer and client fixtures that consume identical encoded frames for use by final verification.

Phase deliverable: the shared runtime and protocol V2 implementation compile and are available for server and omfx integration. Full determinism and protocol fault validation is deferred to final verification.

### Phase 2: deterministic ECS projection pipeline

- add Outcome and ObservableFact stable buffer contracts;
- migrate gameplay systems and scripts to emit complete projection facts;
- implement the Wave A deterministic reduce/commit barrier;
- implement deterministic fixed-point team visibility, overrides, and transition scheduling;
- implement Wave B parallel per-team projection and frame encoding;
- reject missing cross-boundary projection policies at startup or content validation.

Phase deliverable: authoritative gameplay emits complete projection facts and can produce deterministic team frames. Full non-interference and boundary validation is deferred to final verification.

### Phase 3: server team streams and observer validator

- bind sessions to team-specific V2 streams;
- immediately enqueue encoded team frames and retain a bounded replay ring;
- run the isolated validation worker and one observer replica per active team;
- add mismatch reporting and subsequent authority repair/rebase coordination;
- implement filtered join/rejoin and coverage-gap rebootstrap;
- add redacted metrics, traces, replay evidence, and packet audits.

Phase deliverable: server team streams, replay/rebase recovery, and asynchronous observer validation are integrated. Full remote parity and fault validation is deferred to final verification.

### Phase 4: omfx integration and cutover preparation

- replace the global local replica with the team `SelectiveReplicaRuntime`;
- add barrier buffering, remembered render cache, transition presentation, and repair/rebase handling;
- require match-level V2 capability negotiation;
- prepare shadow/dogfood configuration without enabling secure default;
- prepare cleanup patches for the global TickBatch, StateHash, master seed, WorldSnapshot, dead viewport/VisSet state, and superseded nondeterministic vision code, but do not perform the irreversible cutover before final verification.

Phase deliverable: one end-to-end V2 implementation is ready for consolidated verification. No production cutover claim is made at this point.

### Phase 5: consolidated final verification and cutover

Run the complete validation only after Phases 0 through 4 are integrated:

- run the entire unit and property suite;
- run authoritative projection versus server observer versus omfx differential tests;
- run Windows and Linux determinism checks;
- run all visibility-boundary integration scenarios;
- run network fault, reconnect, replay-ring, rebase, and validator-overflow scenarios;
- run adversarial protocol fuzzing, hidden-target probing, packet inspection, redaction review, and side-channel checks;
- run the 10,000-entity performance suite and 30-minute stress soak with every active team observer enabled;
- review all blocking performance, bandwidth, security, and verification-coverage gates together;
- fix failures, mark affected evidence stale, and rerun the affected final-verification groups until every blocking gate passes;
- run V2 shadow and internal dogfood acceptance;
- make V2 the secure default only after the consolidated evidence passes;
- remove player access to the global protocol path and land the prepared cleanup.

Phase exit gate: every blocking gate passes, packet and client-memory inspection finds no hidden information exposure, observer validation remains non-blocking, and no unresolved release blocker remains.

## Rollout and Rollback

Rollout stages are server shadow generation, internal dogfood, opt-in secure matches, then secure default. Protocol versions are selected at match creation; V1 and V2 clients do not coexist in a secure match.

Rollback is allowed only before a match starts by selecting an explicitly non-secure legacy mode. An active secure match that cannot repair or rebase must end safely and preserve diagnostics. It must not continue by sending global world state.

## Compatibility and Cleanup

During migration, old and new types may coexist behind match-level protocol selection. The V2 player path must never invoke legacy global snapshot/hash serialization. Admin/query tools require a distinct capability and transport boundary.

After cutover, remove global player lockstep fan-out, global player snapshots, player-visible master seed, raw ECS IDs on the player wire, dead `client_visibility`/`last_visibility_tick` state, and any renderer-only fog assumptions that could expose data.

## Approved Decisions

- Hidden information must be absent from unauthorized packets and client memory.
- Visibility rules combine deterministic automatic vision and explicit gameplay overrides.
- Visibility is shared by team.
- The client simulates the disclosed subworld rather than receiving only state deltas.
- Reveal uses scheduled commitment and no rollback.
- Remember behavior is policy-driven; the default is forget, with selected units allowed to keep last-known presentation.
- The server always wins conflicts.
- Every active team has a server-local observer replica for validation.
- Observer validation is asynchronous and never blocks outbound frames.
- Gameplay outcomes and observable facts are produced together in the Specs gameplay wave.
- Post-step team visibility and projection run in a second parallel wave after a deterministic commit barrier.
