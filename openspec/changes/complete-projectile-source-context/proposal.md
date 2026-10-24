# Projectile source context

彈體來源退休後，正式命中事件失去ScriptUnitTag、升級旗標、屬性與陣營，導致追加傷害／分裂／彈射遺失。以每發authority來源快照與有界來源封套修正，維持存活來源即時查詢與entity generation隔離。範圍僅正式Rust runtime與內容測試，不改ABI／公開wire、不新增對局。
