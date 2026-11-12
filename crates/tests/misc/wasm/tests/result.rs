use windows_result::*;

const E_FAIL: HRESULT = HRESULT(0x80004005_u32 as _);
const S_OK: HRESULT = HRESULT(0);

#[test]
fn hresult() {
    assert!(E_FAIL.is_err());
    assert!(S_OK.is_ok());
    assert_eq!(Error::from(E_FAIL).code(), E_FAIL);
}

#[test]
fn ok() {
    assert_eq!(S_OK.ok(), Ok(()));
    assert_eq!(E_FAIL.ok().unwrap_err().code(), E_FAIL);
}
