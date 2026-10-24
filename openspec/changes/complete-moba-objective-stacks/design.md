# 設計

objective_buff.max_stacks缺欄為1，上限16；同隊同buff在未到期且政策相同時加一封頂，所有層共享最新取得時起算的duration。過期或政策不同重設一層。每層攻擊／護甲效果以固定整數倍套用，始終只有一個普通BuffStore entry，不疊加多份重複修飾。

隊伍lease保存stacks並進入digest；原有lost_on_death、死亡隊員資格、重生剩餘時間、pause、結束清理均沿既有流程。重複Outcome／匿名／強制／副體死亡不可增加；正式營地死亡才授予。北史詩維持max1，南史詩定義max4、每層15護甲，仍有120秒有效期，非永久累積。正式Unreal沿既有公開戰鬥屬性與Buff顯示，ABI33不擴張。
