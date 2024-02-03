#![no_main]
use libfuzzer_sys::fuzz_target;
use oxiri::IriRef;
use std::str;

fuzz_target!(|data: &[u8]| {
    let base = IriRef::parse("http://a/b/c/d;p?q").unwrap();
    if let Ok(s) = str::from_utf8(data) {
        let unchecked = base.resolve_unchecked(s);
        if let Ok(valid) = base.resolve(s) {
            // We check that unchecked resolving gives the same result
            assert_eq!(valid, unchecked);
            assert_eq!(valid.scheme(), unchecked.scheme());
            assert_eq!(valid.authority(), unchecked.authority());
            assert_eq!(valid.path(), unchecked.path());
            assert_eq!(valid.query(), unchecked.query());
            assert_eq!(valid.fragment(), unchecked.fragment());
        }
    }
});
