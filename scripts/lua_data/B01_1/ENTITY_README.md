# Entity.json 單位定義文件說明

本文檔說明 `entity.json` 的結構與用途，該文件定義了戰役中所有可用的遊戲單位，包括英雄、敵人、小兵、中立生物和召喚物等。

## 📋 檔案結構概覽

```json
{
    "heroes": {},          // 英雄單位定義
    "enemyHeroes": {},     // 敵方英雄/強敵
    "creeps": {},          // 小兵單位
    "neutrals": {},        // 中立生物
    "summons": {}          // 召喚物
}
```

## 🦸 英雄單位 (Heroes)

定義玩家可控制的英雄角色。

### 基本結構

```json
{
    "heroId": {
        "name": "英雄名稱",
        "title": "英雄稱號",
        "type": "Hero",
        "mainAttribute": "Strength|Agility|Intelligence",
        "baseStats": {},
        "attributes": {},
        "abilities": {},
        "talents": {}
    }
}
```

### 屬性說明

#### baseStats（基礎數值）
| 欄位 | 類型 | 說明 |
|------|------|------|
| hp | number | 基礎生命值 |
| mp | number | 基礎魔法值 |
| armor | number | 基礎護甲值 |
| magicResist | number | 魔法抗性（%） |
| moveSpeed | number | 移動速度 |
| attackDamage | number | 基礎攻擊力 |
| attackRange | number | 攻擊距離 |
| attackSpeed | number | 每秒攻擊次數 |
| hpRegen | number | 每秒生命回復 |
| mpRegen | number | 每秒魔法回復 |
| turnRate | number | 轉向速度 |
| visionDay/Night | number | 白天/夜晚視野距離 |

#### attributes（三圍屬性）
| 欄位 | 類型 | 說明 |
|------|------|------|
| strength | number | 初始力量值 |
| agility | number | 初始敏捷值 |
| intelligence | number | 初始智力值 |
| strengthGain | number | 每級力量成長 |
| agilityGain | number | 每級敏捷成長 |
| intelligenceGain | number | 每級智力成長 |

#### abilities（技能系統）
```json
{
    "Q|W|E|R": {
        "id": "技能唯一識別符",
        "name": "技能名稱",
        "description": "技能描述",
        "abilityType": "TargetUnit|TargetPoint|TargetDirection|NoTarget",
        "damageType": "Physical|Magical|Pure",
        "cooldown": [8, 7, 6, 5],      // 各等級冷卻時間
        "manaCost": [70, 80, 90, 100], // 各等級魔法消耗
        "damage": [120, 180, 240, 300] // 各等級傷害
        // ... 其他技能特定屬性
    }
}
```

**技能類型說明：**
- `TargetUnit`: 指定單位目標
- `TargetPoint`: 指定地面位置
- `TargetDirection`: 指定方向施放
- `NoTarget`: 無需目標，立即施放

**傷害類型說明：**
- `Physical`: 物理傷害，受護甲影響
- `Magical`: 魔法傷害，受魔抗影響
- `Pure`: 純粹傷害，無視防禦

## 👹 敵方單位 (EnemyHeroes)

定義 AI 控制的敵方強敵。

### 特殊屬性

#### phases（階段系統）
用於 Boss 級敵人的分階段戰鬥：

```json
{
    "phases": {
        "phase1": {
            "hpThreshold": 1.0,                    // 血量閾值
            "abilities": ["技能1", "技能2"],        // 該階段可用技能
            "bonusStats": {                        // 額外屬性加成
                "attackSpeed": 0.2,
                "moveSpeed": 20
            }
        }
    }
}
```

#### AI 行為模式
| 模式 | 說明 |
|------|------|
| `kite_caster` | 風箏型法師，保持距離施法 |
| `aggressive_caster` | 激進型法師，主動進攻 |
| `boss_intelligent` | 智能型 Boss，複雜行為模式 |

## 🥷 小兵單位 (Creeps)

定義戰場上的小兵單位。

### 陣營分類
- `Radiant`: 光明陣營（友方）
- `Dire`: 暗影陣營（敵方）

### 小兵類型
| 類型 | 特點 |
|------|------|
| 近戰兵 | 高血量，近距離攻擊 |
| 遠程兵 | 中等血量，遠距離攻擊 |
| 攻城車 | 高血量高護甲，對建築額外傷害 |
| 超級兵 | 精英單位，提供光環效果 |

### 特殊屬性
```json
{
    "collisionSize": 16,                    // 碰撞體積
    "projectileSpeed": 900,                 // 投射物速度（遠程單位）
    "bonusDamageToStructures": 2.5,         // 對建築額外傷害倍率
    "ancientAura": true                     // 是否提供光環效果
}
```

## 🐺 中立生物 (Neutrals)

定義野怪營地的中立生物。

### 營地等級
| 等級 | 說明 | 獎勵範圍 |
|------|------|----------|
| Small | 小型營地 | 40-50 金幣 |
| Medium | 中型營地 | 60-80 金幣 |
| Large | 大型營地 | 100-150 金幣 |
| Ancient | 遠古營地 | 300+ 金幣 |

### 特殊能力
中立生物可能擁有被動或主動技能：

```json
{
    "abilities": {
        "frost_attack": {
            "name": "冰霜攻擊",
            "passive": true,          // 被動技能
            "slow": 0.2,             // 減速效果
            "duration": 1.5          // 持續時間
        }
    }
}
```

## 👥 召喚物 (Summons)

由技能或物品召喚的臨時單位。

### 召喚物屬性
```json
{
    "summonedBy": "forest_scroll",    // 召喚來源
    "duration": 30,                   // 存在時間
    "bounty": 10,                     // 擊殺獎勵（通常較低）
    "experience": 10                  // 經驗值獎勵
}
```

## 💡 設計原則與平衡考量

### 1. 數值平衡
- **血量 vs 傷害**: 高血量單位通常傷害較低
- **射程 vs 血量**: 遠程單位血量通常較低
- **特殊能力**: 擁有強力技能的單位基礎屬性相對較弱

### 2. 角色定位
- **Tank**: 高血量、高護甲、低傷害
- **DPS**: 中等血量、高傷害、特殊技能
- **Support**: 低血量、輔助技能、群體效果

### 3. 屬性成長
- **線性成長**: 大部分屬性按固定值成長
- **主屬性加成**: 英雄主屬性提供額外攻擊力
- **等級差異**: 高等級單位在各方面都有明顯優勢

## 🔧 技術實現注意事項

### 1. 性能優化
- 合理設置碰撞體積避免卡位
- 投射物速度影響命中判定
- 技能冷卻時間影響戰鬥節奏

### 2. AI 行為
- 不同 AI 模式需要對應的行為邏輯
- Boss 階段切換需要平滑過渡
- 召喚物需要合適的 AI 指令

### 3. 數值驗證
- 攻擊速度不應超過合理上限
- 技能傷害需要考慮等級差異
- 護甲和魔抗的減傷公式要平衡

## 📝 範例：完整英雄定義

```json
{
    "B01_SaikaMagoichi": {
        "name": "雜賀孫市",
        "title": "千里狙擊手",
        "type": "Hero",
        "mainAttribute": "Agility",
        
        "baseStats": {
            "hp": 550,
            "mp": 300,
            "armor": 3,
            "magicResist": 25,
            "moveSpeed": 280,
            "attackDamage": 52,
            "attackRange": 900,
            "attackSpeed": 0.7,
            "hpRegen": 1.5,
            "mpRegen": 1.0
        },
        
        "attributes": {
            "strength": 18,
            "agility": 24,
            "intelligence": 16,
            "strengthGain": 1.8,
            "agilityGain": 3.2,
            "intelligenceGain": 1.5
        },
        
        "abilities": {
            "Q": {
                "id": "snipe_shot",
                "name": "狙擊射擊",
                "abilityType": "TargetUnit",
                "damageType": "Physical",
                "cooldown": [8, 7, 6, 5],
                "manaCost": [70, 80, 90, 100],
                "damage": [120, 180, 240, 300]
            }
        }
    }
}
```

## 🛠️ 擴展與維護

### 1. 新增單位
1. 確定單位類型和定位
2. 設計基礎屬性和技能
3. 平衡測試和調整
4. 更新相關文檔

### 2. 技能設計
1. 明確技能機制和數值
2. 考慮與其他技能的配合
3. 設置合理的冷卻和消耗
4. 測試各等級的平衡性

### 3. 數值調整
1. 收集遊戲數據和玩家反饋
2. 識別過強或過弱的單位
3. 進行小幅度調整測試
4. 記錄變更歷史和原因

---

此文檔會隨著遊戲內容更新而持續維護，請定期檢查最新版本。