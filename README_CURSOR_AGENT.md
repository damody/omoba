# MOBA 遊戲專案 - Cursor Agent 指南

## 🎯 專案概述

這是一個完整的 MOBA (Multiplayer Online Battle Arena) 遊戲專案，包含後端服務器和前端客戶端，使用 Rust 開發。專案採用 Git Submodule 架構，將前後端分離為獨立的儲存庫。

## 📁 專案結構

```
moba/                          # 主專案目錄
├── omb/                       # 後端 submodule (omobab)
│   ├── src/                   # Rust 後端源碼
│   ├── ability-system/        # 技能系統子模組
│   ├── ability-configs/       # 技能配置檔案
│   ├── Cargo.toml            # 後端依賴配置
│   └── README.md             # 後端詳細文檔
├── omf/                       # 前端 submodule (omobaf)
│   ├── src/                   # Rust 前端源碼
│   ├── Cargo.toml            # 前端依賴配置
│   └── README.md             # 前端詳細文檔
├── mqtt_log_viewer/           # MQTT 日誌查看器 submodule
│   ├── src/                   # Rust 日誌查看器源碼
│   ├── Cargo.toml            # 日誌查看器依賴配置
│   └── README.md             # 日誌查看器詳細文檔
├── specs/                     # ECS 框架 submodule
│   ├── src/                   # Specs ECS 源碼
│   ├── specs-derive/          # 衍生巨集
│   ├── examples/              # 範例程式
│   ├── Cargo.toml            # ECS 框架依賴配置
│   └── README.md             # ECS 框架文檔
├── log4rs/                    # 日誌框架 submodule
│   ├── src/                   # Log4rs 源碼
│   ├── examples/              # 日誌配置範例
│   ├── docs/                  # 配置文檔
│   ├── Cargo.toml            # 日誌框架依賴配置
│   └── README.md             # 日誌框架文檔
├── .gitmodules               # Git submodule 配置
└── README_CURSOR_AGENT.md    # 本文件
```

## 🔗 Submodule 關係

### omb/ (Open MOBA Backend)
- **用途**: 遊戲後端服務器
- **技術**: Rust + ECS 架構 + MQTT
- **遠端**: https://github.com/damody/open_moba_backend.git
- **主要功能**:
  - 即時多人對戰邏輯
  - ECS 實體組件系統
  - 技能系統與配置
  - MQTT 通信協議
  - 視野計算與優化

### omf/ (Open MOBA Frontend)
- **用途**: 遊戲前端客戶端 (測試用)
- **技術**: Rust + CLI + MQTT
- **遠端**: https://github.com/damody/open_moba_frontend.git
- **主要功能**:
  - 模擬玩家操作
  - MQTT 客戶端通信
  - 遊戲狀態同步
  - 自動化測試

### mqtt_log_viewer/ (MQTT Log Viewer)
- **用途**: MQTT 訊息記錄檢視器
- **技術**: Rust + TUI + SQLite + MQTT
- **遠端**: https://github.com/damody/mqtt_log_viewer.git
- **主要功能**:
  - 即時接收和儲存 MQTT 訊息
  - 三層互動式介面設計
  - 強大的過濾功能 (Topic、Payload、時間範圍)
  - JSON 美化顯示
  - 訊息刪除和複製功能
  - Windows API 按鍵偵測

### specs/ (ECS Framework)
- **用途**: Entity Component System 框架
- **技術**: Rust + ECS + 衍生巨集
- **遠端**: https://github.com/damody/specs.git
- **主要功能**:
  - 高性能 ECS 架構實現
  - 實體、組件、系統管理
  - 並行系統執行
  - 衍生巨集支援
  - 資源管理系統
  - 事件系統

### log4rs/ (Logging Framework)
- **用途**: 結構化日誌框架
- **技術**: Rust + YAML 配置 + 多種輸出格式
- **遠端**: https://github.com/damody/log4rs.git
- **主要功能**:
  - 多種日誌輸出格式 (JSON, 文字, 自定義)
  - 日誌輪轉和檔案管理
  - 多層級日誌過濾
  - 非同步日誌記錄
  - 自定義編碼器和附加器
  - 配置檔案驅動

## 🛠️ 開發環境設置

### 前置需求
- Rust 1.70+
- Git (支援 submodule)
- MQTT Broker (如 Mosquitto)

### 初始化專案
```bash
# 克隆主專案 (包含 submodule)
git clone --recursive https://github.com/damody/omoba.git
cd omoba

# 或更新現有 submodule
git submodule update --init --recursive
```

### 後端開發
```bash
cd omb/
cargo build --release
cargo run --release
```

### 前端開發
```bash
cd omf/
cargo build --release
cargo run -- --help
```

### MQTT 日誌查看器開發
```bash
cd mqtt_log_viewer/
cargo build --release
cargo run
```

### ECS 框架開發
```bash
cd specs/
cargo build
cargo test
cargo run --example simple
```

### 日誌框架開發
```bash
cd log4rs/
cargo build
cargo test
cargo run --example custom
```

### MQTT 日誌查看器開發
```bash
cd mqtt_log_viewer/
cargo build --release
cargo run
```

## 🎮 遊戲架構

### 後端 (omb) 核心系統
1. **ECS 架構**: 使用自定義 Specs 框架 (specs/ submodule)
2. **遊戲循環**: 10 TPS 更新頻率
3. **通信協議**: MQTT 訊息傳遞
4. **技能系統**: JSON 配置驅動
5. **視野系統**: 四叉樹空間分割優化

### 前端 (omf) 核心功能
1. **玩家模擬**: 自動化遊戲操作
2. **狀態同步**: 與後端實時同步
3. **英雄支援**: 雜賀孫一、伊達政宗
4. **技能測試**: 完整的技能系統測試

### MQTT 日誌查看器 (mqtt_log_viewer) 核心功能
1. **即時監控**: 即時接收和顯示 MQTT 訊息
2. **資料持久化**: SQLite 資料庫儲存
3. **互動式介面**: 三層 TUI 介面設計
4. **過濾功能**: Topic、Payload、時間範圍過濾
5. **JSON 處理**: 自動美化和格式化 JSON 內容

### ECS 框架 (specs) 核心功能
1. **實體管理**: 高效的實體創建和銷毀
2. **組件系統**: 靈活的組件添加和移除
3. **系統執行**: 並行系統處理
4. **資源管理**: 全域資源共享
5. **事件系統**: 實體間通信機制

### 日誌框架 (log4rs) 核心功能
1. **多格式輸出**: 支援 JSON、文字、自定義格式
2. **日誌輪轉**: 基於時間和檔案大小的輪轉策略
3. **層級過濾**: 靈活的日誌級別控制
4. **非同步記錄**: 高性能的非阻塞日誌記錄
5. **配置驅動**: YAML 配置檔案管理
6. **自定義擴展**: 支援自定義編碼器和附加器

## 🔧 常用開發指令

### Git Submodule 操作
```bash
# 更新所有 submodule
git submodule update --recursive

# 更新特定 submodule
git submodule update --remote omb/
git submodule update --remote omf/
git submodule update --remote mqtt_log_viewer/
git submodule update --remote specs/
git submodule update --remote log4rs/

# 提交 submodule 變更
git add omb/ omf/ mqtt_log_viewer/ specs/ log4rs/
git commit -m "Update submodules"
```

### 後端開發
```bash
cd omb/
# 編譯
cargo build

# 運行
cargo run

# 測試
cargo test

# 檢查程式碼
cargo clippy
cargo fmt
```

### 前端開發
```bash
cd omf/
# 編譯
cargo build --release

# 運行測試
cargo run -- play --hero saika_magoichi

# 自動遊戲測試
cargo run -- auto --duration 60
```

## 📊 專案特色

### 技術亮點
- **高性能**: 多線程 ECS 架構
- **模組化**: 完全重構的模組化設計
- **事件驅動**: 統一的事件分派系統
- **配置驅動**: JSON 技能配置系統
- **實時通信**: MQTT 低延遲通信

### 架構優勢
- **前後端分離**: 獨立的 submodule 管理
- **可擴展性**: 模組化設計易於擴展
- **測試友好**: 完整的前端測試工具
- **開發效率**: 熱重載和快速迭代

## 🐛 除錯指南

### 常見問題
1. **Submodule 更新問題**
   ```bash
   git submodule update --init --recursive
   ```

2. **MQTT 連接失敗**
   - 檢查 `omb/game.toml` 配置
   - 確認 MQTT Broker 運行狀態

3. **編譯錯誤**
   - 檢查 Rust 版本: `rustc --version`
   - 清理編譯緩存: `cargo clean`

### 日誌調試
```bash
# 後端詳細日誌
cd omb/
RUST_LOG=debug cargo run

# 前端詳細日誌
cd omf/
RUST_LOG=debug cargo run -- play --verbose
```

## 🚀 快速開始

1. **啟動後端**
   ```bash
   cd omb/
   cargo run --release
   ```

2. **測試前端**
   ```bash
   cd omf/
   cargo run -- play --hero saika_magoichi
   ```

3. **自動測試**
   ```bash
   cargo run -- auto --duration 30
   ```

## 📝 開發注意事項

### 程式碼規範
- 遵循 Rust 官方風格指南
- 使用 `cargo fmt` 格式化
- 使用 `cargo clippy` 檢查
- 添加適當的註釋和文檔

### 提交規範
- 後端變更: 在 `omb/` 目錄下提交
- 前端變更: 在 `omf/` 目錄下提交
- 主專案變更: 在主目錄下提交 submodule 更新

### 測試策略
- 單元測試: 在各個 submodule 中
- 整合測試: 使用前端模擬器
- 性能測試: 監控 TPS 和記憶體使用

## 🔗 相關連結

- [後端詳細文檔](omb/README.md)
- [前端詳細文檔](omf/README.md)
- [MQTT 日誌查看器文檔](mqtt_log_viewer/README.md)
- [ECS 框架文檔](specs/README.md)
- [日誌框架文檔](log4rs/README.md)
- [後端儲存庫](https://github.com/damody/open_moba_backend)
- [前端儲存庫](https://github.com/damody/open_moba_frontend)
- [MQTT 日誌查看器儲存庫](https://github.com/damody/mqtt_log_viewer)
- [ECS 框架儲存庫](https://github.com/damody/specs)
- [日誌框架儲存庫](https://github.com/damody/log4rs)

---

**注意**: 這是一個活躍開發中的專案，架構和功能可能會持續演進。建議定期更新 submodule 以獲取最新功能。