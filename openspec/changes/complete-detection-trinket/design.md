# Design

Append catalog ID2; keep ID1 values unchanged. Detection lantern: visible body, sight600/detection2, lifetime60, one charge, recharge180, HP4, maximum one owned placement. Existing LOS and team rules apply; this is original balance, not exact LoL control-ward parity.

Extend generated native FOmTrinketIdentity with description and readable gameplay distances/lifetime (not C ABI or wire structs). A shared ward-tooltip detail builder consumes these fields. Loadout remains an authority configuration through the existing API, not a client-declared placement catalog ID. Tests use real admitted local ward commands and the production60Hz driver.

OmGenerated depends on OmRuntime, so the generated bridge copies this metadata into the existing native-only FOmOwnerWardHudView. Runtime's shared formatter consumes that view and rejects unknown/unequipped or malformed dimensions; it never includes an upper-layer generated catalog. Native acceptance includes the existing texture-import identity regression with an actually unshipped ID65535.
