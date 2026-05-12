#!/usr/bin/env python3
"""Generate stress-test TD map.

Usage: python scripts/gen_stress_map.py
Writes to: D:/omoba/scripts/lua_data/TD_STRESS/map.lua

Optional environment variables:
  OMOBA_STRESS_CREEPS=2000
  OMOBA_STRESS_TOWERS=1000
  OMOBA_STRESS_SPAWN_INTERVAL=0.001
  OMOBA_STRESS_TOWER_SPACING=50
  OMOBA_STRESS_DIRECT_CREEPS=1
"""
import os
from pathlib import Path

OUT = Path("D:/omoba/scripts/lua_data/TD_STRESS/map.lua")

N_CREEPS = int(os.environ.get("OMOBA_STRESS_CREEPS", "2000"))
SPAWN_INTERVAL = float(os.environ.get("OMOBA_STRESS_SPAWN_INTERVAL", "0.001"))  # 秒
DIRECT_CREEPS = os.environ.get("OMOBA_STRESS_DIRECT_CREEPS", "1") != "0"

N_TOWERS = int(os.environ.get("OMOBA_STRESS_TOWERS", "1000"))
TOWER_SPACING = float(os.environ.get("OMOBA_STRESS_TOWER_SPACING", "50.0"))  # grid 間距（radius=25 時 50 間隔留 50% 空隙）

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

PATH_POINTS = [
    (-1400.0, -800.0),
    (1400.0, -800.0),
    (1400.0, -200.0),
    (-1400.0, -200.0),
    (-1400.0, 400.0),
    (1400.0, 400.0),
    (1400.0, 800.0),
    (-1400.0, 800.0),
]


def initial_creep_points(count):
    segments = []
    total = 0.0
    for i, ((x0, y0), (x1, y1)) in enumerate(zip(PATH_POINTS, PATH_POINTS[1:])):
        length = ((x1 - x0) ** 2 + (y1 - y0) ** 2) ** 0.5
        segments.append((i, x0, y0, x1, y1, length))
        total += length

    out = []
    for i in range(count):
        d = ((i + 0.5) / max(count, 1)) * total
        acc = 0.0
        for segment_idx, x0, y0, x1, y1, length in segments:
            if d <= acc + length:
                t = (d - acc) / length if length > 0.0 else 0.0
                out.append({
                    "Creep": "td_stress",
                    "Path": "td_main",
                    "PathIndex": segment_idx + 1,
                    "X": round(x0 + (x1 - x0) * t, 3),
                    "Y": round(y0 + (y1 - y0) * t, 3),
                })
                break
            acc += length
    return out


def stress_waves():
    if DIRECT_CREEPS:
        return []
    return [{
        "Name": "W_STRESS",
        "StartTime": 0.0,
        "Detail": [{
            "Path": "td_main",
            "Creeps": [
                {"Time": round(i * SPAWN_INTERVAL, 3), "Creep": "td_stress"}
                for i in range(N_CREEPS)
            ],
        }],
    }]

data = {
    "GameMode": "TowerDefense",
    "Path": [{
        "Name": "td_main",
        "Points": ["td_spawn", "td_cp1", "td_cp2", "td_cp3",
                   "td_cp4", "td_cp5", "td_cp6", "td_exit"],
    }],
    "Creep": [{
        "Name": "td_stress",
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
    # `scripts/lua_data/templates.lua` 的 towers[]，由 omoba-template-ids 編譯期生成
    # `TOWER_*_STATS` const，base_content 的 tower_ice / tower_bomb 腳本直接讀。
    # map.lua 的 Tower fallback 在 stress 場景永遠不會觸發。
    "Tower": [],
    "InitialCreeps": initial_creep_points(N_CREEPS) if DIRECT_CREEPS else [],
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
    "CreepWave": stress_waves(),
    "BlockedRegions": [],
}


def lua_string(value):
    return '"' + value.replace('\\', '\\\\').replace('"', '\\"') + '"'


def lua_key(key):
    return key if key.replace('_', '').isalnum() and not key[0].isdigit() else f"[{lua_string(key)}]"


def to_lua(value, indent=0):
    space = "  " * indent
    child = "  " * (indent + 1)
    if value is None:
        return "nil"
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, (int, float)):
        return repr(float(value)) if isinstance(value, float) else str(value)
    if isinstance(value, str):
        return lua_string(value)
    if isinstance(value, list):
        if not value:
            return "{}"
        lines = ["{"]
        for item in value:
            lines.append(f"{child}{to_lua(item, indent + 1)},")
        lines.append(f"{space}}}")
        return "\n".join(lines)
    if isinstance(value, dict):
        items = [(k, v) for k, v in value.items() if v is not None]
        if not items:
            return "{}"
        lines = ["{"]
        for key, item in items:
            lines.append(f"{child}{lua_key(key)} = {to_lua(item, indent + 1)},")
        lines.append(f"{space}}}")
        return "\n".join(lines)
    raise TypeError(type(value))

content = "return function(ctx)\n  return " + to_lua(data, 1) + "\nend\n"
OUT.parent.mkdir(parents=True, exist_ok=True)
if OUT.exists() and OUT.read_text(encoding="utf-8") == content:
    action = "unchanged"
else:
    OUT.write_text(content, encoding="utf-8")
    action = "wrote"

print(
    f"{action} {OUT}  towers={len(data['Structures'])}  "
    f"initial_creeps={len(data['InitialCreeps'])}  "
    f"wave_creeps={0 if DIRECT_CREEPS else len(data['CreepWave'][0]['Detail'][0]['Creeps'])}"
)
