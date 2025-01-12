#![no_main]

use libfuzzer_sys::fuzz_target;
use oxiri::Iri;
use std::str;

fuzz_target!(|data: &[u8]| {
    let parts = data.split(|c| *c == b'\0').collect::<Vec<_>>();
    let Ok([iri, base]) = <[&[u8]; 2]>::try_from(parts) else {
        return;
    };
    let Ok(base) = str::from_utf8(base) else {
        return;
    };
    let Ok(base) = Iri::parse(base) else {
        return;
    };
    let Ok(iri) = str::from_utf8(iri) else {
        return;
    };
    let (absolute, was_relative) = if let Ok(iri) = Iri::parse(iri.to_string()) {
        (iri, false)
    } else if let Ok(iri) = base.resolve(iri) {
        (iri, true)
    } else {
        return;
    };
    let base = Iri::parse_unchecked(base);
    match base.relativize(&absolute) {
        Ok(relative) => {
            let from_relative = base.resolve(relative.as_str()).unwrap();
            assert_eq!(
                absolute, from_relative,
                "Resolving {relative} computed from {absolute} with base {base} gives {from_relative}"
            );
        }
        Err(_) => {
            // It is always possible to relativize an IRI that has been resolved
            assert!(!was_relative || base.path().contains('.'), "It should be always possible to relativize a former relative IRI {iri} (base {base})");
        }
    }
});
