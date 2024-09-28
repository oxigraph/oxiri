#![no_main]
use libfuzzer_sys::fuzz_target;
use oxiri::Iri;
use std::str;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = str::from_utf8(data) {
        let Ok(iri) = Iri::parse(s) else {
            return;
        };
        for base in ["http://a/b/c/d;p?q", "http://a/", "http://a", "http:"] {
            let base = Iri::parse(base).unwrap();
            match base.relativize(&iri) {
                Ok(relative) => {
                    let from_relative = base.resolve(relative.as_str()).unwrap();
                    assert_eq!(
                        iri, from_relative,
                        "Resolving {relative} computed from {iri} with base {base} gives {from_relative}"
                    );
                }
                Err(_) => {
                    // We make sure it's not possible to relativize
                    assert_ne!(base.resolve(&iri.as_str()).unwrap(), iri);
                }
            }
        }
    }
});
