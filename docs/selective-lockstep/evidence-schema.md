# Selective Lockstep Evidence Schema

## JSONL Record

`evidence/index.jsonl` 是 append-only UTF-8 JSON Lines；每行一個 object，不允許原地刪除歷史 record。必填欄位與型別：

Schema freeze 生效點為完成 OpenSpec task `1.2.4` 的 commit。Freeze 前既有 1.1 records 允許一次性 bootstrap migration，補上 `generation/replaces/record_id`；freeze 後任何變更都只能追加 lineage record。

| Field | Type | Contract |
|---|---|---|
| `task_id` | string | 永久 L3 task ID；同一 terminal generation 唯一 |
| `status` | enum string | `passed`, `not-applicable`, `superseded`, `stale` |
| `artifact_or_command` | string | canonical repo-relative `/` path、section anchor或 exact command |
| `expected` | string | 可判定預期結果 |
| `actual` | string | 實際結果與數值 |
| `exit_status_or_reviewer` | string | exit code或 reviewer identity |
| `hashes` | object<string,string> | artifact/raw evidence digest與 source commit |
| `related_gates` | array<string> | 至少一個 gate ID |
| `adjustment_id` | string or null | A/B/C change record ID |
| `timestamp` | RFC3339 string | 含 timezone |
| `subcheck` | string | shared artifact/command 下唯一 stable subcheck |
| `generation` | positive integer | 同 task replacement generation，初始 1 |
| `replaces` | string or null | 被替代 evidence record ID |
| `record_id` | string | `<task_id>@<generation>` |

Canonical JSON 供 record hash 使用：UTF-8、LF、object key lexicographic order、array order保留、無 insignificant whitespace。Evidence record 不得把 failed/blocked/未執行 leaf 標成 terminal pass。

## Artifact Hash

算法固定 SHA-256。Canonical path 是從 repository root 解析後的 `/` separated relative path，禁止 `..`、drive letter與大小寫折疊。File digest 輸入為原始 bytes；section digest 輸入為 UTF-8 heading 起點到下一個同級 heading 前，行尾 canonicalize 為 LF、移除結尾空白行。Directory manifest 依 canonical path byte order排序並 hash `path_len(u32 BE) || path_utf8 || file_len(u64 BE) || file_sha256`。

## Requirement Mapping

Traceability record schema：`requirement_id, scenario_id, task_ids[], gate_ids[], evidence_record_ids[]`。每個 scenario 至少一個 permanent task、blocking gate與 terminal evidence；每個 evidence record反向指回 gate。Mapping 只能追加 replacement generation，不得悄悄刪除 requirement。

## A-level Refinement

A-level 只允許拆分/排序 task、移動檔案、調整 command、fixture 或 evidence mechanics；不得改 scope、requirement、scenario、gate threshold、public/wire contract、authority、security invariant或 required evidence set。Record 必須含 `adjustment_id`, `reason`, `affected_task_ids`, `before`, `after`, `reviewer`。受影響 evidence 若語義仍相同可保留；artifact/hash 改變者必須 append stale + replacement。

## B-level Correction

B-level 是不改產品承諾但需要修改 design/spec/task 的 correction。流程固定：暫停 affected branch → 建立 correction record → reopen design與受影響 delta spec → 新增/修訂 permanent task → 將 dependent evidence append `stale` record → 實作 → 重跑 affected verification → append replacement。未受影響 branch 可繼續。Correction record 必填 `adjustment_id, reason, affected_requirements, affected_tasks, stale_record_ids, replacement_plan, reviewer`。

## C-level Change

C-level 涉及 scope、產品行為、security boundary、authority policy、wire public contract、gate threshold降低或 required evidence刪減。必須停止 affected work並取得使用者明確核准。Record 必填 `adjustment_id, requested_change, rationale, affected_scope, risk, alternatives, user_approval, approved_at`；`user_approval` 在核准前只能是 `null`，不得推定。

## Stale／Replacement Lineage

歷史 record 不修改。要失效既有 `record_id`，append 同 task下一 generation、`status=stale`、`replaces=<old record_id>` 並列 `stale_reason`；通過重跑後再 append下一 generation `status=passed`、`replaces=<stale record_id>`。Gate 只接受 lineage 最末端的 terminal `passed/not-applicable/superseded`；`stale` 沒有 passed replacement 時 gate blocking。
