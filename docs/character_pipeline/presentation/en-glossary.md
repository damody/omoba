# 英文說明用語表（v3 `text.en`）

適用範圍：英雄文件（`packages/<pkg>/v3.lua`）與道具文件（`items/*.lua`）中所有 `text.en` 的 `name` 與 `body`。

`tools\lua\lua.exe docs\character_pipeline\tools\v3_en_lint.lua` 會檢查 §3 能機械判定的規則。以下規則以 2026-10-29 普查時的多數寫法為準，這份表就是用語的唯一來源。

## 1. 用語

| 中文 | 英文 | 不要寫 |
|---|---|---|
| 魔法傷害／物理傷害／純粹傷害 | magic damage／physical damage／pure damage | magical damage、true damage |
| 魔免（魔法免疫） | spell immune；放在名詞前加連字號（spell-immune enemies、non-spell-immune units、spell-immune ones），當述語不加（he is spell immune）；名詞是 spell immunity | magic immune、magic immunity、magic-immune |
| 含魔免、無視魔免 | including spell-immune ones；pierces spell immunity | |
| 暈眩 | stun／stunned | daze |
| 定身、禁錮 | root／rooted（不能移動，但可以攻擊與施法） | snare、ensnare、immobilize |
| 沉默 | silence／silenced | |
| 繳械 | disarm／disarmed | |
| 緩速、減速 | slow／slowed；數值寫成 slows by {x}、{x} less movement speed | |
| 嘲諷 | taunt／taunted | |
| 驅散 | dispel（一般驅散、強驅散分別寫 basic dispel、strong dispel） | purge、cleanse |
| 隱形 | invisible／invisibility | stealth |
| 揭露、反隱 | reveal；看穿隱形寫 sees invisible units | |
| 真實視野 | true sight | |
| 部隊（非英雄單位） | units；「英雄與部隊」寫 heroes and units | troops、minions |
| 小兵 | creeps | soldiers |
| 野怪 | neutral creeps | |
| 召喚物 | summons；動詞是 summon | |
| 分身、幻象 | illusion | clone、mirror image |
| 建築 | buildings | structures、towers（只有在真的指防禦塔時才寫 tower） |
| 友方／敵方 | allied／enemy；名詞是 allies／enemies | friendly、hostile |
| 生命／最大生命 | health／maximum health | HP、hit points、max health、max HP |
| 法力／最大法力 | mana／maximum mana | MP |
| 生命回復／法力回復 | health regeneration／mana regeneration | health regen、hp regen |
| 回復生命 | heals for {x}；restores {x} health | |
| 攻擊力 | attack damage | damage（指攻擊力時） |
| 攻擊速度／移動速度 | attack speed／movement speed | move speed、moving speed、AS、MS |
| 護甲／魔抗 | armor／magic resistance | armour、magic resist（名詞）、spell resistance |
| 狀態抗性／緩速抗性 | status resistance／slow resistance | |
| 閃避 | evasion；動作是 evade | dodge（只用在「躲開投射物」） |
| 爆擊 | critical strike；縮寫寫 crit | |
| 吸血 | lifesteal；技能吸血是 spell lifesteal | life steal |
| 法術增幅 | spell amplification | spell amp |
| 引導、持續施法 | channel／channeling | channelling |
| 層數 | stacks；動詞 stack（「效果可以重疊」寫 stacks with itself） | layers |
| 次數、充能 | charges | |
| 冷卻 | cooldown | cool-down、CD |
| 射程、施法距離 | cast range；攻擊距離寫 attack range | |
| 範圍（半徑） | within {radius} | AoE、area of effect |
| 投射物 | projectile | missile、bullet（彈丸類技能的名稱除外） |
| 燃燒／中毒／流血 | burn／poison／bleed | |
| 擊退 | knock back（動詞）／knockback（名詞） | |
| 金錢／經驗 | gold／experience | XP |
| 樹 | trees | |
| 被動／主動／光環／開關 | Passive:／Active:／Aura:／Toggle:（句首前綴） | Use:、On use: |

## 2. 寫法

- **拼字**：一律用美式拼字，例如 armor、channeling、color、defense。
- **人稱**：
  - 英雄技能用第三人稱。英雄可以用短名（Nagayasu、Ginchiyo），之後用代名詞；代名詞以這位英雄文件中既有的寫法為準，不要改。
  - 道具以持有者為第二人稱（you、your）。同一份道具文件不要混用 the holder、the wearer。
  - 狀態說明描述的是帶著這個狀態的單位，可以省略主詞，例如 `Movement speed reduced by {slow}.`、`+{attack_speed} attack speed.`。
- **句首前綴**：同一段同時有主動與被動效果時，各自用 `Passive:`、`Active:` 開頭，冒號後接小寫動詞。
- **名稱**：
  - 用 Title Case，介系詞與冠詞小寫（`Immovable as the Mountain`、`Soul-Shattering Blade`）。
  - 日文羅馬字一律不加長音符號：Tosho Gongen、Nohime、Saito、Kosaka、Ryuzoji、Meito。長音直接省略，不寫成 ou、oo。
  - 同一個中文名稱，在不同文件中指同一件事時要用同一個英文名稱（§4）。
- **數字與佔位符**：
  - 說明裡不寫數字，一律用佔位符（`{duration}`）。佔位符的名稱與個數必須和 zh_TW 一致，驗證器會檢查。
  - 時間寫 `{x} seconds`（即使值可能是 1）。
  - 百分比佔位符本身會顯示 `%`，不要再加 percent。
  - 增益寫 `+{x} attack speed`；減益寫 `{x} less armor` 或 `reduces armor by {x}`。
- **`{@id}`**：引用另一招、狀態、道具或單位的名稱，顯示的是那個條目的英文名稱。前後的冠詞要讀得通，例如 `the {@b36w_mark}`。
- **不寫開發用語**：玩家看得到的說明（zh_TW 與 en）只描述實際效果。原作的錯誤、死碼、未實作或道具被註解等原因，寫在 quirks 或 notes，例如「施放後沒有任何效果，只消耗法力並進入冷卻」。
- **不寫內部用語**：不要出現 nobu、pool、modifier、handler、tick（玩家看得到的文字改成 every {interval} seconds）、kv、Lua。restarts the timer 這類一般英文可以用。規則層的機制改用玩家看得懂的說法，例如「與其他緩速相乘計算」。
- **語氣**：簡潔的遊戲說明。動詞用現在式第三人稱單數（Deals、Stuns、Grants），不用 will；句末加句號。
- **含括**：中文的「（含魔免）」寫成 `, including spell-immune ones` 或 `(spell-immune ones included)`；「（含自己）」寫成 `(including himself／herself／itself)`，依那位英雄的代名詞。

## 3. 機械檢查（v3_en_lint.lua）

會被擋下（error）：

- §1「不要寫」欄中的變體：magical damage、true damage、magic immune／immunity、max health、max HP、HP、hit points、move speed、armour、channelling、life steal、spell amp、AoE、Use:、On use:。
- 英雄文件出現第二人稱（you、your）。
- 名稱的第一個字母沒有大寫。
- 說明出現內部用語（nobu、modifier、handler、Lua、kv）。
- 說明出現開發用語。中文的「原作」「腳本」「死碼」「執行期」「錯誤」「bug」，英文的 original script、original game、script、bug、errors out、at runtime、commented out，zh_TW 與 en 都檢查。
- 英文出現長音符號（ā、ō、ū 等）。
- zh_TW 與 en 的佔位符不一致（與驗證器的規則相同，這裡逐條列出）。

只提醒（warning）：

- 同一個中文名稱有不同的英文名稱。
- 說明沒有以句號、問號、驚嘆號或右括號結尾。
- 連續兩個空白，或標點前有空白。

## 4. 共用名稱

指同一件事的中文名稱要用同一個英文名稱。以下是 2026-10-29 普查後的決定：

| 中文 | 英文 | 說明 |
|---|---|---|
| 疾如風 | Swift as the Wind | 《孫子》風林火山系列（技能、道具陣法、兵法書都一樣） |
| 徐如林 | Silent as the Forest | 同上 |
| 侵掠如火、侵略如火 | Fierce as Fire | 同上（Raid like Fire、Swift as Fire 都改掉） |
| 不動如山 | Immovable as the Mountain | 同上 |
| 〇〇之陣 | 〇〇 Formation | 例如 Wind Formation |
| 魔法免疫（狀態名稱） | Spell Immunity | |
| 隱形（狀態名稱） | Invisible | |
| 嘲諷 | 技能名稱用 Taunt，被嘲諷的狀態名稱用 Taunted | 技能與狀態可以不同 |
| 三河之勇 | Valor of Mikawa | A01 與 A05 同名 |
| 炎之迅捷 | Flame Swiftness | A35、B04 同名 |
| 忍法．〇〇／忍法密傳．〇〇 | Ninja Art: 〇〇／Secret Ninja Art: 〇〇 | 忍術．〇〇 是 Ninjutsu: 〇〇；奧義是 Secret Art:，秘奧義是 Hidden Secret Art: |
| 〇〇．銘刀 | Meito: 〇〇 | 例如 Meito: Ikeda Kijinmaru Kunishige。原本的 Kogatana（小刀）是誤譯；說明裡引用時只寫刀名 |
| 〇之卷（火、林、風、山、雷） | Book of 〇（Book of Fire、Book of Forest、Book of Wind、Book of Mountains、Book of Thunder） | 系列寫 Book of 〇 series |
| 孫子兵法．〇之章 | The Art of War: Chapter of 〇 | 用冒號，不用句點 |
| 雪走 | Yukibashiri | |
| 霧毒幻化 | Miasma Illusion | B33；原本的 Voodoo Power 與中文意思不符 |
| 般若湯 | Hannya Soup | 不寫 Prajna Wine |
| 般若之護（靴子與般若湯的魔免狀態） | Prajna Ward | 所有道具都用這個名稱 |

網站介面的文字（`scripts/site/i18n.lua`）也照這份用語表寫，例如 Pierces spell immunity、傷害類型 Magic。

同名但指不同東西的（例如技能與它附帶的狀態），可以用不同英文名稱，但要在那份文件的說明或 notes 裡讀得出差別。
