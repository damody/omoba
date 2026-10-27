use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    io::{self, Read},
    net::UdpSocket,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

const VERSION: u32 = 1;
mod workflow_supervisor;
mod owned_run;
mod elevated_run;
mod junction;
mod png_recolor;

#[cfg(windows)]
fn monotonic_ms() -> Result<Value, String> {
    // Machine uptime keeps one clock domain across separate helper processes.
    // Wall-clock corrections never extend a workflow deadline.
    let milliseconds = unsafe { windows_sys::Win32::System::SystemInformation::GetTickCount64() };
    Ok(json!({"milliseconds": milliseconds, "clock": "windows_get_tick_count64"}))
}
#[cfg(not(windows))]
fn monotonic_ms() -> Result<Value, String> {
    Err("monotonic workflow clock is not supported on this platform; no wall-clock fallback".into())
}

#[derive(Deserialize)]
struct Request {
    version: u32,
    operation: String,
    #[serde(default)]
    params: Value,
}

#[derive(Serialize)]
struct Response {
    version: u32,
    ok: bool,
    result: Value,
    error: Option<String>,
}

fn required_str<'a>(value: &'a Value, name: &str) -> Result<&'a str, String> {
    value
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing string {name}"))
}
fn required_u32(value: &Value, name: &str) -> Result<u32, String> {
    value
        .get(name)
        .and_then(Value::as_u64)
        .and_then(|v| u32::try_from(v).ok())
        .ok_or_else(|| format!("missing u32 {name}"))
}

/// Replace complete fields inside explicitly named sections, not a recursive
/// merge of authorization maps. No files or processes are touched here.
fn toml_update_sections(params:&Value) -> Result<Value,String> {
    let mut document:toml::Table=toml::from_str(required_str(params,"source")?)
        .map_err(|error|format!("invalid source TOML: {error}"))?;
    let updates=params.get("updates").and_then(Value::as_array).ok_or("missing updates array")?;
    if updates.len()>64 {return Err("too many TOML section updates".into());}
    for update in updates {
        let path=update.get("path").and_then(Value::as_array).ok_or("missing section path")?;
        if path.is_empty() || path.len()>16 {return Err("section path requires 1..16 keys".into());}
        let fields:toml::Table=toml::from_str(required_str(update,"fields")?)
            .map_err(|error|format!("invalid replacement TOML: {error}"))?;
        let mut table=&mut document;
        for part in path {
            let key=part.as_str().filter(|key|!key.is_empty()).ok_or("section path keys must be nonempty strings")?;
            table=table.entry(key.to_owned()).or_insert_with(||toml::Value::Table(toml::Table::new()))
                .as_table_mut().ok_or_else(||format!("section path collides with non-table '{key}'"))?;
        }
        table.extend(fields);
    }
    let text=toml::to_string(&document).map_err(|error|error.to_string())?;
    Ok(json!({"text":text}))
}

fn run_command(params: &Value) -> Result<Value, String> {
    let exe = required_str(params, "exe")?;
    let args = params
        .get("args")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut command = Command::new(exe);
    command.args(
        args.iter()
            .map(|v| v.as_str().ok_or("command arg must be string"))
            .collect::<Result<Vec<_>, _>>()?,
    );
    if let Some(cwd) = params.get("cwd").and_then(Value::as_str) {
        command.current_dir(cwd);
    }
    if let Some(envs) = params.get("env").and_then(Value::as_object) {
        for (key, value) in envs {
            command.env(
                key,
                value.as_str().ok_or("environment value must be string")?,
            );
        }
    }
    let output = command.output().map_err(|e| format!("run {exe}: {e}"))?;
    Ok(
        json!({"exit_code": output.status.code().unwrap_or(-1), "stdout": String::from_utf8_lossy(&output.stdout), "stderr": String::from_utf8_lossy(&output.stderr)}),
    )
}

fn spawn_command(params: &Value, exclusive: bool) -> Result<Command, String> {
    let exe = required_str(params, "exe")?;
    let args = params
        .get("args")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let stdout_path = PathBuf::from(required_str(params, "stdout")?);
    let stderr_path = PathBuf::from(required_str(params, "stderr")?);
    if let Some(parent) = stdout_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    if let Some(parent) = stderr_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut command = Command::new(exe);
    command.args(
        args.iter()
            .map(|v| v.as_str().ok_or("spawn arg must be string"))
            .collect::<Result<Vec<_>, _>>()?,
    );
    if let Some(cwd) = params.get("cwd").and_then(Value::as_str) {
        command.current_dir(cwd);
    }
    if let Some(envs) = params.get("env").and_then(Value::as_object) {
        for (key, value) in envs {
            command.env(
                key,
                value.as_str().ok_or("environment value must be string")?,
            );
        }
    }
    command
        .stdin(Stdio::null())
        .stdout(Stdio::from(
            if exclusive { fs::OpenOptions::new().write(true).create_new(true).open(&stdout_path) }
            else { fs::File::create(&stdout_path) }.map_err(|e| e.to_string())?,
        ))
        .stderr(Stdio::from(
            if exclusive { fs::OpenOptions::new().write(true).create_new(true).open(&stderr_path) }
            else { fs::File::create(&stderr_path) }.map_err(|e| e.to_string())?,
        ));
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // CREATE_NO_WINDOW. Do not add CREATE_BREAKAWAY_FROM_JOB: a supervised
        // caller must still own this child through the private job.
        command.creation_flags(0x08000000);
        // Rust Command::spawn calls CreateProcessW with bInheritHandles=TRUE, so
        // every inheritable handle is duplicated into the child, not only the
        // null stdin and log files above. Stable CommandExt::inherit_handles is
        // unavailable on 1.95. Ancestor process.run uses Command::output, and
        // os.execute leaves those pipe write ends as this helper's standard
        // handles. Clear that bit before CreateProcess so the publisher does
        // not keep them. Command marks the explicit log-file handles
        // inheritable itself. This one-shot helper does not restore the bit.
        block_standard_handle_inheritance()?;
    }
    Ok(command)
}

fn spawn(params: &Value) -> Result<Value, String> {
    let owned = params.get("owned_identity").and_then(Value::as_bool).unwrap_or(false);
    #[cfg(not(windows))]
    if owned { return Err("owned process identity is supported only on Windows".into()); }
    let exe = required_str(params, "exe")?;
    let mut child = spawn_command(params, false)?.spawn().map_err(|e| format!("spawn {exe}: {e}"))?;
    #[cfg(windows)]
    if owned {
        use std::os::windows::io::AsRawHandle;
        // The Child handle, not a reopened PID, proves this spawn's lifetime.
        let result = owned_identity_from_handle(child.as_raw_handle() as _, child.id());
        if result.is_err() { let _ = child.kill(); let _ = child.wait(); }
        return result;
    }
    Ok(json!({"pid": child.id()}))
}

#[cfg(windows)]
unsafe extern "system" {
    fn GetStdHandle(nstdhandle: u32) -> windows_sys::Win32::Foundation::HANDLE;
}

#[cfg(windows)]
fn block_standard_handle_inheritance() -> Result<(), String> {
    use windows_sys::Win32::Foundation::{
        SetHandleInformation, HANDLE_FLAG_INHERIT, INVALID_HANDLE_VALUE,
    };
    // STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE. A missing standard
    // handle is already safe. A valid handle that stays inheritable is not.
    const STANDARD_HANDLES: [(u32, &str); 3] = [
        (0xFFFF_FFF6, "stdin"),
        (0xFFFF_FFF5, "stdout"),
        (0xFFFF_FFF4, "stderr"),
    ];
    unsafe {
        for (id, name) in STANDARD_HANDLES {
            let handle = GetStdHandle(id);
            if handle.is_null() || handle == INVALID_HANDLE_VALUE {
                continue;
            }
            if SetHandleInformation(handle, HANDLE_FLAG_INHERIT, 0) == 0 {
                return Err(format!(
                    "cannot stop {name} handle inheritance: {}",
                    io::Error::last_os_error()
                ));
            }
        }
    }
    Ok(())
}

fn creation_token(value: &Value) -> Result<u64, String> {
    let text = required_str(value, "creation_token")?;
    if text.is_empty() || text.starts_with('0') || !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err("creation_token must be a canonical nonzero decimal string".into());
    }
    text.parse::<u64>().map_err(|_| "creation_token exceeds u64".into())
}

fn owned_wait_timeout(params: &Value) -> Result<u32, String> {
    let Some(value) = params.get("timeout_ms") else {
        return Ok(5000);
    };
    // WaitForSingleObject treats 0xFFFFFFFF as INFINITE. Keep every accepted bound finite.
    let timeout = value.as_u64().ok_or("invalid owned wait timeout")?;
    let timeout = u32::try_from(timeout).map_err(|_| "invalid owned wait timeout".to_string())?;
    if timeout == u32::MAX {
        return Err("invalid owned wait timeout".into());
    }
    Ok(timeout)
}

#[cfg(windows)]
struct OwnedProcessHandle(windows_sys::Win32::Foundation::HANDLE);
#[cfg(windows)]
impl Drop for OwnedProcessHandle {
    fn drop(&mut self) { unsafe { windows_sys::Win32::Foundation::CloseHandle(self.0); } }
}

#[cfg(windows)]
fn open_owned_process(pid: u32, terminate_access: bool) -> Result<Option<OwnedProcessHandle>, String> {
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_TERMINATE};
    // SYNCHRONIZE (0x00100000) stays on this exact handle for the zero-duration
    // liveness gate and the bounded retirement wait after a failed identity query.
    let rights = PROCESS_QUERY_LIMITED_INFORMATION | 0x00100000
        | if terminate_access { PROCESS_TERMINATE } else { 0 };
    let handle = unsafe { OpenProcess(rights, 0, pid) };
    if handle.is_null() {
        let error = io::Error::last_os_error();
        if error.raw_os_error() == Some(87) { return Ok(None); } // nonexistent PID
        return Err(format!("cannot open owned PID {pid}: {error}"));
    }
    Ok(Some(OwnedProcessHandle(handle)))
}

#[cfg(windows)]
const OWNED_QUERY_RETIREMENT_SETTLE_MS: u32 = 1000;

#[cfg(windows)]
fn owned_handle_alive_within(
    handle: windows_sys::Win32::Foundation::HANDLE,
    timeout_ms: u32,
) -> Result<bool, String> {
    // 0 is WAIT_OBJECT_0 (this process object is signalled). 258 is WAIT_TIMEOUT.
    // WAIT_FAILED and every other code stay errors; they are not retirement.
    match unsafe { windows_sys::Win32::System::Threading::WaitForSingleObject(handle, timeout_ms) } {
        0 => Ok(false),
        258 => Ok(true),
        _ => Err(format!("cannot query owned lifetime: {}", io::Error::last_os_error())),
    }
}

#[cfg(windows)]
fn owned_handle_alive(handle: windows_sys::Win32::Foundation::HANDLE) -> Result<bool, String> {
    owned_handle_alive_within(handle, 0)
}

#[cfg(windows)]
fn owned_identity_from_handle(handle: windows_sys::Win32::Foundation::HANDLE, pid: u32) -> Result<Value, String> {
    use windows_sys::Win32::{Foundation::FILETIME,
        System::Threading::{GetProcessTimes, QueryFullProcessImageNameW}};
    let mut buffer = vec![0u16; 32768];
    let mut length = buffer.len() as u32;
    let mut creation: FILETIME = unsafe { std::mem::zeroed() };
    let mut exit: FILETIME = unsafe { std::mem::zeroed() };
    let mut kernel: FILETIME = unsafe { std::mem::zeroed() };
    let mut user: FILETIME = unsafe { std::mem::zeroed() };
    if unsafe { QueryFullProcessImageNameW(handle, 0, buffer.as_mut_ptr(), &mut length) } == 0
        || unsafe { GetProcessTimes(handle, &mut creation, &mut exit, &mut kernel, &mut user) } == 0 {
        return Err(format!("cannot query exact process identity: {}", io::Error::last_os_error()));
    }
    let token = (u64::from(creation.dwHighDateTime) << 32) | u64::from(creation.dwLowDateTime);
    let unix_ticks = token.checked_sub(116_444_736_000_000_000).ok_or("invalid process creation time")?;
    let path = PathBuf::from(String::from_utf16_lossy(&buffer[..length as usize]));
    Ok(json!({"pid":pid,"path":path,"alive":owned_handle_alive(handle)?,
        "creation_token":token.to_string(),"created_unix_seconds":unix_ticks / 10_000_000}))
}

#[cfg(windows)]
fn inspect_owned(params: &Value) -> Result<Value, String> {
    let pid = required_u32(params, "pid")?;
    let Some(handle) = open_owned_process(pid, false)? else { return Ok(json!({"pid":pid,"alive":false})); };
    if !owned_handle_alive(handle.0)? { return Ok(json!({"pid":pid,"alive":false})); }
    Ok(live_owned_identity(handle.0,pid)?.unwrap_or_else(||json!({"pid":pid,"alive":false})))
}

// A process can finish after a successful zero-duration wait but before the
// image/time query. Access denied is not proof of retirement: only a signalled
// wait on the very same retained process object can settle that race.
// The alive probe is the caller's; this helper never opens a process or PID.
fn settle_owned_query<T>(query:Result<T,String>, alive:impl FnOnce()->Result<bool,String>) -> Result<Option<T>,String> {
    match query {
        Ok(value)=>Ok(Some(value)),
        Err(error)=>match alive() {
            Ok(false)=>Ok(None),
            Ok(true)=>Err(error),
            Err(wait_error)=>Err(format!("{error}; cannot settle lifetime: {wait_error}")),
        },
    }
}
#[cfg(test)]
mod owned_query_race_tests {
    use super::settle_owned_query;
    #[test] fn failed_query_requires_exact_handle_retirement() {
        assert_eq!(settle_owned_query::<u32>(Err("access denied".into()),||Ok(false)).unwrap(),None);
        assert_eq!(settle_owned_query::<u32>(Err("access denied".into()),||Ok(true)).unwrap_err(),"access denied");
    }
    #[test] fn failed_wait_is_not_retirement() {
        let error=settle_owned_query::<u32>(Err("access denied".into()),||Err("wait failed".into())).unwrap_err();
        assert!(error.contains("access denied") && error.contains("wait failed"));
    }
    #[test] fn successful_identity_needs_no_error_recovery() {
        assert_eq!(settle_owned_query(Ok(42),||panic!("unexpected recheck")).unwrap(),Some(42));
    }

    #[cfg(windows)]
    use std::{
        env,
        os::windows::process::CommandExt,
        process::{Command, Stdio},
        thread,
        time::{Duration, Instant},
    };
    #[cfg(windows)]
    use serde_json::{Value, json};
    #[cfg(windows)]
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    #[cfg(windows)]
    fn system_exe(name: &str) -> std::path::PathBuf {
        let root = env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".to_string());
        std::path::PathBuf::from(root).join("System32").join(name)
    }

    #[cfg(windows)]
    fn spawn_held_child() -> std::process::Child {
        Command::new(system_exe("findstr.exe"))
            .arg("x")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .expect("spawn own findstr child")
    }

    #[cfg(windows)]
    struct Reap(Option<std::process::Child>);
    #[cfg(windows)]
    impl Drop for Reap {
        fn drop(&mut self) {
            if let Some(mut child) = self.0.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }

    #[cfg(windows)]
    fn exact_handle(child: &std::process::Child) -> windows_sys::Win32::Foundation::HANDLE {
        use std::os::windows::io::AsRawHandle;
        child.as_raw_handle() as _
    }

    #[cfg(windows)]
    #[test]
    fn immediate_exact_handle_signal_settles_failed_query() {
        let mut child = Command::new(system_exe("cmd.exe"))
            .args(["/c", "exit", "0"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .expect("spawn own cmd child");
        let handle = exact_handle(&child);
        let status = child.wait().expect("wait for own cmd child");
        assert!(status.success());
        let started = Instant::now();
        let settled = settle_owned_query::<u32>(Err("access denied".into()), || {
            super::owned_handle_alive_within(handle, super::OWNED_QUERY_RETIREMENT_SETTLE_MS)
        })
        .expect("signalled exact handle");
        let elapsed = started.elapsed();
        println!("immediate_exact_handle_signal_elapsed_ms={}", elapsed.as_millis());
        assert_eq!(settled, None);
        assert!(
            elapsed < Duration::from_millis(500),
            "already-signalled handle consumed the settle budget: {elapsed:?}"
        );
    }

    #[cfg(windows)]
    #[test]
    fn delayed_exact_handle_signal_settles_failed_query() {
        let mut child = spawn_held_child();
        let handle = exact_handle(&child);
        assert!(child.try_wait().expect("try_wait").is_none(), "child exited before the wait");
        let stdin = child.stdin.take().expect("piped stdin");
        let _reap = Reap(Some(child));
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(250));
            drop(stdin);
        });
        let started = Instant::now();
        let settled = settle_owned_query::<u32>(Err("access denied".into()), || {
            super::owned_handle_alive_within(handle, super::OWNED_QUERY_RETIREMENT_SETTLE_MS)
        });
        let elapsed = started.elapsed();
        println!("delayed_exact_handle_signal_elapsed_ms={} result={settled:?}", elapsed.as_millis());
        assert_eq!(settled.expect("delayed signal"), None);
        assert!(elapsed >= Duration::from_millis(100), "wait returned before the child exit delay: {elapsed:?}");
        assert!(
            elapsed < Duration::from_millis(u64::from(super::OWNED_QUERY_RETIREMENT_SETTLE_MS) + 300),
            "delayed signal overran the settle bound: {elapsed:?}"
        );
    }

    #[cfg(windows)]
    #[test]
    fn live_exact_handle_timeout_keeps_access_denied() {
        let mut reap = Reap(Some(spawn_held_child()));
        let child = reap.0.as_mut().expect("child");
        let handle = exact_handle(child);
        assert!(child.try_wait().expect("try_wait").is_none());
        let query_error = "cannot query exact process identity: Access is denied. (os error 5)";
        let started = Instant::now();
        let error = settle_owned_query::<u32>(Err(query_error.into()), || {
            super::owned_handle_alive_within(handle, super::OWNED_QUERY_RETIREMENT_SETTLE_MS)
        })
        .expect_err("live process must stay a query failure");
        let elapsed = started.elapsed();
        println!("live_timeout_elapsed_ms={} error={error}", elapsed.as_millis());
        assert_eq!(error, query_error);
        let bound = u64::from(super::OWNED_QUERY_RETIREMENT_SETTLE_MS);
        assert!(elapsed >= Duration::from_millis(bound - 50), "timeout returned early: {elapsed:?}");
        assert!(elapsed < Duration::from_millis(bound + 750), "timeout waited too long: {elapsed:?}");
        assert!(child.try_wait().expect("try_wait").is_none(), "live rejection killed or lost the child");
    }

    #[cfg(windows)]
    #[test]
    fn exact_handle_wait_failure_is_not_retirement() {
        let started = Instant::now();
        let error = settle_owned_query::<u32>(Err("access denied".into()), || {
            super::owned_handle_alive_within(std::ptr::null_mut(), super::OWNED_QUERY_RETIREMENT_SETTLE_MS)
        })
        .expect_err("invalid handle");
        let elapsed = started.elapsed();
        println!("wait_failure_elapsed_ms={} error={error}", elapsed.as_millis());
        assert!(error.contains("access denied"), "{error}");
        assert!(error.contains("cannot settle lifetime"), "{error}");
        assert!(error.contains("cannot query owned lifetime"), "{error}");
        assert!(elapsed < Duration::from_millis(500), "WAIT_FAILED consumed the settle budget: {elapsed:?}");
    }

    #[cfg(windows)]
    #[test]
    fn successful_live_identity_does_not_wait() {
        let mut child = spawn_held_child();
        let handle = exact_handle(&child);
        let pid = child.id();
        assert!(child.try_wait().expect("try_wait").is_none());
        let started = Instant::now();
        let identity = super::live_owned_identity(handle, pid)
            .expect("live identity")
            .expect("live child must not settle as retired");
        let elapsed = started.elapsed();
        println!("successful_live_identity_elapsed_ms={} identity={identity}", elapsed.as_millis());
        assert!(elapsed < Duration::from_millis(500), "successful query entered the retirement wait: {elapsed:?}");
        assert_eq!(identity["pid"].as_u64(), Some(u64::from(pid)));
        assert_eq!(identity["alive"].as_bool(), Some(true));
        let token = identity["creation_token"].as_str().expect("creation token");
        assert!(!token.is_empty() && !token.starts_with('0') && token.bytes().all(|byte| byte.is_ascii_digit()));
        let path = identity["path"].as_str().expect("path").to_ascii_lowercase();
        assert!(path.contains("findstr"), "{path}");
        let query = super::owned_identity_from_handle(handle, pid);
        assert!(query.is_ok(), "second exact-handle query failed: {query:?}");
        let settled = settle_owned_query(query, || panic!("successful query must not wait")).expect("settle");
        assert!(settled.is_some());
        drop(child.stdin.take());
        let _ = child.wait();
        let _ = Reap(Some(child));
    }

    #[cfg(windows)]
    #[test]
    fn exited_own_child_exact_handle_settles_retirement() {
        let mut child = Command::new(system_exe("cmd.exe"))
            .args(["/c", "exit", "0"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .expect("spawn own cmd child");
        let handle = exact_handle(&child);
        let pid = child.id();
        let status = child.wait().expect("wait for own cmd child");
        assert!(status.success());
        let started = Instant::now();
        let settled = super::live_owned_identity(handle, pid);
        let elapsed = started.elapsed();
        println!("exited_own_child_elapsed_ms={} settled={settled:?}", elapsed.as_millis());
        match settled {
            Ok(None) => {}
            Ok(Some(identity)) => {
                assert_eq!(identity["pid"].as_u64(), Some(u64::from(pid)));
                assert_eq!(identity["alive"].as_bool(), Some(false), "exited child stayed alive: {identity}");
            }
            Err(error) => panic!("natural exit surfaced a query failure: {error}"),
        }
        assert!(
            elapsed < Duration::from_millis(500),
            "already-exited exact handle consumed the settle budget: {elapsed:?}"
        );
    }

    #[cfg(windows)]
    fn owned_record(child: &std::process::Child) -> Value {
        let identity = super::live_owned_identity(exact_handle(child), child.id())
            .expect("identity")
            .expect("live child");
        json!({
            "pid": child.id(),
            "expected_exe": identity["path"].as_str().expect("path"),
            "creation_token": identity["creation_token"].as_str().expect("token"),
            "timeout_ms": 5000,
        })
    }

    #[cfg(windows)]
    #[test]
    fn verified_child_exit_waits_on_the_same_handle() {
        let mut child = spawn_held_child();
        let params = owned_record(&child);
        assert!(child.try_wait().expect("try_wait").is_none());
        let stdin = child.stdin.take().expect("piped stdin");
        let _reap = Reap(Some(child));
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(200));
            drop(stdin);
        });
        let started = Instant::now();
        let result = super::wait_owned(&params).expect("owned wait");
        let elapsed = started.elapsed();
        println!("verified_child_exit_elapsed_ms={} result={result}", elapsed.as_millis());
        assert_eq!(result["exited"].as_bool(), Some(true));
        assert!(elapsed >= Duration::from_millis(100), "wait returned before the child exit: {elapsed:?}");
        assert!(elapsed < Duration::from_millis(5000), "exit wait used more than the caller bound: {elapsed:?}");
    }

    #[cfg(windows)]
    #[test]
    fn verified_child_timeout_is_not_retirement() {
        let mut reap = Reap(Some(spawn_held_child()));
        let child = reap.0.as_mut().expect("child");
        let mut params = owned_record(child);
        params["timeout_ms"] = json!(200);
        assert!(child.try_wait().expect("try_wait").is_none());
        let started = Instant::now();
        let result = super::wait_owned(&params).expect("owned wait");
        let elapsed = started.elapsed();
        println!("verified_child_timeout_elapsed_ms={} result={result}", elapsed.as_millis());
        assert_eq!(result["exited"].as_bool(), Some(false));
        assert!(elapsed >= Duration::from_millis(150), "timeout returned early: {elapsed:?}");
        assert!(elapsed < Duration::from_millis(900), "timeout waited too long: {elapsed:?}");
        assert!(child.try_wait().expect("try_wait").is_none(), "timeout killed or lost the child");
    }

    #[cfg(windows)]
    #[test]
    fn creation_token_mismatch_is_not_retirement() {
        let mut reap = Reap(Some(spawn_held_child()));
        let child = reap.0.as_mut().expect("child");
        let mut params = owned_record(child);
        params["creation_token"] = json!("1");
        params["timeout_ms"] = json!(5000);
        let started = Instant::now();
        let error = super::wait_owned(&params).expect_err("token mismatch");
        let elapsed = started.elapsed();
        println!("token_mismatch_elapsed_ms={} error={error}", elapsed.as_millis());
        assert!(error.contains("mismatch"), "{error}");
        assert!(elapsed < Duration::from_millis(500), "mismatch consumed the caller wait: {elapsed:?}");
        assert!(child.try_wait().expect("try_wait").is_none(), "mismatch killed or lost the child");
    }

    #[cfg(windows)]
    #[test]
    fn executable_mismatch_is_not_retirement() {
        let mut reap = Reap(Some(spawn_held_child()));
        let child = reap.0.as_mut().expect("child");
        let mut params = owned_record(child);
        params["expected_exe"] = json!(system_exe("cmd.exe"));
        params["timeout_ms"] = json!(5000);
        let started = Instant::now();
        let error = super::wait_owned(&params).expect_err("executable mismatch");
        let elapsed = started.elapsed();
        println!("exe_mismatch_elapsed_ms={} error={error}", elapsed.as_millis());
        assert!(error.contains("mismatch"), "{error}");
        assert!(elapsed < Duration::from_millis(500), "mismatch consumed the caller wait: {elapsed:?}");
        assert!(child.try_wait().expect("try_wait").is_none(), "mismatch killed or lost the child");
    }

    #[cfg(windows)]
    #[test]
    fn already_signalled_or_missing_pid_is_retired_without_a_requery_loop() {
        let mut child = spawn_held_child();
        let params = owned_record(&child);
        drop(child.stdin.take());
        child.wait().expect("wait for own findstr child");
        let started = Instant::now();
        let signalled = super::wait_owned(&params).expect("signalled owned wait");
        let signalled_elapsed = started.elapsed();
        println!("already_signalled_elapsed_ms={} result={signalled}", signalled_elapsed.as_millis());
        assert_eq!(signalled["exited"].as_bool(), Some(true));
        assert!(
            signalled_elapsed < Duration::from_millis(500),
            "signalled handle consumed the settle or caller budget: {signalled_elapsed:?}"
        );
        let pid = params["pid"].as_u64().expect("pid");
        drop(child);
        let started = Instant::now();
        let gone = super::wait_owned(&params);
        let gone_elapsed = started.elapsed();
        println!("pid_after_drop_elapsed_ms={} result={gone:?}", gone_elapsed.as_millis());
        match gone {
            Ok(value) => assert_eq!(value["exited"].as_bool(), Some(true), "pid {pid} stayed alive: {value}"),
            Err(error) => assert!(error.contains("mismatch"), "pid {pid} was adopted: {error}"),
        }
        assert!(gone_elapsed < Duration::from_millis(500), "missing pid consumed the caller wait: {gone_elapsed:?}");
    }
}
#[cfg(windows)]
fn live_owned_identity(handle:windows_sys::Win32::Foundation::HANDLE,pid:u32)->Result<Option<Value>,String> {
    // Only a failed identity query may block, and only on this same handle.
    settle_owned_query(owned_identity_from_handle(handle, pid), || {
        owned_handle_alive_within(handle, OWNED_QUERY_RETIREMENT_SETTLE_MS)
    })
}

#[cfg(windows)]
fn stop_owned(params: &Value) -> Result<Value, String> {
    let pid = required_u32(params, "pid")?;
    let token = creation_token(params)?;
    let expected = fs::canonicalize(required_str(params, "expected_exe")?).map_err(|e| e.to_string())?;
    let Some(handle) = open_owned_process(pid, true)? else { return Ok(json!({"pid":pid,"stopped":false})); };
    if !owned_handle_alive(handle.0)? { return Ok(json!({"pid":pid,"stopped":false})); }
    let Some(identity) = live_owned_identity(handle.0,pid)? else { return Ok(json!({"pid":pid,"stopped":false})); };
    let actual = fs::canonicalize(required_str(&identity, "path")?).map_err(|e| e.to_string())?;
    if actual != expected || creation_token(&identity)? != token {
        return Err("owned process lifetime or executable mismatch; nothing stopped".into());
    }
    // Keep the verified handle alive across verification, termination and wait.
    if unsafe { windows_sys::Win32::System::Threading::TerminateProcess(handle.0, 1) } == 0 {
        let error=io::Error::last_os_error();
        if !owned_handle_alive(handle.0)? { return Ok(json!({"pid":pid,"stopped":false})); }
        return Err(format!("owned termination failed: {error}"));
    }
    if unsafe { windows_sys::Win32::System::Threading::WaitForSingleObject(handle.0, 15000) } != 0 {
        return Err("owned process did not terminate within 15000ms".into());
    }
    Ok(json!({"pid":pid,"stopped":true}))
}

#[cfg(not(windows))]
fn inspect_owned(_: &Value) -> Result<Value, String> { Err("owned process identity is supported only on Windows".into()) }
#[cfg(not(windows))]
fn stop_owned(_: &Value) -> Result<Value, String> { Err("owned process identity is supported only on Windows".into()) }

#[cfg(windows)]
fn close_window_owned(params: &Value) -> Result<Value, String> {
    let pid = required_u32(params, "pid")?;
    let token = creation_token(params)?;
    let expected = fs::canonicalize(required_str(params, "expected_exe")?).map_err(|e| e.to_string())?;
    let handle = open_owned_process(pid, false)?.ok_or("owned window process is not alive")?;
    if !owned_handle_alive(handle.0)? { return Err("owned window process is not alive".into()); }
    let identity = live_owned_identity(handle.0,pid)?.ok_or("owned window process already retired")?;
    let actual = fs::canonicalize(required_str(&identity, "path")?).map_err(|e| e.to_string())?;
    if actual != expected || creation_token(&identity)? != token {
        return Err("owned process lifetime or executable mismatch; no window closed".into());
    }
    // Retain the process object across enumeration: no PID-only reopen or kill.
    let result = close_window(params);
    drop(handle);
    result
}
#[cfg(not(windows))]
fn close_window_owned(_: &Value) -> Result<Value, String> { Err("owned window close is supported only on Windows".into()) }

#[cfg(windows)]
fn wait_owned(params: &Value) -> Result<Value, String> {
    let pid = required_u32(params, "pid")?;
    let token = creation_token(params)?;
    let expected = fs::canonicalize(required_str(params, "expected_exe")?).map_err(|e| e.to_string())?;
    let timeout_ms = owned_wait_timeout(params)?;
    // ERROR_INVALID_PARAMETER is the nonexistent PID. A later process is not adopted.
    let Some(handle) = open_owned_process(pid, false)? else {
        return Ok(json!({"exited": true}));
    };
    // The original object is already signalled. Do not query its image during exit.
    if !owned_handle_alive(handle.0)? {
        return Ok(json!({"exited": true}));
    }
    let Some(identity) = live_owned_identity(handle.0, pid)? else {
        return Ok(json!({"exited": true}));
    };
    let actual = fs::canonicalize(required_str(&identity, "path")?).map_err(|e| e.to_string())?;
    if actual != expected || creation_token(&identity)? != token {
        return Err("owned process lifetime or executable mismatch; nothing waited".into());
    }
    // This is the same verified handle. Exit does not reopen the PID or reread the image.
    let alive = owned_handle_alive_within(handle.0, timeout_ms)?;
    Ok(json!({"exited": !alive}))
}
#[cfg(not(windows))]
fn wait_owned(_: &Value) -> Result<Value, String> { Err("owned process wait is supported only on Windows".into()) }

#[cfg(windows)]
fn process_path(pid: u32) -> Result<PathBuf, String> {
    use windows_sys::Win32::{
        Foundation::CloseHandle,
        System::Threading::{
            OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
        },
    };
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return Err(format!(
                "cannot open PID {pid}: {}",
                io::Error::last_os_error()
            ));
        }
        let mut buffer = vec![0u16; 32768];
        let mut length = buffer.len() as u32;
        let ok = QueryFullProcessImageNameW(handle, 0, buffer.as_mut_ptr(), &mut length);
        CloseHandle(handle);
        if ok == 0 {
            return Err(format!(
                "cannot query PID {pid}: {}",
                io::Error::last_os_error()
            ));
        }
        Ok(PathBuf::from(String::from_utf16_lossy(
            &buffer[..length as usize],
        )))
    }
}
#[cfg(not(windows))]
fn process_path(pid: u32) -> Result<PathBuf, String> {
    fs::read_link(format!("/proc/{pid}/exe")).map_err(|e| e.to_string())
}

fn inspect(params: &Value) -> Result<Value, String> {
    let pid = required_u32(params, "pid")?;
    let path = process_path(pid)?;
    let mut result = json!({"pid":pid,"path":path,"alive":true});
    #[cfg(windows)]
    {
        use windows_sys::Win32::{Foundation::{CloseHandle, FILETIME}, System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, GetProcessTimes}};
        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if handle.is_null() { return Err("process disappeared during identity query".into()); }
            let mut creation: FILETIME = std::mem::zeroed();
            let mut exit: FILETIME = std::mem::zeroed();
            let mut kernel: FILETIME = std::mem::zeroed();
            let mut user: FILETIME = std::mem::zeroed();
            let ok = GetProcessTimes(handle, &mut creation, &mut exit, &mut kernel, &mut user);
            CloseHandle(handle);
            if ok == 0 { return Err("process creation time unavailable".into()); }
            let ticks = (u64::from(creation.dwHighDateTime) << 32) | u64::from(creation.dwLowDateTime);
            let unix_ticks = ticks.checked_sub(116_444_736_000_000_000).ok_or("invalid process creation time")?;
            result["created_unix_seconds"] = json!(unix_ticks / 10_000_000);
        }
    }
    Ok(result)
}

// Identify the actual IPv4 loopback listener before any credentialed MCP call.
#[cfg(windows)]
fn tcp_listener(params: &Value) -> Result<Value, String> {
    use windows_sys::Win32::NetworkManagement::IpHelper::{
        GetExtendedTcpTable, MIB_TCPROW_OWNER_PID, TCP_TABLE_OWNER_PID_LISTENER,
    };
    let port = u16::try_from(required_u32(params, "port")?).map_err(|_| "invalid TCP port")?;
    if port == 0 { return Err("invalid TCP port".into()); }
    let mut size = 0u32;
    unsafe { GetExtendedTcpTable(std::ptr::null_mut(), &mut size, 0, 2, TCP_TABLE_OWNER_PID_LISTENER, 0); }
    for _ in 0..3 {
        if size < 4 || size > 16 * 1024 * 1024 { return Err("invalid TCP table size".into()); }
        // u32 storage supplies the alignment required by Windows table structs.
        let mut storage = vec![0u32; (size as usize + 3) / 4];
        let capacity = storage.len() * 4;
        let status = unsafe { GetExtendedTcpTable(storage.as_mut_ptr().cast(), &mut size, 0, 2, TCP_TABLE_OWNER_PID_LISTENER, 0) };
        if status == 122 { continue; } // ERROR_INSUFFICIENT_BUFFER: bounded resize.
        if status != 0 { return Err(format!("TCP table query failed: {status}")); }
        let count = storage[0] as usize;
        let row_size = std::mem::size_of::<MIB_TCPROW_OWNER_PID>();
        if count > (capacity - 4) / row_size { return Err("invalid TCP table row count".into()); }
        let mut owners = std::collections::BTreeSet::new();
        for index in 0..count {
            let row = unsafe { std::ptr::read_unaligned(storage.as_ptr().cast::<u8>().add(4 + index * row_size).cast::<MIB_TCPROW_OWNER_PID>()) };
            // Windows represents the IPv4 address and port in network byte order.
            if u16::from_be(row.dwLocalPort as u16) == port &&
                (row.dwLocalAddr == 0 || row.dwLocalAddr.to_ne_bytes() == [127, 0, 0, 1]) {
                owners.insert(row.dwOwningPid);
            }
        }
        if owners.len() != 1 { return Err(format!("loopback port {port} requires exactly one listener owner, found {}", owners.len())); }
        let pid = *owners.iter().next().unwrap();
        return inspect(&json!({"pid":pid}));
    }
    Err("TCP listener table changed during bounded query".into())
}

#[cfg(not(windows))]
fn tcp_listener(_: &Value) -> Result<Value, String> {
    Err("TCP listener identity is supported only on Windows".into())
}

#[cfg(windows)]
fn terminate(pid: u32, exit_code: u32) -> Result<(), String> {
    use windows_sys::Win32::{
        Foundation::CloseHandle,
        System::Threading::{OpenProcess, PROCESS_TERMINATE, TerminateProcess},
    };
    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
        if handle.is_null() {
            return Err(io::Error::last_os_error().to_string());
        }
        let ok = TerminateProcess(handle, exit_code);
        CloseHandle(handle);
        if ok == 0 {
            return Err(io::Error::last_os_error().to_string());
        }
        Ok(())
    }
}
#[cfg(not(windows))]
fn terminate(pid: u32, _: u32) -> Result<(), String> {
    Command::new("kill")
        .args(["-TERM", &pid.to_string()])
        .status()
        .map_err(|e| e.to_string())
        .and_then(|s| {
            if s.success() {
                Ok(())
            } else {
                Err("kill failed".into())
            }
        })
}

fn stop(params: &Value) -> Result<Value, String> {
    let pid = required_u32(params, "pid")?;
    let expected =
        fs::canonicalize(required_str(params, "expected_exe")?).map_err(|e| e.to_string())?;
    let actual = fs::canonicalize(process_path(pid)?).map_err(|e| e.to_string())?;
    if actual != expected {
        return Err(format!(
            "PID executable mismatch: {} != {}",
            actual.display(),
            expected.display()
        ));
    }
    terminate(pid, 1)?;
    Ok(json!({"pid":pid,"stopped":true}))
}
fn udp(params: &Value) -> Result<Value, String> {
    let addr = required_str(params, "addr")?;
    let payload = required_str(params, "payload")?.as_bytes();
    let socket = UdpSocket::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let sent = socket.send_to(payload, addr).map_err(|e| e.to_string())?;
    Ok(json!({"sent":sent}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::UdpSocket;

    #[test]
    fn owned_creation_token_preserves_exact_filetime_without_float() {
        assert_eq!(creation_token(&json!({"creation_token":"133000000000000001"})).unwrap(),133000000000000001);
        assert_eq!(creation_token(&json!({"creation_token":u64::MAX.to_string()})).unwrap(),u64::MAX);
        for value in [json!(null),json!(1),json!("0"),json!("01"),json!("-1"),json!("1.0"),json!(" 1"),json!("18446744073709551616")] {
            assert!(creation_token(&json!({"creation_token":value})).is_err());
        }
    }

    #[cfg(windows)]
    #[test]
    fn monotonic_workflow_clock_uses_uptime_and_advances_across_wait() {
        let first=monotonic_ms().unwrap();
        thread::sleep(Duration::from_millis(40));
        let second=monotonic_ms().unwrap();
        assert_eq!(first["clock"],"windows_get_tick_count64");
        assert!(second["milliseconds"].as_u64().unwrap()>first["milliseconds"].as_u64().unwrap());
    }

    #[test]
    fn toml_sections_preserve_types_and_replace_authorization_tables() {
        let source=r#"title='unchanged'
[server]
AUTHENTICATED_TEAM_BINDINGS={"1"=1,"2"=2}
quoted="a # value"
array=[1,2,3]
[other]
multiline="""first
second"""
"#;
        let result=toml_update_sections(&json!({"source":source,"updates":[{"path":["server"],
            "fields":"AUTHENTICATED_TEAM_BINDINGS={\"7\"=2}\nSTEP_FPS=60\n"}]})).unwrap();
        let parsed:toml::Value=toml::from_str(result["text"].as_str().unwrap()).unwrap();
        assert_eq!(parsed["title"].as_str(),Some("unchanged"));
        assert_eq!(parsed["server"]["quoted"].as_str(),Some("a # value"));
        assert_eq!(parsed["server"]["array"].as_array().unwrap().len(),3);
        assert_eq!(parsed["other"]["multiline"].as_str(),Some("first\nsecond"));
        let auth=parsed["server"]["AUTHENTICATED_TEAM_BINDINGS"].as_table().unwrap();
        assert_eq!(auth.len(),1);assert_eq!(auth["7"].as_integer(),Some(2));
    }
    #[test]
    fn toml_sections_reject_malformed_updates_and_path_collisions() {
        for params in [json!({"source":"a=1\na=2","updates":[]}),
            json!({"source":"server=1","updates":[{"path":["server"],"fields":"a=1"}]}),
            json!({"source":"","updates":[{"path":[],"fields":"a=1"}]}),
            json!({"source":"","updates":[{"path":["server"],"fields":"a=1\na=2"}]})] {
            assert!(toml_update_sections(&params).is_err());
        }
    }

    #[test]
    fn command_protocol_reports_exit_and_output() {
        let (exe, args) = if cfg!(windows) {
            ("cmd.exe", json!(["/d", "/c", "echo lua-host"]))
        } else {
            ("sh", json!(["-c", "printf lua-host"]))
        };
        let result = run_command(&json!({"exe": exe, "args": args})).unwrap();
        assert_eq!(result["exit_code"], 0);
        assert!(result["stdout"].as_str().unwrap().contains("lua-host"));
    }

    #[test]
    fn inspect_reports_current_executable_identity() {
        let result = inspect(&json!({"pid": std::process::id()})).unwrap();
        let actual = fs::canonicalize(result["path"].as_str().unwrap()).unwrap();
        let expected = fs::canonicalize(env::current_exe().unwrap()).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn udp_send_reaches_loopback_receiver() {
        let receiver = UdpSocket::bind("127.0.0.1:0").unwrap();
        receiver.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        let address = receiver.local_addr().unwrap().to_string();
        let result = udp(&json!({"addr": address, "payload": "hello"})).unwrap();
        assert_eq!(result["sent"], 5);
        let mut buffer = [0_u8; 16];
        let (size, _) = receiver.recv_from(&mut buffer).unwrap();
        assert_eq!(&buffer[..size], b"hello");
    }

    #[cfg(windows)]
    #[test]
    fn tcp_listener_proves_owner_and_rejects_closed_port() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let owner = tcp_listener(&json!({"port":port})).unwrap();
        assert_eq!(owner["pid"], std::process::id());
        assert!(owner["created_unix_seconds"].as_u64().unwrap() > 0);
        drop(listener);
        assert!(tcp_listener(&json!({"port":port})).is_err());
        assert!(tcp_listener(&json!({"port":0})).is_err());
        assert!(tcp_listener(&json!({"port":65536})).is_err());
    }
}
fn sha256(params: &Value) -> Result<Value, String> {
    let file = required_str(params, "path")?;
    let mut input = fs::File::open(file).map_err(|e| e.to_string())?;
    let mut hash = Sha256::new();
    io::copy(&mut input, &mut hash).map_err(|e| e.to_string())?;
    Ok(json!({"sha256":format!("{:x}",hash.finalize())}))
}

fn wait_for(params: &Value) -> Result<Value, String> {
    let pid = required_u32(params, "pid")?;
    let timeout = params
        .get("timeout_ms")
        .and_then(Value::as_u64)
        .unwrap_or(5000);
    let start = Instant::now();
    while start.elapsed() < Duration::from_millis(timeout) {
        if process_path(pid).is_err() {
            return Ok(json!({"exited":true}));
        }
        thread::sleep(Duration::from_millis(50));
    }
    Ok(json!({"exited":false}))
}

fn sleep(params: &Value) -> Result<Value, String> {
    let milliseconds = params
        .get("milliseconds")
        .and_then(Value::as_u64)
        .ok_or_else(|| "missing milliseconds".to_string())?;
    thread::sleep(Duration::from_millis(milliseconds));
    Ok(json!({"slept_ms":milliseconds}))
}

#[cfg(windows)]
fn memory_dump(params: &Value) -> Result<Value, String> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::{
        Foundation::CloseHandle,
        System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
    };
    #[link(name = "Dbghelp")]
    unsafe extern "system" {
        fn MiniDumpWriteDump(
            process: *mut std::ffi::c_void,
            pid: u32,
            file: *mut std::ffi::c_void,
            dump_type: u32,
            exception: *const std::ffi::c_void,
            user: *const std::ffi::c_void,
            callback: *const std::ffi::c_void,
        ) -> i32;
    }
    let pid = required_u32(params, "pid")?;
    let output = required_str(params, "output")?;
    let expected =
        fs::canonicalize(required_str(params, "expected_exe")?).map_err(|e| e.to_string())?;
    let actual = fs::canonicalize(process_path(pid)?).map_err(|e| e.to_string())?;
    if actual != expected {
        return Err("PID executable mismatch".into());
    }
    if let Some(parent) = Path::new(output).parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?
    }
    let file = fs::File::create(output).map_err(|e| e.to_string())?;
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, pid);
        if process.is_null() {
            return Err(io::Error::last_os_error().to_string());
        }
        let ok = MiniDumpWriteDump(
            process,
            pid,
            file.as_raw_handle() as _,
            2,
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
        );
        CloseHandle(process);
        if ok == 0 {
            return Err(io::Error::last_os_error().to_string());
        }
    }
    Ok(json!({"pid":pid,"path":output}))
}
#[cfg(not(windows))]
fn memory_dump(params: &Value) -> Result<Value, String> {
    let pid = required_u32(params, "pid")?;
    let output = required_str(params, "output")?;
    let status = Command::new("gcore")
        .args(["-o", output, &pid.to_string()])
        .status()
        .map_err(|e| e.to_string())?;
    if !status.success() {
        return Err("gcore failed".into());
    }
    Ok(json!({"pid":pid,"path":output}))
}

#[cfg(windows)]
fn screenshot(params: &Value) -> Result<Value, String> {
    use windows_sys::Win32::{
        Foundation::{HWND, LPARAM, RECT},
        Graphics::Gdi::{
            BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleBitmap, CreateCompatibleDC,
            DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDIBits, GetWindowDC, ReleaseDC,
            SelectObject,
        },
        UI::WindowsAndMessaging::{
            EnumWindows, GetWindowRect, GetWindowThreadProcessId, IsWindowVisible,
        },
    };
    #[link(name = "User32")]
    unsafe extern "system" {
        fn PrintWindow(hwnd: HWND, hdc: *mut std::ffi::c_void, flags: u32) -> i32;
    }
    #[repr(C)]
    struct Search {
        pid: u32,
        hwnd: HWND,
    }
    unsafe extern "system" fn callback(hwnd: HWND, param: LPARAM) -> i32 {
        unsafe {
            let search = &mut *(param as *mut Search);
            let mut pid = 0;
            GetWindowThreadProcessId(hwnd, &mut pid);
            if pid == search.pid && IsWindowVisible(hwnd) != 0 {
                search.hwnd = hwnd;
                return 0;
            }
        }
        1
    }
    let pid = required_u32(params, "pid")?;
    let output = required_str(params, "output")?;
    let mut search = Search {
        pid,
        hwnd: std::ptr::null_mut(),
    };
    unsafe {
        let _ = EnumWindows(Some(callback), &mut search as *mut _ as LPARAM);
    };
    if search.hwnd.is_null() {
        return Err(format!("PID {pid} has no visible window"));
    }
    let mut rect = RECT::default();
    unsafe {
        if GetWindowRect(search.hwnd, &mut rect) == 0 {
            return Err(io::Error::last_os_error().to_string());
        }
    }
    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;
    if width <= 0 || height <= 0 {
        return Err("invalid window bounds".into());
    }
    let mut pixels = vec![0u8; (width * height * 4) as usize];
    unsafe {
        let window_dc = GetWindowDC(search.hwnd);
        let memory_dc = CreateCompatibleDC(window_dc);
        let bitmap = CreateCompatibleBitmap(window_dc, width, height);
        let old = SelectObject(memory_dc, bitmap);
        if PrintWindow(search.hwnd, memory_dc, 2) == 0 {
            return Err(io::Error::last_os_error().to_string());
        }
        let mut info = BITMAPINFO::default();
        info.bmiHeader = BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB,
            ..Default::default()
        };
        if GetDIBits(
            memory_dc,
            bitmap,
            0,
            height as u32,
            pixels.as_mut_ptr() as _,
            &mut info,
            DIB_RGB_COLORS,
        ) == 0
        {
            return Err(io::Error::last_os_error().to_string());
        }
        SelectObject(memory_dc, old);
        DeleteObject(bitmap);
        DeleteDC(memory_dc);
        let _ = ReleaseDC(search.hwnd, window_dc);
    }
    for p in pixels.chunks_exact_mut(4) {
        p.swap(0, 2)
    }
    if let Some(parent) = Path::new(output).parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?
    }
    let file = fs::File::create(output).map_err(|e| e.to_string())?;
    let mut encoder = png::Encoder::new(file, width as u32, height as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .map_err(|e| e.to_string())?
        .write_image_data(&pixels)
        .map_err(|e| e.to_string())?;
    Ok(json!({"pid":pid,"path":output,"width":width,"height":height}))
}
#[cfg(not(windows))]
fn screenshot(_: &Value) -> Result<Value, String> {
    Err("window screenshot is only supported on Windows".into())
}

#[cfg(windows)]
fn close_window(params: &Value) -> Result<Value, String> {
    use windows_sys::Win32::{
        Foundation::{HWND, LPARAM},
        UI::WindowsAndMessaging::{EnumWindows, GetWindowThreadProcessId, PostMessageW, WM_CLOSE},
    };
    #[repr(C)]
    struct Search {
        pid: u32,
        count: u32,
    }
    unsafe extern "system" fn callback(hwnd: HWND, param: LPARAM) -> i32 {
        unsafe {
            let s = &mut *(param as *mut Search);
            let mut pid = 0;
            GetWindowThreadProcessId(hwnd, &mut pid);
            if pid == s.pid && PostMessageW(hwnd, WM_CLOSE, 0, 0) != 0 {
                s.count += 1
            }
        }
        1
    }
    let mut search = Search {
        pid: required_u32(params, "pid")?,
        count: 0,
    };
    unsafe {
        let _ = EnumWindows(Some(callback), &mut search as *mut _ as LPARAM);
    };
    Ok(json!({"posted":search.count}))
}
#[cfg(not(windows))]
fn close_window(params: &Value) -> Result<Value, String> {
    let pid = required_u32(params, "pid")?;
    let status = Command::new("kill")
        .args(["-TERM", &pid.to_string()])
        .status()
        .map_err(|e| e.to_string())?;
    Ok(json!({"posted":if status.success(){1}else{0}}))
}

fn dispatch(request: &Request) -> Result<Value, String> {
    match request.operation.as_str() {
        "workflow_supervise" => workflow_supervisor::supervise(&request.params),
        "workflow_member" => workflow_supervisor::member(&request.params),
        "run" => run_command(&request.params),
        "run_elevated" => elevated_run::run(&request.params),
        "spawn" => spawn(&request.params),
        "run_owned_to_files" => owned_run::run(&request.params),
        "inspect" => inspect(&request.params),
        "inspect_owned" => inspect_owned(&request.params),
        "tcp_listener" => tcp_listener(&request.params),
        "cpu_capacity" => Ok(json!({"logical_cpus": std::thread::available_parallelism()
            .map_err(|e| format!("cannot query CPU capacity: {e}"))?.get()})),
        "monotonic_ms" => monotonic_ms(),
        "stop" => stop(&request.params),
        "stop_owned" => stop_owned(&request.params),
        "close_window" => close_window(&request.params),
        "close_window_owned" => close_window_owned(&request.params),
        "udp_send" => udp(&request.params),
        "sha256" => sha256(&request.params),
        "junction_create" => junction::create(&request.params),
        "junction_inspect" => junction::inspect(&request.params),
        "junction_remove" => junction::remove(&request.params),
        "wait" => wait_for(&request.params),
        "wait_owned" => wait_owned(&request.params),
        "sleep" => sleep(&request.params),
        "memory_dump" => memory_dump(&request.params),
        "screenshot" => screenshot(&request.params),
        "toml_update_sections" => toml_update_sections(&request.params),
        "png_recolor" => png_recolor::run(&request.params),
        other => Err(format!("unknown operation {other}")),
    }
}

fn main() {
    let response = (|| -> Result<Value, String> {
        let mut text = String::new();
        if let Some(file) = env::args_os().nth(1) {
            text = fs::read_to_string(file).map_err(|e| e.to_string())?
        } else {
            io::stdin()
                .read_to_string(&mut text)
                .map_err(|e| e.to_string())?;
        };
        let request: Request = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        if request.version != VERSION {
            return Err(format!("unsupported request version {}", request.version));
        }
        dispatch(&request)
    })();
    let output = match response {
        Ok(result) => Response {
            version: VERSION,
            ok: true,
            result,
            error: None,
        },
        Err(error) => Response {
            version: VERSION,
            ok: false,
            result: Value::Null,
            error: Some(error),
        },
    };
    let serialized = serde_json::to_string(&output).expect("response serialization");
    if let Some(response_file) = env::args_os().nth(2) {
        fs::write(response_file, serialized).expect("write response file");
    } else {
        println!("{serialized}");
    }
    if !output.ok {
        std::process::exit(2)
    }
}
