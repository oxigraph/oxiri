#![no_main]
use libfuzzer_sys::fuzz_target;
use oxiri::{Iri, IriRef};
use std::str;

fuzz_target!(|data: &[u8]| {
    let parts = data.split(|c| *c == b'\0').collect::<Vec<_>>();
    let Ok([relative, base]) = <[&[u8]; 2]>::try_from(parts) else {
        return;
    };
    let Ok(relative) = str::from_utf8(relative) else {
        return;
    };
    let Ok(relative) = IriRef::parse(relative) else {
        return;
    };
    let Ok(base) = str::from_utf8(base) else {
        return;
    };
    let Ok(base) = IriRef::parse(base) else {
        return;
    };
    let unchecked = base.resolve_unchecked(&relative);
    let Ok(valid) = base.resolve(&relative) else {
        return;
    };

    // We check that unchecked resolving gives the same result
    assert_eq!(valid, unchecked);
    assert_eq!(valid.scheme(), unchecked.scheme());
    assert_eq!(valid.authority(), unchecked.authority());
    assert_eq!(valid.path(), unchecked.path());
    assert_eq!(valid.query(), unchecked.query());
    assert_eq!(valid.fragment(), unchecked.fragment());

    // We check that the parts are valid
    let reparsed = IriRef::parse(valid.as_str()).unwrap();
    assert_eq!(valid.scheme(), reparsed.scheme());
    assert_eq!(valid.authority(), reparsed.authority());
    assert_eq!(valid.path(), reparsed.path());
    assert_eq!(valid.query(), reparsed.query());
    assert_eq!(valid.fragment(), reparsed.fragment());

    // We check associativity with relative IRIs
    if !base.is_absolute() {
        let root_base = Iri::parse_unchecked("http://a/b/c/d;p?q");
        assert_eq!(
            root_base
                .resolve_unchecked(&base)
                .resolve_unchecked(&relative),
            root_base.resolve_unchecked(&base.resolve_unchecked(&relative)),
        );
    }
});
