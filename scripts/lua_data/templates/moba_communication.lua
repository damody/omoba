-- Original omoba text and semantic design, authored 2026-10-11.
-- Explicit append-only IDs; slots bind actions, declaration order does not.
-- Media omitted until imported and independently verified in Unreal.
return function(ctx)
  return {
    { catalog_id=1, id="trail_joke_v1", semantic="joke", interaction_slot=1, wheel_slot=0, label="打趣", subtitle="地圖這麼大，迷路也算探險。", duration_raw=4096 },
    { catalog_id=2, id="steady_challenge_v1", semantic="challenge", interaction_slot=2, wheel_slot=0, label="應戰", subtitle="站穩了，下一步見真章。", duration_raw=3072 },
    { catalog_id=3, id="camp_dance_v1", semantic="dance", interaction_slot=3, wheel_slot=0, label="起舞", subtitle="跟著腳步，換個節奏。", duration_raw=6144 },
    { catalog_id=4, id="bright_laugh_v1", semantic="laugh", interaction_slot=4, wheel_slot=0, label="開懷", subtitle="這一回，笑聲先到。", duration_raw=3072 },
    { catalog_id=5, id="comrade_salute_v1", semantic="salute", interaction_slot=5, wheel_slot=0, label="致意", subtitle="同路的人，向你致意。", duration_raw=3072 },
    { catalog_id=101, id="signal_received_v1", semantic="acknowledge", interaction_slot=0, wheel_slot=1, label="收到", subtitle="訊號收到，準備跟上。", duration_raw=3072 },
    { catalog_id=102, id="fine_step_v1", semantic="applaud", interaction_slot=0, wheel_slot=2, label="喝采", subtitle="漂亮的一步！", duration_raw=3072 },
    { catalog_id=103, id="shared_thanks_v1", semantic="thanks", interaction_slot=0, wheel_slot=3, label="感謝", subtitle="有你支援，這條路更穩。", duration_raw=3072 },
    { catalog_id=104, id="wide_eyes_v1", semantic="surprise", interaction_slot=0, wheel_slot=4, label="驚訝", subtitle="這一幕，值得再看一眼。", duration_raw=3072 },
    { catalog_id=105, id="hold_resolve_v1", semantic="resolve", interaction_slot=0, wheel_slot=5, label="堅定", subtitle="穩住，我們還有下一步。", duration_raw=4096 },
    { catalog_id=106, id="take_a_breath_v1", semantic="patience", interaction_slot=0, wheel_slot=6, label="稍候", subtitle="先吸口氣，等個好時機。", duration_raw=4096 },
    { catalog_id=107, id="small_victory_v1", semantic="celebrate", interaction_slot=0, wheel_slot=7, label="慶賀", subtitle="這份成果，大家都有一份。", duration_raw=4096 },
    { catalog_id=108, id="together_again_v1", semantic="regroup", interaction_slot=0, wheel_slot=8, label="同行", subtitle="重新並肩，繼續向前。", duration_raw=4096 },
  }
end
