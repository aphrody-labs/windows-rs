# WebAssembly host contract

On `target_family = "wasm"`, every function declared with `windows_link::link!` (all of
`windows-sys` and the generated bindings of the other crates) becomes a WebAssembly import. A
host — JavaScript, or another wasm module that implements the NT personality — supplies those
imports. Nothing is imported unless the final module references it.

## Import names

| | Value | Example |
|-|-|-|
| Module | The DLL name exactly as written in the metadata, including `.dll` | `kernel32.dll`, `ntdll.dll`, `api-ms-win-core-winrt-error-l1-1-0.dll` |
| Field | The export name: the link-name literal if present, otherwise the Rust identifier | `GetTickCount64`; `GetWindowLongPtrW` on x86 and wasm32 imports `user32.dll` `GetWindowLongW` |
| Ordinal imports | Imported by name; WebAssembly imports have no ordinals | `shlwapi.dll` `SHCreateMemStream` |

Since Rust 1.96, rustc no longer passes `--allow-undefined` to `wasm-ld`
([rust-lang/rust#149868](https://github.com/rust-lang/rust/pull/149868)), so a plain `extern`
declaration fails to link; `link!` uses `#[link(wasm_import_module = "<dll>")]`, which links as a
host import. Cross-crate `wasm_import_module` needs Rust 1.91.1 or later
([rust-lang/rust#148363](https://github.com/rust-lang/rust/pull/148363)).

## Calling convention

`extern "system"` is `extern "C"` everywhere except 32-bit x86 Windows
([Rust reference, external blocks](https://doc.rust-lang.org/reference/items/external-blocks.html)).
On WebAssembly that is the
[WebAssembly Basic C ABI](https://github.com/WebAssembly/tool-conventions/blob/main/BasicCABI.md),
which `wasm32-unknown-unknown` follows since Rust 1.89. `wasm32` is ILP32.

| Windows type | Wasm value | Notes |
|-|-|-|
| `BOOL`, `INT`, `LONG`, `DWORD`, `NTSTATUS`, `HRESULT`, `WCHAR`, `BYTE` | `i32` | 8- and 16-bit values are sign- or zero-extended |
| `LONGLONG`, `ULONGLONG`, `ULONG64` | `i64` | `BigInt` in JavaScript |
| `FLOAT`, `DOUBLE` | `f32`, `f64` | |
| Pointers, `SIZE_T`, `ULONG_PTR`, `WPARAM`, `LPARAM`, `LRESULT` | `i32` | Pointers are offsets into the module's linear memory (exported as `memory`); JavaScript sees them signed, use `>>> 0` |
| `HANDLE`, `HWND`, `HKEY`, other handles | `i32` | Opaque values chosen by the host; pseudo-handles keep their 32-bit values (`-1` is `0xFFFFFFFF`) |
| Struct or union holding one scalar (the `windows` crate's `HANDLE`, `HRESULT` and `BOOL` newtypes) | that scalar | Passed and returned directly |
| Other structs and unions by value (`POINT`, `RECT`, `GUID`) | `i32` | Pointer to a caller-owned copy; a struct result becomes a hidden first `i32` parameter pointing at the result slot |
| Variadic arguments (`"C"` imports with `...`) | `i32` | The last parameter points to a buffer holding the variadic arguments |
| Callbacks (`WNDPROC`, `PTP_SIMPLE_CALLBACK`) | `i32` | Index into the module's function table. `wasm-ld` keeps the table internal by default; link with `-C link-arg=--export-table` so the host can call callbacks |
| COM interface pointers | `i32` | Points to the vtable pointer; vtable slots are function-table indices |

## Layout

`windows-bindgen` emits `target_arch = "wasm32"` wherever it emits `target_arch = "x86"`, so
`wasm32` uses the 32-bit Windows definitions (for example `SYSTEM_BASIC_INFORMATION` is 44 bytes
with `NumberOfProcessors` at offset 40). Both targets have 4-byte pointers and 8-byte aligned
64-bit scalars, so their `repr(C)` layouts agree. `wasm64` gets no architecture-specific items.

## Crate behaviour

| Crate | wasm32 | Behaviour |
|-|-|-|
| `windows-link` | yes | `link!` emits wasm imports |
| `windows-sys` | yes | Every API is a host import; x86 layouts |
| `windows-result` | yes | `HRESULT::from_thread`, `Error::from_thread` and `WIN32_ERROR::from_thread` import `kernel32.dll` `GetLastError`; `HRESULT::message` returns `0x%08x` text (no `FormatMessageW`); no `IErrorInfo` |
| `windows-strings` | yes | `HSTRING` uses the Rust allocator; no `BSTR` |
| `windows-core` | yes | COM objects implemented in Rust work without imports; `CoCreateInstance`, WinRT activation, `BSTR` and `Array` are Windows-only |
| `windows-numerics`, `windows-collections` | yes | Pure Rust |
| `windows-threading`, `windows-version`, `windows-registry` | yes | Host imports from `kernel32.dll`, `ntdll.dll` and `advapi32.dll` |
| `windows`, `windows-future`, `windows-time`, `windows-services`, `cppwinrt` | no | Gated with `#![cfg(windows)]`; empty on WebAssembly |

`wasm32-wasip2` builds the same crates. Linking a component that references Win32 imports is not
covered here, so modules that call Win32 imports should target `wasm32-unknown-unknown`.
`wasm.yml` checks both targets, runs `test_wasm` under Wasmtime on `wasm32-wasip2`, and runs
`crates/samples/wasm/system-info/host.js` against the `wasm32-unknown-unknown` build of the
sample.
