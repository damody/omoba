#!/usr/bin/env python3
"""Generate stress-test TD map (1000 towers + 1000 creeps).

Usage: python scripts/gen_stress_map.py
Writes to: D:/omoba/omb/Story/TD_STRESS/map.json
"""
import json
from pathlib import Path

OUT = Path("D:/omoba/omb/Story/TD_STRESS/map.json")

N_CREEPS = 1000
CREEP_HP = 10_000_000.0  # 1000× 原 10K — stress profile 要 creep 活久才能讓塔持續開火
CREEP_SPEED = 100.0
SPAWN_INTERVAL = 0.1  # 秒

N_TOWERS = 1000
TOWER_SPACING = 50.0  # grid 間距（radius=25 時 50 間隔留 50% 空隙）

# 走廊範圍（避開 U 字路徑的四條水平線 Y=-800/-200/400/800，各留 60px 安全距離）
CORRIDORS = [
    (-1340.0, 1340.0, -740.0, -260.0),  # A
    (-1340.0, 1340.0, -140.0,  340.0),  # B
    (-1340.0, 1340.0,  460.0,  740.0),  # C
]


# 英雄起始點 (0,0) 周圍留一個淨空圈，否則 hero 一生出來就被 corridor B 的塔
# 包住，rmb 移動因為 collision 推不動，看起來像「英雄完全不會動」。
HERO_CLEAR_RADIUS = 250.0  # backend units；hero coll_radius=30 + tower coll_radius=50 + 餘裕


def grid_points(corridors, spacing, limit):
    out = []
    r2 = HERO_CLEAR_RADIUS * HERO_CLEAR_RADIUS
    for (xmin, xmax, ymin, ymax) in corridors:
        x = xmin
        while x <= xmax and len(out) < limit:
            y = ymin
            while y <= ymax and len(out) < limit:
                # 跳過英雄淨空圈內的 grid 點
                if x * x + y * y >= r2:
                    out.append((x, y))
                y += spacing
            x += spacing
    return out


pts = grid_points(CORRIDORS, TOWER_SPACING, N_TOWERS)
assert len(pts) >= N_TOWERS, (
    f"Grid容量不夠：{len(pts)} < {N_TOWERS}（調小 TOWER_SPACING 或擴大 CORRIDORS）"
)

data = {
    "GameMode": "TowerDefense",
    "Path": [{
        "Name": "td_main",
        "Points": ["td_spawn", "td_cp1", "td_cp2", "td_cp3",
                   "td_cp4", "td_cp5", "td_cp6", "td_exit"],
    }],
    "Creep": [{
        "Name": "td_stress",
        "Label": "壓測怪",
        "HP": CREEP_HP,
        "DefendPhysic": 0.0,
        "DefendMagic": 0.0,
        "MoveSpeed": CREEP_SPEED,
        "Faction": "Enemy",
        "TurnSpeed": None,
        "CollisionRadius": None,
    }],
    "CheckPoint": [
        {"Name": "td_spawn", "Class": "Spawn", "X": -1400.0, "Y": -800.0},
        {"Name": "td_cp1",   "Class": "Path",  "X":  1400.0, "Y": -800.0},
        {"Name": "td_cp2",   "Class": "Path",  "X":  1400.0, "Y": -200.0},
        {"Name": "td_cp3",   "Class": "Path",  "X": -1400.0, "Y": -200.0},
        {"Name": "td_cp4",   "Class": "Path",  "X": -1400.0, "Y":  400.0},
        {"Name": "td_cp5",   "Class": "Path",  "X":  1400.0, "Y":  400.0},
        {"Name": "td_cp6",   "Class": "Path",  "X":  1400.0, "Y":  800.0},
        {"Name": "td_exit",  "Class": "Base",  "X": -1400.0, "Y":  800.0},
    ],
    # Tower templates 留空 — 實際 spawn 走 spawn_td_tower 從 TowerTemplateRegistry
    # 取，數值（atk / range / cost / footprint / ...）唯一來源是
    # `omb/Story/templates.json` 的 towers[]，由 omoba-template-ids 編譯期生成
    # `TOWER_*_STATS` const，base_content 的 tower_ice / tower_bomb 腳本直接讀。
    # map.json 的 Tower fallback 在 stress 場景永遠不會觸發。
    "Tower": [],
    # Structures 按 grid index 交錯：偶數 ice / 奇數 bomb。兩種 script 各 ~500 個，
    # 可在 tick_profile 看到 per-script-id 的耗時對比。
    "Structures": [
        {
            "Tower": "tower_ice" if i % 2 == 0 else "tower_bomb",
            "Faction": "Player",
            "X": float(x),
            "Y": float(y),
            "IsBase": False,
            "CollisionRadius": None,
        }
        for i, (x, y) in enumerate(pts[:N_TOWERS])
    ],
    "CreepWave": [{
        "Name": "W_STRESS",
        "StartTime": 0.0,
        "Detail": [{
            "Path": "td_main",
            "Creeps": [
                {"Time": round(i * SPAWN_INTERVAL, 3), "Creep": "td_stress"}
                for i in range(N_CREEPS)
            ],
        }],
    }],
    "BlockedRegions": [],
}

OUT.parent.mkdir(parents=True, exist_ok=True)
OUT.write_text(json.dumps(data, indent=2, ensure_ascii=False), encoding="utf-8")
print(
    f"wrote {OUT}  towers={len(data['Structures'])}  "
    f"creeps={len(data['CreepWave'][0]['Detail'][0]['Creeps'])}"
)
