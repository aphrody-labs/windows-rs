//! Win32 and NT calls compiled to WebAssembly host imports.
//!
//! Built for `wasm32-unknown-unknown`, the module imports exactly
//!
//! ```text
//! (import "kernel32.dll" "GetTickCount64" (func (result i64)))
//! (import "ntdll.dll" "NtQuerySystemInformation" (func (param i32 i32 i32 i32) (result i32)))
//! ```
//!
//! and `host.js` provides them. Built for Windows, the same code calls the real DLLs.

use windows_sys::Win32::sysinfoapi::GetTickCount64;
use windows_sys::Win32::winternl::{
    NtQuerySystemInformation, SYSTEM_BASIC_INFORMATION, SystemBasicInformation,
};

/// Milliseconds since the system started.
#[unsafe(no_mangle)]
pub extern "C" fn uptime_ms() -> u64 {
    unsafe { GetTickCount64() }
}

/// The number of logical processors, or `-1` if the query fails.
#[unsafe(no_mangle)]
pub extern "C" fn processor_count() -> i32 {
    let mut info = SYSTEM_BASIC_INFORMATION::default();
    let mut length = 0;

    let status = unsafe {
        NtQuerySystemInformation(
            SystemBasicInformation,
            (&raw mut info).cast(),
            size_of::<SYSTEM_BASIC_INFORMATION>() as u32,
            &mut length,
        )
    };

    if status < 0 {
        -1
    } else {
        i32::from(info.NumberOfProcessors)
    }
}
