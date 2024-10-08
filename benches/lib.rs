use codspeed_criterion_compat::{criterion_group, criterion_main, Criterion};
use oxiri::{Iri, IriRef};

fn abs_examples() -> &'static [&'static str] {
    &[
        "file://foo",
        "ftp://ftp.is.co.za/rfc/rfc1808.txt",
        "http://www.ietf.org/rfc/rfc2396.txt",
        //"ldap://[2001:db8::7]/c=GB?objectClass?one",
        "mailto:John.Doe@example.com",
        "news:comp.infosystems.www.servers.unix",
        "tel:+1-816-555-1212",
        "telnet://192.0.2.16:80/",
        "urn:oasis:names:specification:docbook:dtd:xml:4.1.2",
        "http://example.com",
        "http://example.com/",
        "http://example.com/foo",
        "http://example.com/foo/bar",
        "http://example.com/foo/bar/",
        "http://example.com/foo/bar?q=1&r=2",
        "http://example.com/foo/bar/?q=1&r=2",
        "http://example.com#toto",
        "http://example.com/#toto",
        "http://example.com/foo#toto",
        "http://example.com/foo/bar#toto",
        "http://example.com/foo/bar/#toto",
        "http://example.com/foo/bar?q=1&r=2#toto",
        "http://example.com/foo/bar/?q=1&r=2#toto",
        "http://example.com/foo/bar/.././baz",
    ]
}

fn iri_parse(c: &mut Criterion) {
    c.bench_function("Iri::parse", |b| {
        b.iter(|| {
            for iri in abs_examples().iter() {
                Iri::parse(*iri).unwrap();
            }
        })
    });
    c.bench_function("Iri::parse_unchecked", |b| {
        b.iter(|| {
            for iri in abs_examples().iter() {
                Iri::parse_unchecked(*iri);
            }
        })
    });
}

fn iri_parse_relative(c: &mut Criterion) {
    c.bench_function("IriRef::parse", |b| {
        b.iter(|| {
            for iri in abs_examples().iter() {
                IriRef::parse(*iri).unwrap();
            }
        })
    });
    c.bench_function("IriRef::parse_unchecked", |b| {
        b.iter(|| {
            for iri in abs_examples().iter() {
                IriRef::parse_unchecked(*iri);
            }
        })
    });
}

fn iri_resolve(c: &mut Criterion) {
    let examples = [
        "g:h",
        "g",
        "g/",
        "/g",
        "//g",
        "?y",
        "g?y",
        "#s",
        "g#s",
        "g?y#s",
        ";x",
        "g;x",
        "g;x?y#s",
        "",
        ".",
        "./",
        "./g",
        "..",
        "../",
        "../g",
        "../..",
        "../../",
        "../../g",
        "../../../g",
        "../../../../g",
        "/./g",
        "/../g",
        "g.",
        ".g",
        "g..",
        "..g",
        "./../g",
        "./g/.",
        "g/./h",
        "g/../h",
        "g;x=1/./y",
        "g;x=1/../y",
        "g?y/./x",
        "g?y/../x",
        "g#s/./x",
        "g#s/../x",
        "http:g",
        "./g:h",
    ];

    let base = Iri::parse("http://a/b/c/d;p?q").unwrap();

    let mut buf = String::new();
    c.bench_function("Iri::resolve_into", |b| {
        b.iter(|| {
            for relative in examples.iter() {
                buf.clear();
                base.resolve_into(relative, &mut buf).unwrap();
            }
        })
    });
    c.bench_function("Iri::resolve_into_unchecked", |b| {
        b.iter(|| {
            for relative in examples.iter() {
                buf.clear();
                base.resolve_into_unchecked(relative, &mut buf);
            }
        })
    });
}

fn iri_relativize(c: &mut Criterion) {
    let base = Iri::parse("http://a/b/c/d;p?q").unwrap();
    let examples = [
        "g:h",
        "g",
        "g/",
        "/g",
        "//g",
        "?y",
        "g?y",
        "#s",
        "g#s",
        "g?y#s",
        ";x",
        "g;x",
        "g;x?y#s",
        "",
        ".",
        "./",
        "./g",
        "..",
        "../",
        "../g",
        "../..",
        "../../",
        "../../g",
        "../../../g",
        "../../../../g",
        "/./g",
        "/../g",
        "g.",
        ".g",
        "g..",
        "..g",
        "./../g",
        "./g/.",
        "g/./h",
        "g/../h",
        "g;x=1/./y",
        "g;x=1/../y",
        "g?y/./x",
        "g?y/../x",
        "g#s/./x",
        "g#s/../x",
        "http:g",
        "./g:h",
    ]
    .into_iter()
    .map(|iri| base.resolve(iri).unwrap())
    .collect::<Vec<_>>();

    c.bench_function("Iri::relativize", |b| {
        b.iter(|| {
            for iri in &examples {
                base.relativize(iri).unwrap();
            }
        })
    });
}

criterion_group!(
    iri,
    iri_parse,
    iri_parse_relative,
    iri_resolve,
    iri_relativize
);

criterion_main!(iri);
