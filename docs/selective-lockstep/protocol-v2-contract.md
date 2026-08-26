# Selective Lockstep Protocol V2 Contract

本文件固定 secure match 的 wire、timing、identity、authority、snapshot 與 recovery contract。所有整數使用 protobuf unsigned varint；所有 repeated field 在 encode 前依本文件 canonical key 排序。Secure match 只能使用 V2，且不得攜帶 global master seed、raw ECS ID 或其他 team hidden state。

## Versions

- `protocol_version = 2`。
- `snapshot_schema_version = 1`，只描述 filtered team snapshot schema。
- `frame_schema_version = 1`。
- 未知 major protocol 必須在取得任何 snapshot 前拒絕；未知 message/field 不得靜默改變 deterministic semantics。

## TeamGameStart

Canonical field order：

1. `protocol_version: u32`
2. `snapshot_schema_version: u32`
3. `content_schema_version: u32`
4. `player_id: u32`
5. `team_id: u32`
6. `server_tick: u64`
7. `replica_start_tick: u64`
8. `tick_rate_hz: u32`
9. `visibility_commit_delay_ticks: u32`
10. `replica_buffer_ticks: u32`
11. `view_epoch: u64`
12. `next_team_sequence: u64`
13. `snapshot_id: SnapshotId`
14. `snapshot_manifest_hash: bytes[32]`
15. `filtered_snapshot: bytes`
16. `public_metadata: repeated DeterministicMetadata`
17. `team_private_metadata: repeated DeterministicMetadata`

Metadata 依 `(namespace UTF-8 bytes, key UTF-8 bytes, schema_version)` 排序。`filtered_snapshot` 必須先通過 manifest verification。禁止 global seed、canonical entity ID、其他 team metadata。

## Timing Contract

- `visibility_commit_delay_ticks`：default `3`，合法範圍 `[2, 4]`。
- `replica_buffer_ticks`：default `12`，合法範圍 `[3, 24]`，且必須 `>= visibility_commit_delay_ticks`。
- Shared tick rate 固定 `120 Hz`。唯一換算 helper：`ticks_to_micros(t) = floor(t * 1_000_000 / 120)`；deadline 比較一律使用 integer tick，不用 wall-clock 反推 gameplay tick。
- Visibility candidate 在 delay 到期且條件仍成立時，才從當下 authoritative state 擷取 fresh baseline。

## Team-scoped Identity

`ReplicaEntityId` 是 non-zero `u64`，namespace 綁定 `(match_id, team_id)`；由 `1` 起 monotonic 配置，overflow 必須終止 match，不得 wrap/reuse。Mapping 保存 canonical entity、replica ID、`disclosure_epoch`、狀態與 retire reason。

Canonical entity destroyed 且 death 已 team-known，或 authoritative `ForgetEntity` 生效後，ID 永久 retired。Remembered presentation 存續期間 ID 可保持 stable。任何 retired ID 永不重新指派。

`disclosure_epoch` 在同一 mapping 每次 reveal/replace authority incarnation 遞增；`view_epoch` 在會改變整體 team view interpretation 的 rebase/authority reset 後遞增。低於目前 epoch 的 transition、input、repair、chunk 或 frame一律 stale reject；高於預期 epoch 時停在 barrier並要求 replay/rebase。

## TeamTickFrame Envelope

Canonical field order：

1. `protocol_version: u32`
2. `frame_schema_version: u32`
3. `content_schema_version: u32`
4. `team_id: u32`
5. `server_tick: u64`
6. `replica_tick: u64`
7. `team_sequence: u64`
8. `view_epoch: u64`
9. `authority_revision: u64`
10. `pre_step: PreStep`
11. `step: Step`
12. `post_step: PostStep`
13. `padding: bytes`

`team_sequence` 每 team 由 1 起 monotonic；duplicate 可 idempotent ignore，gap 必須停 barrier。Padding 只能在完成內容 encode 後依 configured bucket 加入，且不參與 deterministic hash。

## PreStep

Payload：`transitions: repeated Transition`。Canonical key：`(effective_tick, transition_kind_order, replica_entity_id, disclosure_epoch, stable_sub_index)`；kind order 固定 `Reveal=0, Replace=1, Hide=2, Forget=3`。

Transition 欄位：

- `RevealEntity`：`replica_entity_id, disclosure_epoch, effective_tick, entity_kind, safe_baseline, disclosed_dependencies`。
- `ReplaceEntity`：`replica_entity_id, disclosure_epoch, effective_tick, authority_revision, safe_baseline, disclosed_dependencies`。
- `HideEntity`：`replica_entity_id, disclosure_epoch, effective_tick, remember_policy, sanitized_remembered_presentation`。
- `ForgetEntity`：`replica_entity_id, disclosure_epoch, effective_tick, retire_reason`。

Transition 只在 `effective_tick == replica_tick` 套用。Baseline 欄位依 snapshot schema field number排序，dependency 依 `(kind, replica_entity_id)` 排序。

## Step

Payload canonical group order：

1. `accepted_inputs`
2. `public_events`
3. `random_tapes`
4. `external_effects`

每組依 `(event_kind_order, replica_entity_id_or_zero, stable_sub_index)` 排序。Accepted input 使用 team replica ID，絕不含 canonical ID；hidden dependency 無法安全 disclosure 時不得送 local-sim input，改送 sanitized external effect。

### Bounded Random Tape

欄位：`tape_id: u64, disclosure_epoch: u64, first_tick: u64, tick_count: u32, algorithm_id: u32, values: repeated u64, consumer_kind: u32, replica_entity_id: u64`。`tick_count` 必須在 `[1, replica_buffer_ticks]`，values 數量由 registered consumer schema 精確決定。Tape 只在 `[first_tick, first_tick + tick_count)` 與相同 disclosure epoch 有效；過期立即銷毀，不得推導 window 外值。Global seed/PRNG state 永不進 wire。

## PostStep

Payload canonical group order：`component_repairs, entity_replaces, optional_hash_checkpoint, optional_rebase_notice`。Repair key：`(authority_revision, repair_kind_order, replica_entity_id, component_schema_id, field_number)`。

### Authority Repair

- `ComponentRepair`：`replica_entity_id, disclosure_epoch, component_schema_id, field_mask, replacement_fields, authority_revision, effective_tick`。
- `EntityReplace`：`replica_entity_id, disclosure_epoch, safe_baseline, authority_revision, effective_tick`。
- `TeamViewRebaseNotice`：`snapshot_id, manifest_hash, resume_team_sequence, view_epoch, authority_revision`。

`authority_revision` 每 team 由 1 起 monotonic 配置且不得 reuse。Conflict 時較高 server revision 無條件覆寫 client/observer；相等 revision 必須 byte-identical，否則視為 protocol violation；較低 revision stale ignore。

## Snapshot、Chunk 與 Manifest

`SnapshotId` canonical bytes：`snapshot_schema_version(u32 BE) || match_instance_id(16 bytes) || team_id(u32 BE) || view_epoch(u64 BE) || authoritative_tick(u64 BE) || monotonic_snapshot_ordinal(u64 BE)`。同一 match/team 永不重用 ordinal。

`TeamViewRebaseChunk` 欄位順序：`protocol_version, snapshot_schema_version, snapshot_id, chunk_index, chunk_count, uncompressed_offset, uncompressed_len, compression_id, payload, chunk_hash`。`chunk_index` 為 `[0, chunk_count)`；同一 snapshot 的 version/count/compression 不得改變。

Chunk hash 使用 SHA-256，輸入為 domain tag `omoba-team-rebase-chunk-v1\0`、canonical SnapshotId bytes、`chunk_index(u32 BE)`、`chunk_count(u32 BE)`、`uncompressed_offset(u64 BE)`、`uncompressed_len(u32 BE)`、實際 transmitted payload bytes。

Manifest 欄位順序：`manifest_version, protocol_version, snapshot_schema_version, snapshot_id, team_id, view_epoch, authoritative_tick, resume_team_sequence, authority_revision, total_uncompressed_len, compression_id, chunk_count, ordered_chunk_hashes, filtered_snapshot_hash`。

Manifest hash 使用 SHA-256，輸入為 domain tag `omoba-team-rebase-manifest-v1\0` 加上依上述欄位順序的 canonical fixed-width bytes；`ordered_chunk_hashes` 前置 `u32 BE count`，每筆固定 32 bytes。所有 chunk、snapshot ID 與 final manifest hash 驗證成功前不得套用 rebase。

## Security／Recovery Invariants

- Server authoritative state、revision 與 repair永遠優先。
- Player/observer只能取得自己 team 的 filtered bootstrap、frame、replica mapping與 hash。
- Encoded frame 先直接 enqueue outbound；同一 `Arc<[u8]>` 才 non-blocking tap 到同 process另一 thread 的 team observer validator。
- Replay ring 以 encoded frame、team sequence保留 bounded `replica_buffer_ticks` window；過期則 filtered rebase。
- Fixed cadence、size bucket與 padding不得包含可推導 hidden activity的差異。
