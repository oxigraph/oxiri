#![no_main]
use libfuzzer_sys::fuzz_target;
use oxiri::IriRef;
use std::str;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = str::from_utf8(data) {
        let _ = IriRef::parse(s);
    }
});
