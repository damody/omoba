# 最終可追溯性報告

日期：2026-08-27。結論：proposal、design、delta spec、永久 task ID、blocking gate 與 evidence record 的鏈路完整。

## 承諾 → 決策 → Requirement／Scenario → Task／Evidence

| Proposal 承諾 | Design 決策 | Requirement／Scenario 群組 | 永久 task ID | 最終 evidence／gate |
|---|---|---|---|---|
| Server-authoritative selective lockstep | D1、D6、D10 | selective-replica-authority 全部；server revision 永遠勝出 | 2.1–2.5、4.1、5.1、6.1、6.4 | `unit-property`、`fault-recovery`／G-FINAL-UNIT、G-FINAL-RECOVERY |
| Team-scoped 戰爭迷霧 | D2、D3、D5 | team-visibility-projection 全部 scenario | 1.3、2.2–2.4、3.1–3.4、6.3 | `boundary`／G-FINAL-BOUNDARY |
| 不向玩家揭露 hidden state | D3、D7 | secure-fog-information-boundary 全部 scenario | 1.3、2.2、3.4、4.5、6.2、6.5 | `differential`、`security`／G-FINAL-NONINTERFERENCE、G-FINAL-HIDDEN-DATA |
| V2 team stream 與 filtered bootstrap | D6 | selective-lockstep-protocol 全部 scenario | 1.2、2.1–2.5、4.1、5.1、6.1–6.4 | `unit-property`、`boundary`、`fault-recovery` |
| Client 與 observer 共用 replica | D8 | frontend-backend-decoupling、sim-snapshot-rendering 全部 scenario | 2.5、4.1–4.4、5.1–5.4、6.2、6.4 | `differential`、`fault-recovery` |
| 同 process、另一 thread 的 team observer | D8、D9 | team-observer-validation 全部 scenario | 4.2–4.4、6.4、6.6、6.8 | `fault-recovery`、`performance`、`release/cutover-summary.json` |
| Specs 2／3／4 同 wave 平行計算 | D4 | lockstep-event-flow 的 Outcome／ObservableFact 與 canonical merge scenario | 3.3、3.5、4.1、6.1–6.3 | `unit-property`、`differential`、`boundary` |
| 120Hz shared cadence | D6 | lockstep-cadence、render-sim-cadence 全部 scenario | 1.1、2.5、5.2、6.1、6.6 | `unit-property`、`performance` |
| Player input 不形成 hidden oracle | D7、D10 | player-input-routing 全部 scenario | 3.4、4.5、5.3、6.3、6.5 | `boundary`、`security` |
| 舊 global disclosure path cleanup | D1、D6 | lockstep-event-flow legacy 禁止與 filtered snapshot scenario | 3.5、4.5、5.4、6.5、6.8 | `security`、`release/cutover-summary.json`／G-SECURE-DEFAULT |
| 完整驗證集中最後 | D11、D12 | cadence final verification scenario 與 evidence lineage | 1.4、5.5、6.1–6.9 | `evidence/index.jsonl`、所有 `evidence/final/*` |

每個 requirement 下的 scenario 都由同列 task 群組的獨立 L3 leaf 與唯一 evidence record 覆蓋；boundary summary 額外列出 30 個跨可見性 scenario，security summary 列出 28 個安全 scenario 與 743 個 fuzz case。

## 完整性稽核

- 31 個 L2 均具目的、輸入、產出、依賴、Owner／Wave、Gate／Evidence 與完成門檻。
- 631 個 L3 均為單一、可執行、可驗證的小步驟；Phase 1–6 已逐 phase 通過 Luna 原子化審查，沒有需再拆分的高難度 leaf。
- Artifact 掃描只命中規格本身的「不存在 TODO stub」scenario 與最終門檻文字，沒有實際未完成標記。
- Evidence terminal status 只有 `passed`、`not-applicable`；沒有 duplicate record ID、failed、blocked 或 stale terminal record。
- Conditional leaf 僅 6.7 的 B/C 分支使用有證據的 `not-applicable`；A-level 修正 A-20260827-001 已有完整 replacement lineage。
- Forbidden reference 掃描命中的 `TickBatch`、`StateHash`、`WorldSnapshot`、`master_seed` 與 raw Specs ID 只存在 authoritative runtime、測試或明確 legacy non-secure path；secure V2 transport guard 與 schema 測試證明玩家路徑不可達。

## 最終命令

- `cargo test --manifest-path omoba-core/Cargo.toml`：230 unit + 3 integration + 1 doc passed，1 doc ignored。
- `cargo test --manifest-path omb/Cargo.toml -p omobab --lib`：129 passed，1 ignored。
- `cargo test --manifest-path scripts/Cargo.toml -p omb-script-abi`：13 passed。
- `cargo test --manifest-path scripts/Cargo.toml -p base_content`：45 passed。
- `cargo test --manifest-path omfx/Cargo.toml -p omfx --lib`：123 passed。
- `openspec validate server-authoritative-selective-lockstep --strict`：valid。

