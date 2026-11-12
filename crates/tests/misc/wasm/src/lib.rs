//! WebAssembly support checks.
//!
//! The crate builds for every target, so the Windows test jobs run it natively while `wasm.yml`
//! checks it for `wasm32-unknown-unknown` and `wasm32-wasip2`. On WebAssembly each `link!`
//! below becomes a host import whose module is the DLL name and whose field is the export name.
//! The constant assertions pin the 32-bit layouts that `windows-sys` selects for `wasm32`: the
//! same definitions as 32-bit (x86) Windows.

use core::ffi::c_void;
use windows_sys::Win32::basetsd::HALF_PTR;
use windows_sys::Win32::sysinfoapi::GetTickCount64;
use windows_sys::Win32::winternl::{
    NtQuerySystemInformation, SYSTEM_BASIC_INFORMATION, SystemBasicInformation,
};

// Imported as `"kernel32.dll" "GetCurrentProcessId"`.
windows_link::link!("kernel32.dll" "system" fn GetCurrentProcessId() -> u32);

// The link-name literal selects the import field: `"kernel32.dll" "GetTickCount"`.
windows_link::link!("kernel32.dll" "system" "GetTickCount" fn TickCount32() -> u32);

// WebAssembly imports have no ordinals, so this is imported as `"shlwapi.dll" "SHCreateMemStream"`.
windows_link::link!("shlwapi.dll" "system" ordinal(12) fn SHCreateMemStream(data: *const u8, len: u32) -> *mut c_void);

// `HALF_PTR` only exists under an architecture gate, so this also proves that `wasm32` gets one.
const _: () = assert!(size_of::<HALF_PTR>() * 2 == size_of::<usize>());
const _: () = assert!(size_of::<windows_sys::core::GUID>() == 16);
const _: () = assert!(size_of::<windows_core::HRESULT>() == 4);
const _: () = assert!(size_of::<windows_strings::HSTRING>() == size_of::<usize>());

#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(size_of::<SYSTEM_BASIC_INFORMATION>() == 44);
    assert!(core::mem::offset_of!(SYSTEM_BASIC_INFORMATION, NumberOfProcessors) == 40);
};

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(size_of::<SYSTEM_BASIC_INFORMATION>() == 64);
    assert!(core::mem::offset_of!(SYSTEM_BASIC_INFORMATION, NumberOfProcessors) == 56);
};

/// Milliseconds since the system started (`kernel32.dll!GetTickCount64`).
pub fn uptime_ms() -> u64 {
    unsafe { GetTickCount64() }
}

/// Low 32 bits of the tick count (`kernel32.dll!GetTickCount`).
pub fn uptime_ms_32() -> u32 {
    unsafe { TickCount32() }
}

/// The current process identifier (`kernel32.dll!GetCurrentProcessId`).
pub fn process_id() -> u32 {
    unsafe { GetCurrentProcessId() }
}

/// Logical processor count (`ntdll.dll!NtQuerySystemInformation`).
pub fn processor_count() -> Option<u8> {
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

    (status >= 0).then_some(info.NumberOfProcessors as u8)
}

/// A memory stream over `data` (`shlwapi.dll` ordinal 12).
pub fn create_mem_stream(data: &[u8]) -> *mut c_void {
    unsafe { SHCreateMemStream(data.as_ptr(), data.len() as u32) }
}

/// The calling thread's last error (`kernel32.dll!GetLastError` on Windows and WebAssembly).
pub fn last_error() -> windows_result::HRESULT {
    windows_result::HRESULT::from_thread()
}

/// The running OS version (`ntdll.dll!RtlGetVersion`).
pub fn os_version() -> windows_version::OsVersion {
    windows_version::OsVersion::current()
}

/// Opens a key under `HKEY_CURRENT_USER` (`advapi32.dll` registry functions).
pub fn open_user_key(path: &str) -> windows_result::Result<windows_registry::Key> {
    windows_registry::CURRENT_USER.open(path)
}

/// Queues `f` on the default thread pool (`kernel32.dll!TrySubmitThreadpoolCallback`).
pub fn submit<F: FnOnce() + Send + 'static>(f: F) {
    windows_threading::submit(f);
}

/// The first element of a WinRT iterable.
pub fn first(
    iterable: &windows_collections::IIterable<windows_strings::HSTRING>,
) -> windows_core::Result<windows_strings::HSTRING> {
    iterable.First()?.Current()
}

/// Pure Rust numerics, no imports.
pub fn length(x: f32, y: f32) -> f32 {
    windows_numerics::Vector2::new(x, y).length()
}
