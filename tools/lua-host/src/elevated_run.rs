//! Explicit Windows UAC request for authorized machine setup. No shell and no
//! automatic retry after cancellation; the OS owns consent and credentials.
use serde_json::Value;

fn quote_argument(value: &str) -> Result<String, String> {
    if value.contains('\0') { return Err("NUL in elevated argument".into()); }
    let mut out = String::from("\"");
    let mut slashes = 0;
    for ch in value.chars() {
        if ch == '\\' { slashes += 1; continue; }
        out.extend(std::iter::repeat_n('\\', if ch == '"' { slashes * 2 + 1 } else { slashes }));
        out.push(ch);
        slashes = 0;
    }
    out.extend(std::iter::repeat_n('\\', slashes * 2));
    out.push('"');
    Ok(out)
}

#[cfg(windows)]
pub fn run(params: &Value) -> Result<Value, String> {
    use windows_sys::Win32::{Foundation::CloseHandle, System::Threading::{GetExitCodeProcess, WaitForSingleObject, INFINITE}, UI::{Shell::{ShellExecuteExW, SHELLEXECUTEINFOW, SEE_MASK_NOCLOSEPROCESS, SEE_MASK_NOASYNC}, WindowsAndMessaging::SW_HIDE}};
    let exe = super::required_str(params, "exe")?;
    if !std::path::Path::new(exe).is_absolute() || !exe.to_ascii_lowercase().ends_with(".exe") {
        return Err("elevated executable requires an absolute .exe path".into());
    }
    let args = params.get("args").and_then(Value::as_array).ok_or("missing elevated args")?;
    let command_line = args.iter().map(|v| quote_argument(v.as_str().ok_or("elevated arg must be string")?))
        .collect::<Result<Vec<_>, String>>()?.join(" ");
    fn wide(value: &str) -> Result<Vec<u16>, String> {
        if value.contains('\0') { return Err("NUL in elevated parameter".into()); }
        Ok(value.encode_utf16().chain(Some(0)).collect())
    }
    let file = wide(exe)?;
    let parameters = wide(&command_line)?;
    let verb = wide("runas")?;
    let mut info: SHELLEXECUTEINFOW = unsafe { std::mem::zeroed() };
    info.cbSize = std::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
    info.fMask = SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC;
    info.lpVerb = verb.as_ptr();
    info.lpFile = file.as_ptr();
    info.lpParameters = parameters.as_ptr();
    info.nShow = SW_HIDE;
    if unsafe { ShellExecuteExW(&mut info) } == 0 {
        return Err(format!("Windows UAC launch failed: {}", std::io::Error::last_os_error()));
    }
    if info.hProcess.is_null() { return Err("UAC launch returned no process handle".into()); }
    let result = (|| {
        if unsafe { WaitForSingleObject(info.hProcess, INFINITE) } != 0 {
            return Err(format!("elevated wait failed: {}", std::io::Error::last_os_error()));
        }
        let mut code = 0;
        if unsafe { GetExitCodeProcess(info.hProcess, &mut code) } == 0 {
            return Err(format!("elevated exit code failed: {}", std::io::Error::last_os_error()));
        }
        Ok(serde_json::json!({"exit_code":code}))
    })();
    unsafe { CloseHandle(info.hProcess); }
    result
}

#[cfg(not(windows))]
pub fn run(_: &Value) -> Result<Value, String> { Err("UAC is supported only on Windows".into()) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn structured_arguments_preserve_spaces_quotes_and_trailing_slashes() {
        for (arg, expected) in [("", "\"\""), ("a b", "\"a b\""), ("a\"b", "\"a\\\"b\""), ("C:\\path\\", "\"C:\\path\\\\\""), ("$(x)&%PATH%", "\"$(x)&%PATH%\"")] {
            assert_eq!(quote_argument(arg).unwrap(), expected);
        }
        assert!(quote_argument("bad\0arg").is_err());
    }
}
