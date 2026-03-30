#![no_main]
use libfuzzer_sys::fuzz_target;
use oxiri::{Iri, IriRef};
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

    let abs_unchecked = Iri::parse_unchecked(s);
    let Ok(abs_iri) = Iri::parse(s) else {
        return;
    };
    assert_eq!(abs_iri, s);
    assert_eq!(abs_iri, abs_unchecked);
    assert_eq!(abs_iri.scheme(), abs_unchecked.scheme());
    assert_eq!(abs_iri.authority(), abs_unchecked.authority());
    assert_eq!(abs_iri.path(), abs_unchecked.path());
    assert_eq!(abs_iri.query(), abs_unchecked.query());
    assert_eq!(abs_iri.fragment(), abs_unchecked.fragment());
    assert_eq!(iri, abs_iri);
    assert_eq!(iri.scheme(), Some(abs_iri.scheme()));
    assert_eq!(iri.authority(), abs_iri.authority());
    assert_eq!(iri.path(), abs_iri.path());
    assert_eq!(iri.query(), abs_iri.query());
    assert_eq!(iri.fragment(), abs_iri.fragment());
});
