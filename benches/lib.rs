use codspeed_criterion_compat::{criterion_group, criterion_main, BenchmarkId, Criterion};
use oxiri::{Iri, IriRef};
use std::fs;
use std::hint::black_box;

const ABS_EXAMPLES: &[&str] = &[
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
];

fn absolute_url_datasets() -> [(&'static str, Vec<String>); 4] {
    [
        (
            "top_100",
            parse_url_file("benches/url-various-datasets/top100/top100.txt"),
        ),
        (
            "wikipedia",
            parse_url_file("benches/url-various-datasets/wikipedia/wikipedia_100k.txt"),
        ),
        (
            "kasztp",
            parse_url_file("benches/url-various-datasets/others/kasztp.txt"),
        ),
        (
            "userbait",
            parse_url_file("benches/url-various-datasets/others/userbait.txt"),
        ),
    ]
}

fn parse_url_file(path: &str) -> Vec<String> {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .filter_map(|s| {
            let url = s.trim();
            if Iri::parse(url).is_ok() {
                Some(url.to_owned())
            } else {
                None
            }
        })
        .collect()
}

fn relative_url_datasets() -> [(&'static str, Vec<(String, String)>); 1] {
    [(
        "cc_10k",
        parse_relative_url_file("benches/relative_urls_cc_10k.csv"),
    )]
}

fn parse_relative_url_file(path: &str) -> Vec<(String, String)> {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .skip(1)
        .map(|s| {
            let (base, relative) = s.split_once(',').unwrap();
            (base.trim().to_owned(), relative.trim().to_owned())
        })
        .collect()
}

fn iri_parse(c: &mut Criterion) {
    c.bench_function("Iri::parse", |b| {
        b.iter(|| {
            for iri in ABS_EXAMPLES {
                Iri::parse(black_box(*iri)).unwrap();
            }
        })
    });
    for (name, urls) in absolute_url_datasets() {
        let urls = urls.iter().map(String::as_str).collect::<Vec<_>>();
        c.bench_with_input(BenchmarkId::new("Iri::parse", name), &urls, |b, urls| {
            b.iter(|| {
                for url in urls {
                    Iri::parse(black_box(*url)).unwrap();
                }
            })
        });
    }
    c.bench_function("Iri::parse_unchecked", |b| {
        b.iter(|| {
            for iri in ABS_EXAMPLES {
                Iri::parse_unchecked(black_box(*iri));
            }
        })
    });
    for (name, urls) in absolute_url_datasets() {
        let urls = urls.iter().map(String::as_str).collect::<Vec<_>>();
        c.bench_with_input(
            BenchmarkId::new("Iri::parse_unchecked", name),
            &urls,
            |b, urls| {
                b.iter(|| {
                    for url in urls {
                        Iri::parse_unchecked(black_box(*url));
                    }
                })
            },
        );
    }
}

fn iri_parse_relative(c: &mut Criterion) {
    c.bench_function("IriRef::parse", |b| {
        b.iter(|| {
            for iri in ABS_EXAMPLES {
                IriRef::parse(black_box(*iri)).unwrap();
            }
        })
    });
    for (name, urls) in absolute_url_datasets() {
        let urls = urls.iter().map(String::as_str).collect::<Vec<_>>();
        c.bench_with_input(BenchmarkId::new("IriRef::parse", name), &urls, |b, urls| {
            b.iter(|| {
                for url in urls {
                    IriRef::parse(black_box(*url)).unwrap();
                }
            })
        });
    }
    for (name, urls) in relative_url_datasets() {
        let urls = urls.iter().map(|(_, url)| url.as_str()).collect::<Vec<_>>();
        c.bench_with_input(BenchmarkId::new("IriRef::parse", name), &urls, |b, urls| {
            b.iter(|| {
                for url in urls {
                    IriRef::parse(black_box(*url)).unwrap();
                }
            })
        });
    }
    c.bench_function("IriRef::parse_unchecked", |b| {
        b.iter(|| {
            for iri in ABS_EXAMPLES {
                IriRef::parse_unchecked(black_box(*iri));
            }
        })
    });
    for (name, urls) in absolute_url_datasets() {
        let urls = urls.iter().map(String::as_str).collect::<Vec<_>>();
        c.bench_with_input(
            BenchmarkId::new("IriRef::parse_unchecked", name),
            &urls,
            |b, urls| {
                b.iter(|| {
                    for url in urls {
                        IriRef::parse_unchecked(black_box(*url));
                    }
                })
            },
        );
    }
    for (name, urls) in relative_url_datasets() {
        let urls = urls.iter().map(|(_, url)| url.as_str()).collect::<Vec<_>>();
        c.bench_with_input(
            BenchmarkId::new("IriRef::parse_unchecked", name),
            &urls,
            |b, urls| {
                b.iter(|| {
                    for url in urls {
                        IriRef::parse_unchecked(black_box(*url));
                    }
                })
            },
        );
    }
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
                black_box(base)
                    .resolve_into(&IriRef::parse(black_box(*relative)).unwrap(), &mut buf)
                    .unwrap();
            }
        })
    });
    for (name, urls) in relative_url_datasets() {
        let urls = urls
            .iter()
            .map(|(base, relative)| (Iri::parse_unchecked(base.as_str()), relative.as_str()))
            .collect::<Vec<_>>();

        c.bench_with_input(
            BenchmarkId::new("Iri::resolve_into", name),
            &urls,
            |b, urls| {
                b.iter(|| {
                    for (base, relative) in urls {
                        buf.clear();
                        base.resolve_into(&IriRef::parse(black_box(*relative)).unwrap(), &mut buf)
                            .unwrap();
                    }
                })
            },
        );
    }
    c.bench_function("Iri::resolve_into_unchecked", |b| {
        b.iter(|| {
            for relative in examples.iter() {
                buf.clear();
                black_box(base).resolve_into_unchecked(
                    &IriRef::parse_unchecked(black_box(*relative)),
                    &mut buf,
                );
            }
        })
    });
    for (name, urls) in relative_url_datasets() {
        let urls = urls
            .iter()
            .map(|(base, relative)| (Iri::parse_unchecked(base.as_str()), relative.as_str()))
            .collect::<Vec<_>>();
        c.bench_with_input(
            BenchmarkId::new("Iri::resolve_into_unchecked", name),
            &urls,
            |b, urls| {
                b.iter(|| {
                    for (base, relative) in urls {
                        buf.clear();
                        base.resolve_into_unchecked(
                            &IriRef::parse_unchecked(black_box(*relative)),
                            &mut buf,
                        );
                    }
                })
            },
        );
    }
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
    .map(|iri| base.resolve(&IriRef::parse(iri).unwrap()).unwrap())
    .collect::<Vec<_>>();

    c.bench_function("Iri::relativize", |b| {
        b.iter(|| {
            for iri in &examples {
                black_box(base).relativize(black_box(iri)).unwrap();
            }
        })
    });
    for (name, urls) in relative_url_datasets() {
        let urls = urls
            .iter()
            .map(|(base, relative)| {
                let base = Iri::parse_unchecked(base.as_str());
                let resolved = base.resolve_unchecked(&IriRef::parse_unchecked(relative.as_str()));
                (base, resolved)
            })
            .collect::<Vec<_>>();
        c.bench_with_input(
            BenchmarkId::new("Iri::relativize", name),
            &urls,
            |b, urls| {
                b.iter(|| {
                    for (base, resolved) in urls {
                        base.relativize(resolved).unwrap();
                    }
                })
            },
        );
    }
}

criterion_group!(
    iri,
    iri_parse,
    iri_parse_relative,
    iri_resolve,
    iri_relativize
);

criterion_main!(iri);
