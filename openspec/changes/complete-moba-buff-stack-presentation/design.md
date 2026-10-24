# 設計

所有Buff可透過JSON object數字欄位`stacks`提供正整數1..65535。空payload／缺欄向後相容為1；非法JSON、非數字、非整數、超界、超過4096 bytes或越界string ref拒絕該列。權威objective同時保留既有objective_stacks以相容消費端，新增通用stacks。不更動ABI33，不依ID判斷效果種類。

Unreal共用有界decoder，只有owner-local hero、正確generation與合法duration才解碼完整列表；typed StackCount仍在widget驗證。單層文字維持，複數顯示「×N」，期限維持原格式。player text差異即使diagnostic不變也會更新快取。既有空snapshot、死亡／新generation清除契約維持。
