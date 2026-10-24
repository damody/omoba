# Requirements

- 來源退休後，已承諾彈體仍呼叫發射時腳本。
- 只允許精確source entity generation的唯讀回退，不把退休來源重新標成alive。
- 追加傷害／分裂／彈射保持每發原始來源歸因及既有generation上限。
- 存活來源保留即時數值語意；其他單位不讀取來源快照。
- 來源快照、envelope與buff不公開到前端，無新ABI。
- 正式生成／飛行／命中／dispatch測試覆蓋退休、ID重用、追加傷害、子彈、清理及錯誤來源拒絕。
