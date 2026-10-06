# Grok bounded Rust implementation proposal (no tools/writes)

Return a minimal diff for omoba-client-runtime/src/evidence.rs. No tools, builds, simulations, commits, Git mutation, credentials, engine/omfx edits or external operations. Codex independently applies and tests.

Bug: Explicit legal '--evidence-dir' is parsed by ClientRuntimeConfig even in normal mode, but EvidenceRecorder::create silently returns None unless test_mode=true. We need observe a real PIE long match WITHOUT enabling scripted/fault test behavior. Existing authority capture uses OMOBA_FOG_EVIDENCE_DIR; runtime uses CLI --evidence-dir (NOT that env).

Current exact code:
```rust
impl EvidenceRecorder {
    pub fn create(
        config: &ClientRuntimeConfig,
        global_seed: u64,
    ) -> Result<Option<Self>, ClientRuntimeError> {
        if !config.test_mode {
            return Ok(None);
        }
        let Some(base) = config.evidence_dir.as_ref() else {
            return Err(ClientRuntimeError::Config(
                "test mode requires evidence directory".into(),
            ));
        };
        let root = base.join(format!("team-{}-runtime", config.team_id));
        fs::create_dir_all(&root).map_err(io_error)?;
```
Minimal policy: explicit evidence_dir Some -> normal existing record path regardless test_mode; None+normal -> Ok(None); None+test -> original error. Do not change config defaults, scripted flags, files/format, protocol, privacy, output location, or permissive overwrites. No new dependency. Provide a small pure helper policy if required to directly test four cases WITHOUT file/network/game simulation, with a unique new cfg(test) module name 'capture_policy_tests' (existing tests must be untouched). Helper input has_directory:bool,test_mode:bool -> Result<bool,ClientRuntimeError>. create uses it as early-return gate and then as_ref.expect of validated directory; preserve original error exactly. No other architectural changes. Actual tests will be cargo test --manifest-path omoba-client-runtime/Cargo.toml --features compiled-content-only capture_policy_tests -- --test-threads=1.
