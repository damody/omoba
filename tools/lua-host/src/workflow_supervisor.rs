//! Opt-in workflow lifetime; never changes the legacy spawn operations.
use serde_json::Value;

#[cfg(not(windows))]
pub fn supervise(_: &Value) -> Result<Value, String> {
    Err("workflow supervision requires Windows".into())
}
#[cfg(not(windows))]
pub fn member(_: &Value) -> Result<Value, String> {
    Err("workflow supervision requires Windows".into())
}

// CommandLineToArgvW / CRT quoting, not shell quoting. Empty and trailing-slash
// arguments must survive exactly. No shell interprets this command line.
fn quote(value: &str) -> Result<String, String> {
    if value.contains('\0') {
        return Err("NUL in workflow argument".into());
    }
    let mut out = String::from("\"");
    let mut slashes = 0;
    for ch in value.chars() {
        if ch == '\\' {
            slashes += 1;
            continue;
        }
        out.extend(std::iter::repeat_n(
            '\\',
            if ch == '"' { slashes * 2 + 1 } else { slashes },
        ));
        slashes = 0;
        out.push(ch);
    }
    out.extend(std::iter::repeat_n('\\', slashes * 2));
    out.push('"');
    Ok(out)
}

#[cfg(windows)]
mod windows {
    use super::*;
    use serde_json::json;
    use std::{
        env,
        ffi::OsStr,
        mem::{size_of, zeroed},
        os::windows::ffi::OsStrExt,
        path::{Path, PathBuf},
        ptr::{null, null_mut},
        thread,
        time::{Duration, Instant},
    };
    use windows_sys::Win32::System::SystemServices::JOB_OBJECT_QUERY;
    use windows_sys::Win32::{
        Foundation::*,
        System::{Diagnostics::ToolHelp::*, JobObjects::*, Threading::*},
    };
    const MARKER: &str = "_OMOBA_PRIVATE_WORKFLOW_JOB";
    const PREFIX: &str = "Local\\omoba-workflow-";
    struct Handle(HANDLE);
    impl Drop for Handle {
        fn drop(&mut self) {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
    fn handle(value: HANDLE, operation: &str) -> Result<Handle, String> {
        if value.is_null() || value == INVALID_HANDLE_VALUE {
            Err(format!("{operation}: {}", std::io::Error::last_os_error()))
        } else {
            Ok(Handle(value))
        }
    }
    fn wide(value: impl AsRef<OsStr>) -> Vec<u16> {
        value.as_ref().encode_wide().chain(Some(0)).collect()
    }
    fn lua() -> Result<PathBuf, String> {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../lua/lua.exe")
            .canonicalize()
            .map_err(|e| e.to_string())
    }
    fn created(h: HANDLE) -> Result<u64, String> {
        let mut a: FILETIME = unsafe { zeroed() };
        let mut b = a;
        let mut c = a;
        let mut d = a;
        if unsafe { GetProcessTimes(h, &mut a, &mut b, &mut c, &mut d) } == 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
        Ok((u64::from(a.dwHighDateTime) << 32) | u64::from(a.dwLowDateTime))
    }
    fn image(h: HANDLE) -> Result<PathBuf, String> {
        let mut buf = vec![0u16; 32768];
        let mut len = buf.len() as u32;
        if unsafe { QueryFullProcessImageNameW(h, 0, buf.as_mut_ptr(), &mut len) } == 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
        PathBuf::from(String::from_utf16(&buf[..len as usize]).map_err(|e| e.to_string())?)
            .canonicalize()
            .map_err(|e| e.to_string())
    }
    fn same(left: &Path, right: &Path) -> bool {
        left.as_os_str()
            .to_string_lossy()
            .eq_ignore_ascii_case(&right.as_os_str().to_string_lossy())
    }
    fn argument_path(path: &Path) -> Result<String, String> {
        let text = path.to_str().ok_or("workflow path is not Unicode")?;
        // Canonical handles use verbatim names, but the fixed Lua runtime's
        // argv/file loader cannot consume \\?\ paths (verified directly).
        // This conversion does not change the canonical identity checks.
        Ok(if let Some(rest) = text.strip_prefix("\\\\?\\UNC\\") {
            format!("\\\\{rest}")
        } else {
            text.strip_prefix("\\\\?\\").unwrap_or(text).to_owned()
        })
    }
    // Retain every ancestor handle while validating. Toolhelp PIDs are lookup
    // hints only: a reused parent's birth after its child is always rejected.
    fn owner() -> Result<Handle, String> {
        let snapshot = handle(
            unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) },
            "process snapshot",
        )?;
        let mut entries = Vec::new();
        let mut entry: PROCESSENTRY32W = unsafe { zeroed() };
        entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;
        if unsafe { Process32FirstW(snapshot.0, &mut entry) } == 0 {
            return Err("empty process snapshot".into());
        }
        loop {
            entries.push((entry.th32ProcessID, entry.th32ParentProcessID));
            if unsafe { Process32NextW(snapshot.0, &mut entry) } == 0 {
                break;
            }
        }
        let expected = lua()?;
        let cmd = PathBuf::from(env::var_os("SystemRoot").ok_or("missing SystemRoot")?)
            .join("System32/cmd.exe")
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let mut pid = unsafe { GetCurrentProcessId() };
        let mut child_birth = created(unsafe { GetCurrentProcess() })?;
        let mut retained = Vec::new();
        for depth in 0..2 {
            let parent = entries
                .iter()
                .find(|e| e.0 == pid)
                .ok_or("missing parent snapshot")?
                .1;
            if parent == 0 || parent == pid {
                return Err("invalid workflow ancestry".into());
            }
            let process = handle(
                unsafe {
                    OpenProcess(
                        PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
                        0,
                        parent,
                    )
                },
                "open ancestor",
            )?;
            let birth = created(process.0)?;
            if birth > child_birth || unsafe { WaitForSingleObject(process.0, 0) } != WAIT_TIMEOUT {
                return Err("stale workflow ancestry".into());
            }
            let exe = image(process.0)?;
            if same(&exe, &expected) {
                return Ok(process);
            }
            if depth != 0 || !same(&exe, &cmd) {
                return Err("workflow caller must be fixed Lua, optionally through cmd.exe".into());
            }
            retained.push(process);
            pid = parent;
            child_birth = birth;
        }
        Err("fixed Lua owner not found".into())
    }
    pub fn member(params: &Value) -> Result<Value, String> {
        let name = params
            .get("job_name")
            .and_then(Value::as_str)
            .ok_or("missing job name")?;
        if !name.starts_with(PREFIX) || name.len() > 160 || name.contains('\0') {
            return Err("invalid private workflow marker".into());
        }
        let owner = owner()?;
        let job = handle(
            unsafe { OpenJobObjectW(JOB_OBJECT_QUERY, 0, wide(name).as_ptr()) },
            "open private workflow job",
        )?;
        let mut belongs = 0;
        if unsafe { IsProcessInJob(owner.0, job.0, &mut belongs) } == 0 || belongs == 0 {
            return Err("Lua caller is not a member of its private workflow job".into());
        }
        Ok(json!({"supervised":true}))
    }
    struct Suspended(PROCESS_INFORMATION);
    impl Drop for Suspended {
        fn drop(&mut self) {
            unsafe {
                // Only this original CreateProcess handle, never a PID lookup.
                if WaitForSingleObject(self.0.hProcess, 0) == WAIT_TIMEOUT {
                    TerminateProcess(self.0.hProcess, 125);
                    WaitForSingleObject(self.0.hProcess, 5000);
                }
                CloseHandle(self.0.hThread);
                CloseHandle(self.0.hProcess);
            }
        }
    }
    pub fn supervise(params: &Value) -> Result<Value, String> {
        let owner = owner()?;
        let exe = lua()?;
        let script = PathBuf::from(
            params
                .get("script")
                .and_then(Value::as_str)
                .ok_or("missing workflow script")?,
        )
        .canonicalize()
        .map_err(|e| e.to_string())?;
        if !script.is_file() || script.extension().and_then(|s| s.to_str()) != Some("lua") {
            return Err("workflow script must be a saved Lua file".into());
        }
        let cwd = PathBuf::from(
            params
                .get("cwd")
                .and_then(Value::as_str)
                .ok_or("missing workflow cwd")?,
        )
        .canonicalize()
        .map_err(|e| e.to_string())?;
        if !cwd.is_dir() {
            return Err("workflow cwd must be a directory".into());
        }
        let args = params
            .get("args")
            .and_then(Value::as_array)
            .ok_or("missing workflow args")?;
        if args.len() > 256 {
            return Err("too many workflow arguments".into());
        }
        let mut line = quote(&argument_path(&exe)?)? + " " + &quote(&argument_path(&script)?)?;
        for value in args {
            line.push(' ');
            line.push_str(&quote(
                value.as_str().ok_or("workflow argument is not a string")?,
            )?);
        }
        let mut command = wide(&line);
        if command.len() > 32767 {
            return Err("workflow command line exceeds Windows limit".into());
        }
        let name = format!(
            "{PREFIX}{}-{}",
            unsafe { GetCurrentProcessId() },
            created(unsafe { GetCurrentProcess() })?
        );
        let job = handle(
            unsafe { CreateJobObjectW(null(), wide(&name).as_ptr()) },
            "create workflow job",
        )?;
        if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
            return Err("private workflow job collision".into());
        }
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        if unsafe {
            SetInformationJobObject(
                job.0,
                JobObjectExtendedLimitInformation,
                &limits as *const _ as _,
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        } == 0
        {
            return Err(std::io::Error::last_os_error().to_string());
        }
        let mut vars: Vec<_> = env::vars_os()
            .filter(|(key, _)| !key.to_string_lossy().eq_ignore_ascii_case(MARKER))
            .collect();
        vars.push((MARKER.into(), name.into()));
        vars.sort_by_key(|(key, _)| key.to_string_lossy().to_uppercase());
        let mut environment = Vec::new();
        for (key, value) in vars {
            environment.extend(key.encode_wide());
            environment.push(b'=' as u16);
            environment.extend(value.encode_wide());
            environment.push(0);
        }
        environment.push(0);
        let mut startup: STARTUPINFOW = unsafe { zeroed() };
        startup.cb = size_of::<STARTUPINFOW>() as u32;
        // Default CreateProcess standard handles keep the caller's console or
        // redirected stream. Job/owner handles are non-inheritable.
        let mut info: PROCESS_INFORMATION = unsafe { zeroed() };
        if unsafe {
            CreateProcessW(
                wide(exe.as_os_str()).as_ptr(),
                command.as_mut_ptr(),
                null(),
                null(),
                1,
                CREATE_SUSPENDED | CREATE_UNICODE_ENVIRONMENT,
                environment.as_ptr() as _,
                wide(argument_path(&cwd)?).as_ptr(),
                &startup,
                &mut info,
            )
        } == 0
        {
            return Err(std::io::Error::last_os_error().to_string());
        }
        let child = Suspended(info);
        if unsafe { AssignProcessToJobObject(job.0, child.0.hProcess) } == 0 {
            return Err(format!(
                "assign suspended workflow: {}",
                std::io::Error::last_os_error()
            ));
        }
        // No user code has run before successful assignment. Owner can exit
        // during setup; reject before resume rather than leaving a live tree.
        if unsafe { WaitForSingleObject(owner.0, 0) } != WAIT_TIMEOUT {
            return Err("workflow owner exited before resume".into());
        }
        if unsafe { ResumeThread(child.0.hThread) } == u32::MAX {
            return Err("cannot resume assigned workflow".into());
        }
        let handles = [owner.0, child.0.hProcess];
        let signal = unsafe { WaitForMultipleObjects(2, handles.as_ptr(), 0, INFINITE) };
        if signal != WAIT_OBJECT_0 && signal != WAIT_OBJECT_0 + 1 {
            return Err("workflow lifetime wait failed".into());
        }
        let mut exit_code = 125;
        if signal == WAIT_OBJECT_0 + 1
            && unsafe { GetExitCodeProcess(child.0.hProcess, &mut exit_code) } == 0
        {
            return Err("cannot read workflow exit code".into());
        }
        if unsafe { TerminateJobObject(job.0, 125) } == 0 {
            return Err("cannot retire workflow job".into());
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let mut accounting: JOBOBJECT_BASIC_ACCOUNTING_INFORMATION = unsafe { zeroed() };
            if unsafe {
                QueryInformationJobObject(
                    job.0,
                    JobObjectBasicAccountingInformation,
                    &mut accounting as *mut _ as _,
                    size_of::<JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>() as u32,
                    null_mut(),
                )
            } == 0
            {
                return Err("cannot confirm workflow retirement".into());
            }
            if accounting.ActiveProcesses == 0 {
                break;
            }
            if Instant::now() >= deadline {
                return Err("workflow job retirement timed out".into());
            }
            thread::sleep(Duration::from_millis(20));
        }
        Ok(json!({"exit_code":exit_code,"owner_exited":signal==WAIT_OBJECT_0,"retired":true}))
    }
}
#[cfg(windows)]
pub use windows::{member, supervise};

#[cfg(test)]
mod tests {
    use super::quote;
    #[test]
    fn exact_windows_argument_quoting() {
        assert_eq!(quote("").unwrap(), "\"\"");
        assert_eq!(quote("a b").unwrap(), "\"a b\"");
        assert_eq!(quote("a\"b").unwrap(), "\"a\\\"b\"");
        assert_eq!(quote("x\\").unwrap(), "\"x\\\\\"");
        assert!(quote("a\0b").is_err());
    }
}
