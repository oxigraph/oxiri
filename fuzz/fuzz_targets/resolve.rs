#![no_main]
use libfuzzer_sys::fuzz_target;
use oxiri::IriRef;
use std::str;

fuzz_target!(|data: &[u8]| {
    let base = IriRef::parse("http://a/b/c/d;p?q").unwrap();
    if let Ok(s) = str::from_utf8(data) {
        let valid_result = base.resolve(s);

        // We check that unchecked resolving gives the same result
        let unchecked_result = base.resolve_unchecked(s);
        if let Ok(valid) = valid_result {
            assert_eq!(valid, unchecked_result.unwrap());
        }
    }
});
