# 施法進度錯誤紀錄

- P01：只有前搖tooltip沒有權威動態進度；缺結果時不能用cooldown推斷成功，改明確記錄取消／釋放交易結果。
- P02：OpenSpec CLI已於前輪確認不可用，依既有格式維護artifacts。protoc缺少時既有build使用checked-in generated fallback，本輪同步proto及fallback並驗證codec，不新增工作流fallback。
- P03：首次集中Rust驗收client-runtime新測試的state由metric輸入推斷為i64，與protobuf u32直接assert_eq導致E0308／E0277；改測試明確checked conversion，不修改正式狀態驗證。保留V1九工作與失敗log，待V1 worker／watcher退出後重跑完整V2，不能拿其他package通過掩蓋client未編譯。
- P04：WorldV1的RuntimePresentationReset仍有4個既有preview-world DestroyActor「World has no context」警告；35案為34 Success＋1 SuccessWithWarnings，0 Failed／NotRun。新增CastProgressLifecycle／CastWindupTooltip均零警告，保留原report／engine log，不寫成無警告驗收。
