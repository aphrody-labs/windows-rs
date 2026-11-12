use windows_strings::*;

#[test]
fn hstring() {
    let value = HSTRING::from("wasm");
    assert_eq!(value, "wasm");
    assert_eq!(value.to_string_lossy(), "wasm");
    assert_eq!(h!("wasm"), &value);
    assert_eq!(HSTRING::new(), "");
}
