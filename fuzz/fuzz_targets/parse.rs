#![no_main]
use libfuzzer_sys::fuzz_target;
use oxiri::IriRef;
use std::str;

fuzz_target!(|data: &[u8]| {
    let Ok(s) = str::from_utf8(data) else {
        return;
    };
    let unchecked = IriRef::parse_unchecked(s);
    let Ok(iri) = IriRef::parse(s) else {
        return;
    };
    assert_eq!(iri, s);
    assert_eq!(iri, unchecked);
    assert_eq!(iri.scheme(), unchecked.scheme());
    assert_eq!(iri.authority(), unchecked.authority());
    assert_eq!(iri.path(), unchecked.path());
    assert_eq!(iri.query(), unchecked.query());
    assert_eq!(iri.fragment(), unchecked.fragment());
});
