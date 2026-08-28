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
    value.get(name).and_then(Value::as_str).ok_or_else(|| format!("missing string {name}"))
}
fn required_u32(value: &Value, name: &str) -> Result<u32, String> {
    value.get(name).and_then(Value::as_u64).and_then(|v| u32::try_from(v).ok()).ok_or_else(|| format!("missing u32 {name}"))
}

fn run_command(params: &Value) -> Result<Value, String> {
    let exe = required_str(params, "exe")?;
    let args = params.get("args").and_then(Value::as_array).cloned().unwrap_or_default();
    let mut command = Command::new(exe);
    command.args(args.iter().map(|v| v.as_str().ok_or("command arg must be string")).collect::<Result<Vec<_>, _>>()?);
    if let Some(cwd) = params.get("cwd").and_then(Value::as_str) { command.current_dir(cwd); }
    if let Some(envs) = params.get("env").and_then(Value::as_object) {
        for (key, value) in envs { command.env(key, value.as_str().ok_or("environment value must be string")?); }
    }
    let output = command.output().map_err(|e| format!("run {exe}: {e}"))?;
    Ok(json!({"exit_code": output.status.code().unwrap_or(-1), "stdout": String::from_utf8_lossy(&output.stdout), "stderr": String::from_utf8_lossy(&output.stderr)}))
}

fn spawn(params: &Value) -> Result<Value, String> {
    let exe = required_str(params, "exe")?;
    let args = params.get("args").and_then(Value::as_array).cloned().unwrap_or_default();
    let stdout_path = PathBuf::from(required_str(params, "stdout")?);
    let stderr_path = PathBuf::from(required_str(params, "stderr")?);
    if let Some(parent) = stdout_path.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
    if let Some(parent) = stderr_path.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
    let mut command = Command::new(exe);
    command.args(args.iter().map(|v| v.as_str().ok_or("spawn arg must be string")).collect::<Result<Vec<_>, _>>()?);
    if let Some(cwd) = params.get("cwd").and_then(Value::as_str) { command.current_dir(cwd); }
    if let Some(envs) = params.get("env").and_then(Value::as_object) {
        for (key, value) in envs { command.env(key, value.as_str().ok_or("environment value must be string")?); }
    }
    command.stdin(Stdio::null())
        .stdout(Stdio::from(fs::File::create(&stdout_path).map_err(|e| e.to_string())?))
        .stderr(Stdio::from(fs::File::create(&stderr_path).map_err(|e| e.to_string())?));
    #[cfg(windows)] { use std::os::windows::process::CommandExt; command.creation_flags(0x08000000); }
    let child = command.spawn().map_err(|e| format!("spawn {exe}: {e}"))?;
    Ok(json!({"pid": child.id()}))
}

#[cfg(windows)]
fn process_path(pid: u32) -> Result<PathBuf, String> {
    use windows_sys::Win32::{Foundation::CloseHandle, System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION}};
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() { return Err(format!("cannot open PID {pid}: {}", io::Error::last_os_error())); }
        let mut buffer = vec![0u16; 32768]; let mut length = buffer.len() as u32;
        let ok = QueryFullProcessImageNameW(handle, 0, buffer.as_mut_ptr(), &mut length);
        CloseHandle(handle);
        if ok == 0 { return Err(format!("cannot query PID {pid}: {}", io::Error::last_os_error())); }
        Ok(PathBuf::from(String::from_utf16_lossy(&buffer[..length as usize])))
    }
}
#[cfg(not(windows))]
fn process_path(pid: u32) -> Result<PathBuf, String> { fs::read_link(format!("/proc/{pid}/exe")).map_err(|e| e.to_string()) }

fn inspect(params: &Value) -> Result<Value, String> {
    let pid=required_u32(params,"pid")?; let path=process_path(pid)?;
    Ok(json!({"pid":pid,"path":path,"alive":true}))
}

#[cfg(windows)]
fn terminate(pid: u32, exit_code: u32) -> Result<(), String> {
    use windows_sys::Win32::{Foundation::CloseHandle, System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE}};
    unsafe { let handle=OpenProcess(PROCESS_TERMINATE,0,pid);if handle.is_null(){return Err(io::Error::last_os_error().to_string())}let ok=TerminateProcess(handle,exit_code);CloseHandle(handle);if ok==0{return Err(io::Error::last_os_error().to_string())}Ok(()) }
}
#[cfg(not(windows))]
fn terminate(pid:u32,_:u32)->Result<(),String>{Command::new("kill").args(["-TERM",&pid.to_string()]).status().map_err(|e|e.to_string()).and_then(|s|if s.success(){Ok(())}else{Err("kill failed".into())})}

fn stop(params:&Value)->Result<Value,String>{let pid=required_u32(params,"pid")?;let expected=fs::canonicalize(required_str(params,"expected_exe")?).map_err(|e|e.to_string())?;let actual=fs::canonicalize(process_path(pid)?).map_err(|e|e.to_string())?;if actual!=expected{return Err(format!("PID executable mismatch: {} != {}",actual.display(),expected.display()))}terminate(pid,1)?;Ok(json!({"pid":pid,"stopped":true}))}
fn udp(params:&Value)->Result<Value,String>{let addr=required_str(params,"addr")?;let payload=required_str(params,"payload")?.as_bytes();let socket=UdpSocket::bind("127.0.0.1:0").map_err(|e|e.to_string())?;let sent=socket.send_to(payload,addr).map_err(|e|e.to_string())?;Ok(json!({"sent":sent}))}
fn sha256(params:&Value)->Result<Value,String>{let file=required_str(params,"path")?;let mut input=fs::File::open(file).map_err(|e|e.to_string())?;let mut hash=Sha256::new();io::copy(&mut input,&mut hash).map_err(|e|e.to_string())?;Ok(json!({"sha256":format!("{:x}",hash.finalize())}))}

fn wait_for(params:&Value)->Result<Value,String>{let pid=required_u32(params,"pid")?;let timeout=params.get("timeout_ms").and_then(Value::as_u64).unwrap_or(5000);let start=Instant::now();while start.elapsed()<Duration::from_millis(timeout){if process_path(pid).is_err(){return Ok(json!({"exited":true}))}thread::sleep(Duration::from_millis(50));}Ok(json!({"exited":false}))}

#[cfg(windows)]
fn memory_dump(params:&Value)->Result<Value,String>{
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::{Foundation::CloseHandle,System::{Diagnostics::Debug::MiniDumpWriteDump,Threading::{OpenProcess,PROCESS_QUERY_INFORMATION,PROCESS_VM_READ}}};
    let pid=required_u32(params,"pid")?;let output=required_str(params,"output")?;let expected=fs::canonicalize(required_str(params,"expected_exe")?).map_err(|e|e.to_string())?;let actual=fs::canonicalize(process_path(pid)?).map_err(|e|e.to_string())?;if actual!=expected{return Err("PID executable mismatch".into())}if let Some(parent)=Path::new(output).parent(){fs::create_dir_all(parent).map_err(|e|e.to_string())?}let file=fs::File::create(output).map_err(|e|e.to_string())?;unsafe{let process=OpenProcess(PROCESS_QUERY_INFORMATION|PROCESS_VM_READ,0,pid);if process.is_null(){return Err(io::Error::last_os_error().to_string())}let ok=MiniDumpWriteDump(process,pid,file.as_raw_handle() as _,2,std::ptr::null(),std::ptr::null(),std::ptr::null());CloseHandle(process);if ok==0{return Err(io::Error::last_os_error().to_string())}}Ok(json!({"pid":pid,"path":output}))
}
#[cfg(not(windows))]
fn memory_dump(params:&Value)->Result<Value,String>{let pid=required_u32(params,"pid")?;let output=required_str(params,"output")?;let status=Command::new("gcore").args(["-o",output,&pid.to_string()]).status().map_err(|e|e.to_string())?;if !status.success(){return Err("gcore failed".into())}Ok(json!({"pid":pid,"path":output}))}

#[cfg(windows)]
fn screenshot(params:&Value)->Result<Value,String>{
 use windows_sys::Win32::{Foundation::{BOOL,HWND,LPARAM,RECT},Graphics::Gdi::{BITMAPINFO,BITMAPINFOHEADER,BI_RGB,CreateCompatibleBitmap,CreateCompatibleDC,DeleteDC,DeleteObject,GetDIBits,GetWindowDC,ReleaseDC,SelectObject,DIB_RGB_COLORS},UI::WindowsAndMessaging::{EnumWindows,GetWindowRect,GetWindowThreadProcessId,IsWindowVisible,PrintWindow}};
 #[repr(C)]struct Search{pid:u32,hwnd:HWND}unsafe extern "system" fn callback(hwnd:HWND,param:LPARAM)->BOOL{let search=&mut*(param as*mut Search);let mut pid=0;GetWindowThreadProcessId(hwnd,&mut pid);if pid==search.pid&&IsWindowVisible(hwnd)!=0{search.hwnd=hwnd;return 0}1}
 let pid=required_u32(params,"pid")?;let output=required_str(params,"output")?;let mut search=Search{pid,hwnd:std::ptr::null_mut()};unsafe{EnumWindows(Some(callback),&mut search as*mut _ as LPARAM)};if search.hwnd.is_null(){return Err(format!("PID {pid} has no visible window"))}let mut rect=RECT::default();unsafe{if GetWindowRect(search.hwnd,&mut rect)==0{return Err(io::Error::last_os_error().to_string())}}let width=rect.right-rect.left;let height=rect.bottom-rect.top;if width<=0||height<=0{return Err("invalid window bounds".into())}let mut pixels=vec![0u8;(width*height*4)as usize];unsafe{let window_dc=GetWindowDC(search.hwnd);let memory_dc=CreateCompatibleDC(window_dc);let bitmap=CreateCompatibleBitmap(window_dc,width,height);let old=SelectObject(memory_dc,bitmap);if PrintWindow(search.hwnd,memory_dc,2)==0{return Err(io::Error::last_os_error().to_string())}let mut info=BITMAPINFO::default();info.bmiHeader=BITMAPINFOHEADER{biSize:std::mem::size_of::<BITMAPINFOHEADER>()as u32,biWidth:width,biHeight:-height,biPlanes:1,biBitCount:32,biCompression:BI_RGB,..Default::default()};if GetDIBits(memory_dc,bitmap,0,height as u32,pixels.as_mut_ptr()as _,&mut info,DIB_RGB_COLORS)==0{return Err(io::Error::last_os_error().to_string())}SelectObject(memory_dc,old);DeleteObject(bitmap);DeleteDC(memory_dc);ReleaseDC(search.hwnd,window_dc)}for p in pixels.chunks_exact_mut(4){p.swap(0,2)}if let Some(parent)=Path::new(output).parent(){fs::create_dir_all(parent).map_err(|e|e.to_string())?}let file=fs::File::create(output).map_err(|e|e.to_string())?;let mut encoder=png::Encoder::new(file,width as u32,height as u32);encoder.set_color(png::ColorType::Rgba);encoder.set_depth(png::BitDepth::Eight);encoder.write_header().map_err(|e|e.to_string())?.write_image_data(&pixels).map_err(|e|e.to_string())?;Ok(json!({"pid":pid,"path":output,"width":width,"height":height}))
}
#[cfg(not(windows))]
fn screenshot(_:&Value)->Result<Value,String>{Err("window screenshot is only supported on Windows".into())}

#[cfg(windows)]
fn close_window(params:&Value)->Result<Value,String>{use windows_sys::Win32::{Foundation::{BOOL,HWND,LPARAM},UI::WindowsAndMessaging::{EnumWindows,GetWindowThreadProcessId,PostMessageW,WM_CLOSE}};#[repr(C)]struct Search{pid:u32,count:u32}unsafe extern "system" fn callback(hwnd:HWND,param:LPARAM)->BOOL{let s=&mut*(param as*mut Search);let mut pid=0;GetWindowThreadProcessId(hwnd,&mut pid);if pid==s.pid&&PostMessageW(hwnd,WM_CLOSE,0,0)!=0{s.count+=1}1}let mut search=Search{pid:required_u32(params,"pid")?,count:0};unsafe{EnumWindows(Some(callback),&mut search as*mut _ as LPARAM)}Ok(json!({"posted":search.count}))}
#[cfg(not(windows))]
fn close_window(params:&Value)->Result<Value,String>{let pid=required_u32(params,"pid")?;let status=Command::new("kill").args(["-TERM",&pid.to_string()]).status().map_err(|e|e.to_string())?;Ok(json!({"posted":if status.success(){1}else{0}}))}

fn dispatch(request:&Request)->Result<Value,String>{match request.operation.as_str(){"run"=>run_command(&request.params),"spawn"=>spawn(&request.params),"inspect"=>inspect(&request.params),"stop"=>stop(&request.params),"close_window"=>close_window(&request.params),"udp_send"=>udp(&request.params),"sha256"=>sha256(&request.params),"wait"=>wait_for(&request.params),"memory_dump"=>memory_dump(&request.params),"screenshot"=>screenshot(&request.params),other=>Err(format!("unknown operation {other}"))}}

fn main(){let response=(||->Result<Value,String>{let mut text=String::new();if let Some(file)=env::args_os().nth(1){text=fs::read_to_string(file).map_err(|e|e.to_string())?}else{io::stdin().read_to_string(&mut text).map_err(|e|e.to_string())?};let request:Request=serde_json::from_str(&text).map_err(|e|e.to_string())?;if request.version!=VERSION{return Err(format!("unsupported request version {}",request.version))}dispatch(&request)})();let output=match response{Ok(result)=>Response{version:VERSION,ok:true,result,error:None},Err(error)=>Response{version:VERSION,ok:false,result:Value::Null,error:Some(error)}};println!("{}",serde_json::to_string(&output).expect("response serialization"));if !output.ok{std::process::exit(2)}}
