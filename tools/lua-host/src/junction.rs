// Directory junctions (NTFS mount points). Creating a directory symlink needs a privilege
// most accounts lack; a junction does not. Used to assemble run snapshots that share the
// live project's Content/Config/Saved without copying them (scripts/ue_run_snapshot.lua).
use serde_json::{Value, json};

#[cfg(windows)]
mod imp {
    use std::{ffi::OsStr, os::windows::ffi::OsStrExt, path::Path, ptr};
    use windows_sys::Win32::{
        Foundation::{CloseHandle, GENERIC_READ, GENERIC_WRITE, INVALID_HANDLE_VALUE},
        Storage::FileSystem::{
            CreateDirectoryW, CreateFileW, FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_BACKUP_SEMANTICS,
            FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
            GetFileAttributesW, INVALID_FILE_ATTRIBUTES, OPEN_EXISTING, RemoveDirectoryW,
        },
        System::{
            IO::DeviceIoControl,
            Ioctl::{FSCTL_GET_REPARSE_POINT, FSCTL_SET_REPARSE_POINT},
            SystemServices::IO_REPARSE_TAG_MOUNT_POINT,
        },
    };

    fn wide(value: &OsStr) -> Vec<u16> {
        value.encode_wide().chain(std::iter::once(0)).collect()
    }

    fn open(link: &[u16], access: u32) -> Result<isize, String> {
        let handle = unsafe {
            CreateFileW(link.as_ptr(), access, FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                ptr::null(), OPEN_EXISTING, FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS, ptr::null_mut())
        };
        if handle == INVALID_HANDLE_VALUE {
            return Err(format!("cannot open {}: {}", String::from_utf16_lossy(&link[..link.len() - 1]), std::io::Error::last_os_error()));
        }
        Ok(handle as isize)
    }

    pub fn create(link: &Path, target: &Path) -> Result<(), String> {
        let target = std::fs::canonicalize(target).map_err(|e| format!("junction target {}: {e}", target.display()))?;
        let target = target.to_string_lossy();
        // canonicalize returns \\?\C:\...; a mount point stores the NT form \??\C:\...
        let plain = target.strip_prefix(r"\\?\").unwrap_or(&target).to_string();
        let substitute: Vec<u16> = OsStr::new(&format!(r"\??\{plain}")).encode_wide().collect();
        let print: Vec<u16> = OsStr::new(&plain).encode_wide().collect();
        let link_w = wide(link.as_os_str());
        if unsafe { CreateDirectoryW(link_w.as_ptr(), ptr::null()) } == 0 {
            return Err(format!("cannot create {}: {}", link.display(), std::io::Error::last_os_error()));
        }
        let path_bytes = (substitute.len() + 1 + print.len() + 1) * 2;
        let data_len = 8 + path_bytes;
        let mut buffer: Vec<u8> = Vec::with_capacity(8 + data_len);
        buffer.extend_from_slice(&IO_REPARSE_TAG_MOUNT_POINT.to_le_bytes());
        buffer.extend_from_slice(&(data_len as u16).to_le_bytes());
        buffer.extend_from_slice(&0u16.to_le_bytes());
        buffer.extend_from_slice(&0u16.to_le_bytes());
        buffer.extend_from_slice(&((substitute.len() * 2) as u16).to_le_bytes());
        buffer.extend_from_slice(&(((substitute.len() + 1) * 2) as u16).to_le_bytes());
        buffer.extend_from_slice(&((print.len() * 2) as u16).to_le_bytes());
        for unit in substitute.iter().chain([0u16].iter()).chain(print.iter()).chain([0u16].iter()) {
            buffer.extend_from_slice(&unit.to_le_bytes());
        }
        let handle = match open(&link_w, GENERIC_WRITE) {
            Ok(handle) => handle,
            Err(error) => { unsafe { RemoveDirectoryW(link_w.as_ptr()) }; return Err(error); }
        };
        let mut returned = 0u32;
        let ok = unsafe {
            DeviceIoControl(handle as _, FSCTL_SET_REPARSE_POINT, buffer.as_ptr() as _, buffer.len() as u32,
                ptr::null_mut(), 0, &mut returned, ptr::null_mut())
        };
        let error = std::io::Error::last_os_error();
        unsafe { CloseHandle(handle as _) };
        if ok == 0 {
            unsafe { RemoveDirectoryW(link_w.as_ptr()) };
            return Err(format!("cannot make {} a junction: {error}", link.display()));
        }
        Ok(())
    }

    // Some(true) for a junction, Some(false) for anything else that exists, None when missing.
    pub fn is_junction(link: &Path) -> Result<Option<bool>, String> {
        let link_w = wide(link.as_os_str());
        let attributes = unsafe { GetFileAttributesW(link_w.as_ptr()) };
        if attributes == INVALID_FILE_ATTRIBUTES { return Ok(None); }
        if attributes & FILE_ATTRIBUTE_REPARSE_POINT == 0 { return Ok(Some(false)); }
        let handle = open(&link_w, GENERIC_READ)?;
        let mut buffer = vec![0u8; 16 * 1024];
        let mut returned = 0u32;
        let ok = unsafe {
            DeviceIoControl(handle as _, FSCTL_GET_REPARSE_POINT, ptr::null(), 0, buffer.as_mut_ptr() as _,
                buffer.len() as u32, &mut returned, ptr::null_mut())
        };
        let error = std::io::Error::last_os_error();
        unsafe { CloseHandle(handle as _) };
        if ok == 0 { return Err(format!("cannot read reparse point {}: {error}", link.display())); }
        Ok(Some(u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]) == IO_REPARSE_TAG_MOUNT_POINT))
    }

    // Removes only the link; the target's contents are never touched.
    pub fn remove(link: &Path) -> Result<bool, String> {
        if is_junction(link)? != Some(true) { return Ok(false); }
        let link_w = wide(link.as_os_str());
        if unsafe { RemoveDirectoryW(link_w.as_ptr()) } == 0 {
            return Err(format!("cannot remove junction {}: {}", link.display(), std::io::Error::last_os_error()));
        }
        Ok(true)
    }
}

fn path_param<'a>(params: &'a Value, name: &str) -> Result<&'a str, String> {
    params.get(name).and_then(Value::as_str).filter(|v| !v.is_empty()).ok_or_else(|| format!("missing {name}"))
}

#[cfg(windows)]
pub fn create(params: &Value) -> Result<Value, String> {
    imp::create(path_param(params, "link")?.as_ref(), path_param(params, "target")?.as_ref())?;
    Ok(json!({"created": true}))
}
#[cfg(windows)]
pub fn inspect(params: &Value) -> Result<Value, String> {
    let state = imp::is_junction(path_param(params, "link")?.as_ref())?;
    Ok(json!({"exists": state.is_some(), "junction": state == Some(true)}))
}
#[cfg(windows)]
pub fn remove(params: &Value) -> Result<Value, String> {
    Ok(json!({"removed": imp::remove(path_param(params, "link")?.as_ref())?}))
}

#[cfg(not(windows))]
pub fn create(_: &Value) -> Result<Value, String> { Err("junctions are Windows-only".into()) }
#[cfg(not(windows))]
pub fn inspect(_: &Value) -> Result<Value, String> { Err("junctions are Windows-only".into()) }
#[cfg(not(windows))]
pub fn remove(_: &Value) -> Result<Value, String> { Err("junctions are Windows-only".into()) }
