# hero_smoke_astral_ranger Review

## AI-made assumptions

- 使用 stylized isometric MOBA 可讀性作為預設美術方向。
- 數值只作 draft，不代表正式平衡。
- 第一版只產生 package，不產生實際 PNG/GLB。

## Character identity and role

`hero_smoke_astral_ranger` 是遠程 marksman，核心辨識是深紅長外套、compact arc rifle、cyan crystal glow。

## Skill kit coherence

Q 是穿透線性射擊，W 是標記，E 是位移，R 是標記後的 charged shot。技能組在玩法與視覺 motif 上一致。

## Isometric readability

長外套與 rifle 形成清楚剪影。需要避免過多細碎掛飾，否則 3D 化後在俯視角會變成噪點。

## Icon readability

四個 icon 都使用單一主形狀：彈道、星痕、殘影、彗星光束。小尺寸應可辨識。

## Model and rig risks

長外套可能需要簡化或用較硬的裙擺權重。rifle 建議獨立 weapon bone 或 hand attach。

## Animation coverage

已覆蓋 `idle`、`run`、`attack`、`cast_q`、`cast_w`、`cast_e`、`cast_r`、`death`。

## omoba import readiness

`omoba_stub.lua` 已列出 hero id、ability ids、portrait/icon/model slots、animation clips、script hints 與 draft stats。

## GameContractWarning

- `omfx/data/heroes/hero_smoke_astral_ranger/hero_smoke_astral_ranger.glb` 是未來 import slot，第一版不會建立該檔。
- `hero_smoke_astral_ranger_e` mobility 行為可能需要 script/runtime 支援，正式實作前需確認 movement effect API。
