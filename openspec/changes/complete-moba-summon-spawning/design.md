# 設計

使用 generated/runtime summon catalog 作為唯一數值來源。事件 hook 依序預留 Entity；平行 tick 先計算，遇到生成要求則丟棄該次結果並依排序重跑，才預留實體。失敗 hook 取消預留，成功後由 outcome 提交元件。既有不生成的熱路徑維持平行。

到期使用固定點倒數並送 EntityRemoved，統一退休管線；不把到期視為擊殺。主人死亡不使已生成召喚物立即消失，PlayerOwner 快照保留獎勵歸因。
