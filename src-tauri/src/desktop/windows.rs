use std::{ffi::OsStr, io, os::windows::ffi::OsStrExt, ptr::{null, null_mut}};
use windows_sys::Win32::{
    Networking::WinHttp::*,
    UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOWNORMAL},
};

fn wide(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain([0]).collect()
}

pub fn open(target: &str) -> io::Result<()> {
    let result = unsafe {
        ShellExecuteW(null_mut(), wide("open").as_ptr(), wide(target).as_ptr(), null(), null(), SW_SHOWNORMAL)
    } as isize;
    if result > 32 { Ok(()) }
    else { Err(io::Error::other(format!("ShellExecute returned {result}"))) }
}

struct Http(*mut std::ffi::c_void);

impl Http {
    fn new(handle: *mut std::ffi::c_void) -> io::Result<Self> {
        if handle.is_null() { Err(io::Error::last_os_error()) } else { Ok(Self(handle)) }
    }
}

impl Drop for Http {
    fn drop(&mut self) { unsafe { WinHttpCloseHandle(self.0); } }
}

fn check(success: i32) -> io::Result<()> {
    if success == 0 { Err(io::Error::last_os_error()) } else { Ok(()) }
}

pub fn latest_release() -> io::Result<Vec<u8>> {
    unsafe {
        let session = Http::new(WinHttpOpen(wide("PicGlance").as_ptr(), WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY, null(), null(), 0))?;
        check(WinHttpSetTimeouts(session.0, 4000, 4000, 4000, 4000))?;
        let connection = Http::new(WinHttpConnect(session.0, wide("api.github.com").as_ptr(), 443, 0))?;
        let path = wide("/repos/Paosin/PicGlance/releases/latest");
        let request = Http::new(WinHttpOpenRequest(connection.0, wide("GET").as_ptr(), path.as_ptr(), null(), null(), null(), WINHTTP_FLAG_SECURE))?;
        let redirect_policy = WINHTTP_OPTION_REDIRECT_POLICY_NEVER;
        check(WinHttpSetOption(request.0, WINHTTP_OPTION_REDIRECT_POLICY, (&redirect_policy as *const u32).cast(), 4))?;
        let header = wide("Accept: application/vnd.github+json\r\n");
        check(WinHttpSendRequest(request.0, header.as_ptr(), (header.len() - 1) as u32, null(), 0, 0, 0))?;
        check(WinHttpReceiveResponse(request.0, null_mut()))?;
        let mut status = 0u32;
        let mut status_size = 4u32;
        check(WinHttpQueryHeaders(request.0, WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
            null(), (&mut status as *mut u32).cast(), &mut status_size, null_mut()))?;
        if status != 200 { return Err(io::Error::other(format!("GitHub returned HTTP {status}"))); }
        let mut payload = Vec::new();
        let mut chunk = [0u8; 8192];
        loop {
            let mut read = 0;
            check(WinHttpReadData(request.0, chunk.as_mut_ptr().cast(), chunk.len() as u32, &mut read))?;
            if read == 0 { return Ok(payload); }
            if payload.len() + read as usize > 1024 * 1024 {
                return Err(io::Error::other("GitHub release response exceeds 1 MiB"));
            }
            payload.extend_from_slice(&chunk[..read as usize]);
        }
    }
}
