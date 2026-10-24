# 設計

post-outcome／post-buff-timer從AbilityWindups驗證當前生命、HP、位置、rank、silence／stun、recall與目標生命，產生無戰鬥modifier的native-only cast_preparation公開Buff。payload含total_raw與精確十進位instance，Buff剩時即權威進度。資料append至既有目錄，保留舊ID／CABI34；沿正式filtered buff資料流抵達可見英雄。

Unreal有界解碼／generation與衝突守門，typed FrameState驅動native cast clip，依權威進度校正位置及播放速率。準備優先於移動／hit，death／respawn／recall優先；取消回最新locomotion，release清準備並重播獨立cast瞬態。epoch／generation／完整缺項清除；退出視野清姿態，重新可見只恢復新鮮當前進度。每170秒監測預留排程餘裕。
