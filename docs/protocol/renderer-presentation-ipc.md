# Renderer Presentation IPC

`omoba-client-runtime`與renderer只在loopback TCP交換`proto/game.proto`的
`RendererIpcEnvelope`。Wire frame為4-byte big-endian payload length加protobuf bytes；
`magic = 0x4f4d5254`、`protocol_version = 1`，單一frame上限8 MiB。

Rust omfx使用`omoba-core::game_proto`產生型別。未來Unreal build應直接以同一份
`proto/game.proto`產生C++型別，不得複製欄位編號或透過Rust ABI讀取runtime記憶體。
Unreal端只需實作：loopback connect、length framing、envelope版本檢查、latest-wins
snapshot、ordered critical result及renderer input。`render_id`只在單一team replica與
disclosure epoch內有效；它不是canonical Specs entity ID。

Renderer不得自行執行Specs systems、載入script DLL、連authoritative KCP、推算hidden
visibility或把remembered ghost當成可target entity。遇到unknown version/message、過長
frame或非loopback endpoint必須fail closed。
