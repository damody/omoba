//! Transport-independent wire tag identities. No socket or feature dependency.

// KCP 成幀協定的標記常數
pub const TAG_PLAYER_COMMAND: u8 = 0x01;
pub const TAG_GAME_EVENT: u8 = 0x02;
pub const TAG_COMMAND_ACK: u8 = 0x03;
pub const TAG_SUBSCRIBE_REQUEST: u8 = 0x04;
pub const TAG_GAME_STATE_REQUEST: u8 = 0x05;
pub const TAG_GAME_STATE_RESPONSE: u8 = 0x06;
pub const TAG_VIEWPORT_UPDATE: u8 = 0x07;

// 第 2 階段鎖步標籤。範圍 0x10..=0x18。 COMPRESSION_FLAG (0x80) 位
// 這些也未使用，因此相同的 write_framed / read_framed 路徑可以工作。
pub const TAG_INPUT_SUBMIT: u8 = 0x10; // C→S
pub const TAG_TICK_BATCH: u8 = 0x11; // S→C broadcast (lockstep_joined sessions)
pub const TAG_STATE_HASH: u8 = 0x12; // S→C broadcast (lockstep_joined sessions)
pub const TAG_JOIN_REQUEST: u8 = 0x13; // C→S
pub const TAG_GAME_START: u8 = 0x14; // S→C unicast (reply to JoinRequest)
pub const TAG_SNAPSHOT_REQ: u8 = 0x15; // C→S
pub const TAG_SNAPSHOT_RESP: u8 = 0x16; // S→C unicast
pub const TAG_PING_REQ: u8 = 0x17; // C→S RTT probe
pub const TAG_PING_RESP: u8 = 0x18; // S→C echo

// Secure selective-lockstep V2. V1 and V2 are negotiated per match and must
// never be mixed within one player session.
pub const TAG_TEAM_GAME_START_V2: u8 = 0x20; // S→C unicast bootstrap
pub const TAG_TEAM_TICK_FRAME_V2: u8 = 0x21; // S→C team stream
pub const TAG_TEAM_REBASE_CHUNK_V2: u8 = 0x22; // S→C rebase chunk
pub const TAG_TEAM_REBASE_MANIFEST_V2: u8 = 0x23; // S→C verified manifest
pub const TAG_TEAM_REPLAY_REQUEST_V2: u8 = 0x24; // C→S
pub const TAG_CLIENT_TEAM_HASH_MISMATCH_V2: u8 = 0x25; // C→S
pub const TAG_TEAM_REBASE_ACK_V2: u8 = 0x26; // C→S
pub const TAG_SECURE_TARGET_INPUT_V2: u8 = 0x27; // C→S
pub const TAG_SECURE_TARGET_INPUT_RESULT_V2: u8 = 0x28; // S→C
pub const TAG_CLIENT_REPLICA_CHECKPOINT_V2: u8 = 0x29; // C→S
pub const TAG_SESSION_CLOSE: u8 = 0x2A; // C→S graceful session teardown
pub const TAG_SHOP_RECEIPT_QUERY: u8 = 0x2B; // C→S read-only owner result query
pub const TAG_SHOP_RECEIPT_REPLAY: u8 = 0x2C; // S→C original result, no gameplay replay
pub const TAG_OWNER_WARD_ENVELOPE: u8 = 0x2D; // S→C owner-addressed private ward state
pub const TAG_OWNER_WARD_RECOVERY_QUERY: u8 = 0x2E; // C→S read-only owner ward recovery
// Team communication. Negotiated separately from lockstep, RTT, shop, and ward.
// Unsupported peers leave these tags unread as any existing command.
pub const TAG_COMMUNICATION_HELLO: u8 = 0x2F; // C→S capability
pub const TAG_COMMUNICATION_HELLO_RESULT: u8 = 0x30; // S→C fail-closed result
pub const TAG_COMMUNICATION_SUBMIT: u8 = 0x31; // C→S bounded request
pub const TAG_COMMUNICATION_RECEIPT: u8 = 0x32; // S→C admission or refusal, not a gameplay ACK
pub const TAG_COMMUNICATION_EVENT: u8 = 0x33; // S→C one authorized recipient
pub const TAG_COMMUNICATION_REPLAY_QUERY: u8 = 0x34; // C→S one recovery read
pub const TAG_COMMUNICATION_REPLAY: u8 = 0x35; // S→C retained authorized events

