@echo off
REM ============================================================
REM  test_kp.bat — 快速測試 KP 發放
REM  debug 難度：1 條命 / 免費塔 / 3 波
REM  第 1 波小兵穿越即觸發失敗 → 應看到 player_profile.json 被建立
REM ============================================================
setlocal
pushd "%~dp0"

set "OMB_DIFFICULTY=debug"

echo [test_kp] 以 debug 難度啟動（1 條命、3 波）
echo [test_kp] 小兵穿越 1 隻 → 失敗 → 應發放 KP +3
echo [test_kp] 預期：omb\player_profile.json 在關卡結束後出現
echo.

call run.bat %*
