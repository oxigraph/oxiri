#![no_main]
use libfuzzer_sys::fuzz_target;
use oxiri::Iri;
use std::str;

fuzz_target!(|data: &[u8]| {
    let parts = data.split(|c| *c == b'\0').collect::<Vec<_>>();
    let Ok([relative, base]) = <[&[u8]; 2]>::try_from(parts) else {
        return;
    };
    let Ok(relative) = str::from_utf8(relative) else {
        return;
    };
    let Ok(base) = str::from_utf8(base) else {
        return;
    };
    let Ok(base) = Iri::parse(base) else {
        return;
    };
    let unchecked = base.resolve_unchecked(relative);
    let Ok(valid) = base.resolve(relative) else {
        return;
    };
    // We check that unchecked resolving gives the same result
    assert_eq!(valid, unchecked);
    assert_eq!(valid.scheme(), unchecked.scheme());
    assert_eq!(valid.authority(), unchecked.authority());
    assert_eq!(valid.path(), unchecked.path());
    assert_eq!(valid.query(), unchecked.query());
    assert_eq!(valid.fragment(), unchecked.fragment());
});
