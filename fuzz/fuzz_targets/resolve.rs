#![no_main]
use libfuzzer_sys::fuzz_target;
use oxiri::IriRef;
use std::str;

fuzz_target!(|data: &[u8]| {
    let base = IriRef::parse("http://a/b/c/d;p?q").unwrap();
    if let Ok(s) = str::from_utf8(data) {
        let _ = base.resolve(s);
    }
});
