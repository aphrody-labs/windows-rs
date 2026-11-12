//! COM objects implemented in Rust need no host imports, so these run under any wasm runtime.

#![expect(non_snake_case)]

use windows_core::*;

#[interface("d888acaa-fb67-46a4-bb35-87cb37db5830")]
unsafe trait ITest: IUnknown {
    unsafe fn Value(&self) -> i32;
}

#[implement(ITest)]
struct Test(i32);

impl ITest_Impl for Test_Impl {
    unsafe fn Value(&self) -> i32 {
        self.0
    }
}

#[test]
fn query_interface() {
    let object = ComObject::new(Test(42));
    let unknown: IUnknown = object.to_interface();
    let test: ITest = unknown.cast().expect("QueryInterface for ITest");
    assert_eq!(unsafe { test.Value() }, 42);
}

#[test]
fn guid() {
    let guid = GUID::from_u128(0xd888acaa_fb67_46a4_bb35_87cb37db5830);
    assert_eq!(guid, ITest::IID);
    assert_eq!(format!("{guid:?}"), "D888ACAA-FB67-46A4-BB35-87CB37DB5830");
}
