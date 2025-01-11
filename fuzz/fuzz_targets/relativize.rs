#![no_main]

use libfuzzer_sys::fuzz_target;
use oxiri::Iri;
use std::str;

fuzz_target!(|data: &[u8]| {
    let parts = data.split(|c| *c == b'\0').collect::<Vec<_>>();
    let Ok([absolute, base]) = <[&[u8]; 2]>::try_from(parts) else {
        return;
    };
    let Ok(absolute) = str::from_utf8(absolute) else {
        return;
    };
    let Ok(absolute) = Iri::parse(absolute) else {
        return;
    };
    let Ok(base) = str::from_utf8(base) else {
        return;
    };
    let Ok(base) = Iri::parse(base) else {
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
            // We make sure it's not possible to relativize
            assert_ne!(base.resolve(&absolute.as_str()).unwrap(), absolute);
        }
    }
});
