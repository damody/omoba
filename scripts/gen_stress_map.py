#!/usr/bin/env python3
"""Generate stress-test TD map (1000 towers + 1000 creeps).

Usage: python scripts/gen_stress_map.py
Writes to: D:/omoba/omb/Story/TD_STRESS/map.json
"""
import json
from pathlib import Path

OUT = Path("D:/omoba/omb/Story/TD_STRESS/map.json")

N_CREEPS = 1000
CREEP_HP = 10000.0
CREEP_SPEED = 100.0
SPAWN_INTERVAL = 0.1  # 秒

N_TOWERS = 1000
TOWER_SPACING = 50.0  # grid 間距；塔 radius=50 時緊貼但不重疊

# 走廊範圍（避開 U 字路徑的四條水平線 Y=-800/-200/400/800，各留 60px 安全距離）
CORRIDORS = [
    (-1340.0, 1340.0, -740.0, -260.0),  # A
    (-1340.0, 1340.0, -140.0,  340.0),  # B
    (-1340.0, 1340.0,  460.0,  740.0),  # C
]


def grid_points(corridors, spacing, limit):
    out = []
    for (xmin, xmax, ymin, ymax) in corridors:
        x = xmin
        while x <= xmax and len(out) < limit:
            y = ymin
            while y <= ymax and len(out) < limit:
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
    "Tower": [{
        "Name": "stress_tower",
        "Property": {"Hp": 1000, "Block": 0},
        "Attack": {"Range": 200.0, "AttackSpeed": 1.0, "Physic": 20.0, "Magic": 0.0},
        "TurnSpeed": 360.0,
        "CollisionRadius": 50.0,
    }],
    "Structures": [
        {
            "Tower": "stress_tower",
            "Faction": "Player",
            "X": float(x),
            "Y": float(y),
            "IsBase": False,
            "CollisionRadius": None,
        }
        for (x, y) in pts[:N_TOWERS]
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
