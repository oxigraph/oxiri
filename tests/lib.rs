#![allow(clippy::eq_op)]
use oxiri::{Iri, IriRef};
#[cfg(feature = "serde")]
use serde_test::{assert_de_tokens, assert_de_tokens_error, assert_tokens, Token};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[test]
fn test_parsing() {
    let examples = [
        "file://foo",
        "ftp://ftp.is.co.za/rfc/rfc1808.txt",
        "http://www.ietf.org/rfc/rfc2396.txt",
        "ldap://[2001:db8::7]/c=GB?objectClass?one",
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
        "http://a.example/AZaz\u{00C0}\u{00D6}\u{00D8}\u{00F6}\u{00F8}\u{02FF}\u{0370}\u{037D}\u{037F}\u{1FFF}\u{200C}\u{200D}\u{2070}\u{218F}\u{2C00}\u{2FEF}\u{3001}\u{D7FF}\u{FA0E}\u{FDCF}\u{FDF0}\u{FFEF}\u{10000}\u{EFFFD}",
        "http://a.example/?AZaz\u{E000}\u{F8FF}\u{F0000}\u{FFFFD}\u{100000}\u{10FFFD}\u{00C0}\u{00D6}\u{00D8}\u{00F6}\u{00F8}\u{02FF}\u{0370}\u{037D}\u{037F}\u{1FFF}\u{200C}\u{200D}\u{2070}\u{218F}\u{2C00}\u{2FEF}\u{3001}\u{D7FF}\u{FA0E}\u{FDCF}\u{FDF0}\u{FFEF}\u{10000}\u{EFFFD}",
        "http://[va.12z]",
        "http://[vff.B]",
        "http://[V0.a]"
    ];

    for e in examples {
        let unchecked = Iri::parse_unchecked(e);
        assert_eq!(unchecked, e);
        match Iri::parse(e) {
            Ok(iri) => {
                assert_eq!(unchecked, iri);
                assert_eq!(unchecked.scheme(), iri.scheme());
                assert_eq!(unchecked.authority(), iri.authority());
                assert_eq!(unchecked.path(), iri.path());
                assert_eq!(unchecked.query(), iri.query());
                assert_eq!(unchecked.fragment(), iri.fragment());
            }
            Err(error) => panic!("{} on IRI {}", error, e),
        }
    }
}

#[test]
fn test_relative_parsing() {
    // From https://sourceforge.net/projects/foursuite/ under Apache License

    let examples = [
        "file:///foo/bar",
        "mailto:user@host?subject=blah",
        "dav:",   // empty opaque part / rel-path allowed by RFC 2396bis
        "about:", // empty opaque part / rel-path allowed by RFC 2396bis
        //
        // the following test cases are from a Perl script by David A. Wheeler
        // at http://www.dwheeler.com/secure-programs/url.pl
        "http://www.yahoo.com",
        "http://www.yahoo.com/",
        "http://1.2.3.4/",
        "http://www.yahoo.com/stuff",
        "http://www.yahoo.com/stuff/",
        "http://www.yahoo.com/hello%20world/",
        "http://www.yahoo.com?name=obi",
        "http://www.yahoo.com?name=obi+wan&status=jedi",
        "http://www.yahoo.com?onery",
        "http://www.yahoo.com#bottom",
        "http://www.yahoo.com/yelp.html#bottom",
        "https://www.yahoo.com/",
        "ftp://www.yahoo.com/",
        "ftp://www.yahoo.com/hello",
        "demo.txt",
        "demo/hello.txt",
        "demo/hello.txt?query=hello#fragment",
        "/cgi-bin/query?query=hello#fragment",
        "/demo.txt",
        "/hello/demo.txt",
        "hello/demo.txt",
        "/",
        "",
        "#",
        "#here",
        // Wheeler"s script says these are invalid, but they aren"t
        "http://www.yahoo.com?name=%00%01",
        "http://www.yaho%6f.com",
        "http://www.yahoo.com/hello%00world/",
        "http://www.yahoo.com/hello+world/",
        "http://www.yahoo.com?name=obi&",
        "http://www.yahoo.com?name=obi&type=",
        "http://www.yahoo.com/yelp.html#",
        "//",
        // the following test cases are from a Haskell program by Graham Klyne
        // at http://www.ninebynine.org/Software/HaskellUtils/Network/URITest.hs
        "http://example.org/aaa/bbb#ccc",
        "mailto:local@domain.org",
        "mailto:local@domain.org#frag",
        "HTTP://EXAMPLE.ORG/AAA/BBB#CCC",
        "//example.org/aaa/bbb#ccc",
        "/aaa/bbb#ccc",
        "bbb#ccc",
        "#ccc",
        "#",
        "A'C",
        //-- escapes
        "http://example.org/aaa%2fbbb#ccc",
        "http://example.org/aaa%2Fbbb#ccc",
        "%2F",
        "?%2F",
        "#?%2F",
        "aaa%2Fbbb",
        //-- ports
        "http://example.org:80/aaa/bbb#ccc",
        "http://example.org:/aaa/bbb#ccc",
        "http://example.org./aaa/bbb#ccc",
        "http://example.123./aaa/bbb#ccc",
        //-- bare authority
        "http://example.org",
        //-- IPv6 literals (from RFC2732):
        "http://[FEDC:AA98:7654:3210:FEDC:AA98:7654:3210]:80/index.html",
        "http://[1080:0:0:0:8:800:200C:417A]/index.html",
        "http://[3ffe:2a00:100:7031::1]",
        "http://[1080::8:800:200C:417A]/foo",
        "http://[::192.9.5.5]/ipng",
        "http://[::FFFF:129.144.52.38]:80/index.html",
        "http://[2010:836B:4179::836B:4179]",
        "//[2010:836B:4179::836B:4179]",
        //-- Random other things that crop up
        "http://example/Andr&#567;",
        "file:///C:/DEV/Haskell/lib/HXmlToolbox-3.01/examples/",
        // iprivate characters are allowed in query
        "http://a/?\u{E000}",
        "?\u{E000}",
    ];

    let base = Iri::parse("http://a/b/c/d;p?q").unwrap();
    for e in examples {
        let unchecked = IriRef::parse_unchecked(e);
        assert_eq!(unchecked, e);
        match IriRef::parse(e) {
            Ok(iri) => {
                assert_eq!(unchecked, iri);
                assert_eq!(unchecked.scheme(), iri.scheme());
                assert_eq!(unchecked.authority(), iri.authority());
                assert_eq!(unchecked.path(), iri.path());
                assert_eq!(unchecked.query(), iri.query());
                assert_eq!(unchecked.fragment(), iri.fragment());
            }
            Err(error) => panic!("{} on relative IRI {}", error, e),
        }
        match base.resolve(e) {
            Ok(iri) => assert_eq!(base.resolve_unchecked(e), iri),
            Err(error) => panic!("{} on relative IRI {}", error, e),
        }
    }
}

#[test]
fn test_wrong_relative_parsing() {
    // From https://sourceforge.net/projects/foursuite/ under Apache License

    let examples = [
        "beepbeep\x07\x07",
        "\n",
        // "::", // not OK, per Roy Fielding on the W3C uri list on 2004-04-01
        //
        // the following test cases are from a Perl script by David A. Wheeler
        // at http://www.dwheeler.com/secure-programs/url.pl
        "http://www yahoo.com",
        "http://www.yahoo.com/hello world/",
        "http://www.yahoo.com/yelp.html#\"",
        //
        // the following test cases are from a Haskell program by Graham Klyne
        // at http://www.ninebynine.org/Software/HaskellUtils/Network/URITest.hs
        "[2010:836B:4179::836B:4179]",
        " ",
        "%",
        "A%Z",
        "%ZZ",
        "%AZ",
        "A C",
        // "A'C",
        "A`C",
        "A<C",
        "A>C",
        "A^C",
        "A\\C",
        "A{C",
        "A|C",
        "A}C",
        "A[C",
        "A]C",
        "A[**]C",
        "http://[xyz]/",
        "http://]/",
        "http://example.org/[2010:836B:4179::836B:4179]",
        "http://example.org/abc#[2010:836B:4179::836B:4179]",
        "http://example.org/xxx/[qwerty]#a[b]",
        // from a post to the W3C uri list on 2004-02-17
        "http://w3c.org:80path1/path2",
        // relative IRIs do not accept colon in the first path segment
        ":a/b",
        // iprivate characters are not allowed in path not in fragment
        "http://example.com/\u{E000}",
        "\u{E000}",
        "http://example.com/#\u{E000}",
        "#\u{E000}",
        // bad characters
        "//\u{FFFF}",
        "?\u{FFFF}",
        "/\u{0000}",
        "?\u{0000}",
        "#\u{0000}",
        "/\u{E000}",
        "/\u{F8FF}",
        "/\u{F0000}",
        "/\u{FFFFD}",
        "/\u{100000}",
        "/\u{10FFFD}",
        "?\u{FDEF}",
        "?\u{FFFF}",
        "/\u{FDEF}",
        "/\u{FFFF}",
        "/\u{1FFFF}",
        "/\u{2FFFF}",
        "/\u{3FFFF}",
        "/\u{4FFFF}",
        "/\u{5FFFF}",
        "/\u{6FFFF}",
        "/\u{7FFFF}",
        "/\u{8FFFF}",
        "/\u{9FFFF}",
        "/\u{AFFFF}",
        "/\u{BFFFF}",
        "/\u{CFFFF}",
        "/\u{DFFFF}",
        "/\u{EFFFF}",
        "/\u{FFFFF}",
        // bad host
        "http://[/",
        "http://[::1]a/",
        // fuzzing bugs
        "//͏@[]",
        "//@@",
        "$:",
        "-:",
        ":",
        // IPvFuture
        "http://[]",
        "http://[a]",
        "http://[vz]",
        "http://[v11]",
        "http://[v1.]",
        "http://[v.a]",
        "http://[v1.@]",
        "http://[v1.%01]",
        "//[v1.ւ]",
    ];

    let base = Iri::parse("http://a/b/c/d;p?q").unwrap();
    for e in examples {
        let result = base.resolve(e);
        assert!(result.is_err(), "{} is wrongly considered valid", e);
    }
}

#[test]
fn test_wrong_relative_parsing_on_scheme() {
    let examples = [".///C:::"];

    let base = Iri::parse("x:").unwrap();
    for e in examples {
        let result = base.resolve(e);
        assert!(result.is_err(), "{} is wrongly considered valid", e);
    }
}

#[test]
fn test_resolve_relative_iri() {
    // From https://sourceforge.net/projects/foursuite/ under Apache License

    let examples = [
        // http://lists.w3.org/Archives/Public/uri/2004Feb/0114.html
        ("/.", "http://a/b/c/d;p?q", "http://a/"),
        ("/.foo", "http://a/b/c/d;p?q", "http://a/.foo"),
        (".foo", "http://a/b/c/d;p?q", "http://a/b/c/.foo"),
        // http://gbiv.com/protocols/uri/test/rel_examples1.html
        // examples from RFC 2396
        ("g:h", "http://a/b/c/d;p?q", "g:h"),
        ("g", "http://a/b/c/d;p?q", "http://a/b/c/g"),
        ("./g", "http://a/b/c/d;p?q", "http://a/b/c/g"),
        ("g/", "http://a/b/c/d;p?q", "http://a/b/c/g/"),
        ("/g", "http://a/b/c/d;p?q", "http://a/g"),
        ("//g", "http://a/b/c/d;p?q", "http://g"),
        // changed with RFC 2396bis
        //("?y"      , "http://a/b/c/d;p?q", "http://a/b/c/d;p?y"),
        ("?y", "http://a/b/c/d;p?q", "http://a/b/c/d;p?y"),
        ("g?y", "http://a/b/c/d;p?q", "http://a/b/c/g?y"),
        // changed with RFC 2396bis
        //("#s"      , "http://a/b/c/d;p?q", CURRENT_DOC_URI + "#s"),
        ("#s", "http://a/b/c/d;p?q", "http://a/b/c/d;p?q#s"),
        ("g#s", "http://a/b/c/d;p?q", "http://a/b/c/g#s"),
        ("g?y#s", "http://a/b/c/d;p?q", "http://a/b/c/g?y#s"),
        (";x", "http://a/b/c/d;p?q", "http://a/b/c/;x"),
        ("g;x", "http://a/b/c/d;p?q", "http://a/b/c/g;x"),
        ("g;x?y#s", "http://a/b/c/d;p?q", "http://a/b/c/g;x?y#s"),
        // changed with RFC 2396bis
        //(""        , "http://a/b/c/d;p?q", CURRENT_DOC_URI),
        ("", "http://a/b/c/d;p?q", "http://a/b/c/d;p?q"),
        (".", "http://a/b/c/d;p?q", "http://a/b/c/"),
        ("./", "http://a/b/c/d;p?q", "http://a/b/c/"),
        ("..", "http://a/b/c/d;p?q", "http://a/b/"),
        ("../", "http://a/b/c/d;p?q", "http://a/b/"),
        ("../g", "http://a/b/c/d;p?q", "http://a/b/g"),
        ("../..", "http://a/b/c/d;p?q", "http://a/"),
        ("../../", "http://a/b/c/d;p?q", "http://a/"),
        ("../../g", "http://a/b/c/d;p?q", "http://a/g"),
        ("../../../g", "http://a/b/c/d;p?q", "http://a/g"),
        ("../../../../g", "http://a/b/c/d;p?q", "http://a/g"),
        // changed with RFC 2396bis
        ("/./g", "http://a/b/c/d;p?q", "http://a/g"),
        // changed with RFC 2396bis
        ("/../g", "http://a/b/c/d;p?q", "http://a/g"),
        ("g.", "http://a/b/c/d;p?q", "http://a/b/c/g."),
        (".g", "http://a/b/c/d;p?q", "http://a/b/c/.g"),
        ("g..", "http://a/b/c/d;p?q", "http://a/b/c/g.."),
        ("..g", "http://a/b/c/d;p?q", "http://a/b/c/..g"),
        ("./../g", "http://a/b/c/d;p?q", "http://a/b/g"),
        ("./g/.", "http://a/b/c/d;p?q", "http://a/b/c/g/"),
        ("g/./h", "http://a/b/c/d;p?q", "http://a/b/c/g/h"),
        ("g/../h", "http://a/b/c/d;p?q", "http://a/b/c/h"),
        ("g;x=1/./y", "http://a/b/c/d;p?q", "http://a/b/c/g;x=1/y"),
        ("g;x=1/../y", "http://a/b/c/d;p?q", "http://a/b/c/y"),
        ("g?y/./x", "http://a/b/c/d;p?q", "http://a/b/c/g?y/./x"),
        ("g?y/../x", "http://a/b/c/d;p?q", "http://a/b/c/g?y/../x"),
        ("g#s/./x", "http://a/b/c/d;p?q", "http://a/b/c/g#s/./x"),
        ("g#s/../x", "http://a/b/c/d;p?q", "http://a/b/c/g#s/../x"),
        ("http:g", "http://a/b/c/d;p?q", "http:g"),
        ("http:", "http://a/b/c/d;p?q", "http:"),
        // not sure where this one originated
        ("/a/b/c/./../../g", "http://a/b/c/d;p?q", "http://a/a/g"),
        // http://gbiv.com/protocols/uri/test/rel_examples2.html
        // slashes in base URI"s query args
        ("g", "http://a/b/c/d;p?q=1/2", "http://a/b/c/g"),
        ("./g", "http://a/b/c/d;p?q=1/2", "http://a/b/c/g"),
        ("g/", "http://a/b/c/d;p?q=1/2", "http://a/b/c/g/"),
        ("/g", "http://a/b/c/d;p?q=1/2", "http://a/g"),
        ("//g", "http://a/b/c/d;p?q=1/2", "http://g"),
        // changed in RFC 2396bis
        ("?y", "http://a/b/c/d;p?q=1/2", "http://a/b/c/d;p?y"),
        ("g?y", "http://a/b/c/d;p?q=1/2", "http://a/b/c/g?y"),
        ("g?y/./x", "http://a/b/c/d;p?q=1/2", "http://a/b/c/g?y/./x"),
        (
            "g?y/../x",
            "http://a/b/c/d;p?q=1/2",
            "http://a/b/c/g?y/../x",
        ),
        ("g#s", "http://a/b/c/d;p?q=1/2", "http://a/b/c/g#s"),
        ("g#s/./x", "http://a/b/c/d;p?q=1/2", "http://a/b/c/g#s/./x"),
        (
            "g#s/../x",
            "http://a/b/c/d;p?q=1/2",
            "http://a/b/c/g#s/../x",
        ),
        ("./", "http://a/b/c/d;p?q=1/2", "http://a/b/c/"),
        ("../", "http://a/b/c/d;p?q=1/2", "http://a/b/"),
        ("../g", "http://a/b/c/d;p?q=1/2", "http://a/b/g"),
        ("../../", "http://a/b/c/d;p?q=1/2", "http://a/"),
        ("../../g", "http://a/b/c/d;p?q=1/2", "http://a/g"),
        // http://gbiv.com/protocols/uri/test/rel_examples3.html
        // slashes in path params
        // all of these changed in RFC 2396bis
        ("g", "http://a/b/c/d;p=1/2?q", "http://a/b/c/d;p=1/g"),
        ("./g", "http://a/b/c/d;p=1/2?q", "http://a/b/c/d;p=1/g"),
        ("g/", "http://a/b/c/d;p=1/2?q", "http://a/b/c/d;p=1/g/"),
        ("g?y", "http://a/b/c/d;p=1/2?q", "http://a/b/c/d;p=1/g?y"),
        (";x", "http://a/b/c/d;p=1/2?q", "http://a/b/c/d;p=1/;x"),
        ("g;x", "http://a/b/c/d;p=1/2?q", "http://a/b/c/d;p=1/g;x"),
        (
            "g;x=1/./y",
            "http://a/b/c/d;p=1/2?q",
            "http://a/b/c/d;p=1/g;x=1/y",
        ),
        (
            "g;x=1/../y",
            "http://a/b/c/d;p=1/2?q",
            "http://a/b/c/d;p=1/y",
        ),
        ("./", "http://a/b/c/d;p=1/2?q", "http://a/b/c/d;p=1/"),
        ("../", "http://a/b/c/d;p=1/2?q", "http://a/b/c/"),
        ("../g", "http://a/b/c/d;p=1/2?q", "http://a/b/c/g"),
        ("../../", "http://a/b/c/d;p=1/2?q", "http://a/b/"),
        ("../../g", "http://a/b/c/d;p=1/2?q", "http://a/b/g"),
        // http://gbiv.com/protocols/uri/test/rel_examples4.html
        // double and triple slash, unknown scheme
        ("g:h", "fred:///s//a/b/c", "g:h"),
        ("g", "fred:///s//a/b/c", "fred:///s//a/b/g"),
        ("./g", "fred:///s//a/b/c", "fred:///s//a/b/g"),
        ("g/", "fred:///s//a/b/c", "fred:///s//a/b/g/"),
        ("/g", "fred:///s//a/b/c", "fred:///g"), // may change to fred:///s//a/g
        ("//g", "fred:///s//a/b/c", "fred://g"), // may change to fred:///s//g
        ("//g/x", "fred:///s//a/b/c", "fred://g/x"), // may change to fred:///s//g/x
        ("///g", "fred:///s//a/b/c", "fred:///g"),
        ("./", "fred:///s//a/b/c", "fred:///s//a/b/"),
        ("../", "fred:///s//a/b/c", "fred:///s//a/"),
        ("../g", "fred:///s//a/b/c", "fred:///s//a/g"),
        ("../../", "fred:///s//a/b/c", "fred:///s//"), // may change to fred:///s//a/../
        ("../../g", "fred:///s//a/b/c", "fred:///s//g"), // may change to fred:///s//a/../g
        ("../../../g", "fred:///s//a/b/c", "fred:///s/g"), // may change to fred:///s//a/../../g
        ("../../../../g", "fred:///s//a/b/c", "fred:///g"), // may change to fred:///s//a/../../../g
        // http://gbiv.com/protocols/uri/test/rel_examples5.html
        // double and triple slash, well-known scheme
        ("g:h", "http:///s//a/b/c", "g:h"),
        ("g", "http:///s//a/b/c", "http:///s//a/b/g"),
        ("./g", "http:///s//a/b/c", "http:///s//a/b/g"),
        ("g/", "http:///s//a/b/c", "http:///s//a/b/g/"),
        ("/g", "http:///s//a/b/c", "http:///g"), // may change to http:///s//a/g
        ("//g", "http:///s//a/b/c", "http://g"), // may change to http:///s//g
        ("//g/x", "http:///s//a/b/c", "http://g/x"), // may change to http:///s//g/x
        ("///g", "http:///s//a/b/c", "http:///g"),
        ("./", "http:///s//a/b/c", "http:///s//a/b/"),
        ("../", "http:///s//a/b/c", "http:///s//a/"),
        ("../g", "http:///s//a/b/c", "http:///s//a/g"),
        ("../../", "http:///s//a/b/c", "http:///s//"), // may change to http:///s//a/../
        ("../../g", "http:///s//a/b/c", "http:///s//g"), // may change to http:///s//a/../g
        ("../../../g", "http:///s//a/b/c", "http:///s/g"), // may change to http:///s//a/../../g
        ("../../../../g", "http:///s//a/b/c", "http:///g"), // may change to http:///s//a/../../../g
        // from Dan Connelly"s tests in http://www.w3.org/2000/10/swap/uripath.py
        ("bar:abc", "foo:xyz", "bar:abc"),
        ("../abc", "http://example/x/y/z", "http://example/x/abc"),
        (
            "http://example/x/abc",
            "http://example2/x/y/z",
            "http://example/x/abc",
        ),
        ("../r", "http://ex/x/y/z", "http://ex/x/r"),
        ("q/r", "http://ex/x/y", "http://ex/x/q/r"),
        ("q/r#s", "http://ex/x/y", "http://ex/x/q/r#s"),
        ("q/r#s/t", "http://ex/x/y", "http://ex/x/q/r#s/t"),
        ("ftp://ex/x/q/r", "http://ex/x/y", "ftp://ex/x/q/r"),
        ("", "http://ex/x/y", "http://ex/x/y"),
        ("", "http://ex/x/y/", "http://ex/x/y/"),
        ("", "http://ex/x/y/pdq", "http://ex/x/y/pdq"),
        ("z/", "http://ex/x/y/", "http://ex/x/y/z/"),
        (
            "#Animal",
            "file:/swap/test/animal.rdf",
            "file:/swap/test/animal.rdf#Animal",
        ),
        ("../abc", "file:/e/x/y/z", "file:/e/x/abc"),
        (
            "/example/x/abc",
            "file:/example2/x/y/z",
            "file:/example/x/abc",
        ),
        ("../r", "file:/ex/x/y/z", "file:/ex/x/r"),
        ("/r", "file:/ex/x/y/z", "file:/r"),
        ("q/r", "file:/ex/x/y", "file:/ex/x/q/r"),
        ("q/r#s", "file:/ex/x/y", "file:/ex/x/q/r#s"),
        ("q/r#", "file:/ex/x/y", "file:/ex/x/q/r#"),
        ("q/r#s/t", "file:/ex/x/y", "file:/ex/x/q/r#s/t"),
        ("ftp://ex/x/q/r", "file:/ex/x/y", "ftp://ex/x/q/r"),
        ("", "file:/ex/x/y", "file:/ex/x/y"),
        ("", "file:/ex/x/y/", "file:/ex/x/y/"),
        ("", "file:/ex/x/y/pdq", "file:/ex/x/y/pdq"),
        ("z/", "file:/ex/x/y/", "file:/ex/x/y/z/"),
        (
            "file://meetings.example.com/cal#m1",
            "file:/devel/WWW/2000/10/swap/test/reluri-1.n3",
            "file://meetings.example.com/cal#m1",
        ),
        (
            "file://meetings.example.com/cal#m1",
            "file:/home/connolly/w3ccvs/WWW/2000/10/swap/test/reluri-1.n3",
            "file://meetings.example.com/cal#m1",
        ),
        ("./#blort", "file:/some/dir/foo", "file:/some/dir/#blort"),
        ("./#", "file:/some/dir/foo", "file:/some/dir/#"),
        // Ryan Lee
        ("./", "http://example/x/abc.efg", "http://example/x/"),
        // Graham Klyne"s tests
        // http://www.ninebynine.org/Software/HaskellUtils/Network/UriTest.xls
        // 01-31 are from Connelly"s cases

        // 32-49
        ("./q:r", "http://ex/x/y", "http://ex/x/q:r"),
        ("./p=q:r", "http://ex/x/y", "http://ex/x/p=q:r"),
        ("?pp/rr", "http://ex/x/y?pp/qq", "http://ex/x/y?pp/rr"),
        ("y/z", "http://ex/x/y?pp/qq", "http://ex/x/y/z"),
        ("y?q", "http://ex/x/y?q", "http://ex/x/y?q"),
        ("/x/y?q", "http://ex?p", "http://ex/x/y?q"),
        ("c/d", "foo:a/b", "foo:a/c/d"),
        ("/c/d", "foo:a/b", "foo:/c/d"),
        ("", "foo:a/b?c#d", "foo:a/b?c"),
        ("b/c", "foo:a", "foo:b/c"),
        ("../b/c", "foo:/a/y/z", "foo:/a/b/c"),
        ("./b/c", "foo:a", "foo:b/c"),
        ("/./b/c", "foo:a", "foo:/b/c"),
        ("../../d", "foo://a//b/c", "foo://a/d"),
        (".", "foo:a", "foo:"),
        ("..", "foo:a", "foo:"),
        // 50-57 (cf. TimBL comments --
        //  http://lists.w3.org/Archives/Public/uri/2003Feb/0028.html,
        //  http://lists.w3.org/Archives/Public/uri/2003Jan/0008.html)
        ("abc", "http://example/x/y%2Fz", "http://example/x/abc"),
        (
            "../../x%2Fabc",
            "http://example/a/x/y/z",
            "http://example/a/x%2Fabc",
        ),
        (
            "../x%2Fabc",
            "http://example/a/x/y%2Fz",
            "http://example/a/x%2Fabc",
        ),
        ("abc", "http://example/x%2Fy/z", "http://example/x%2Fy/abc"),
        ("q%3Ar", "http://ex/x/y", "http://ex/x/q%3Ar"),
        (
            "/x%2Fabc",
            "http://example/x/y%2Fz",
            "http://example/x%2Fabc",
        ),
        ("/x%2Fabc", "http://example/x/y/z", "http://example/x%2Fabc"),
        (
            "/x%2Fabc",
            "http://example/x/y%2Fz",
            "http://example/x%2Fabc",
        ),
        // 70-77
        (
            "http://example/a/b?c/../d",
            "foo:bar",
            "http://example/a/b?c/../d",
        ),
        (
            "http://example/a/b#c/../d",
            "foo:bar",
            "http://example/a/b#c/../d",
        ),
        // 82-88
        ("http:this", "http://example.org/base/uri", "http:this"),
        ("http:this", "http:base", "http:this"),
        (
            "mini1.xml",
            "file:///C:/DEV/Haskell/lib/HXmlToolbox-3.01/examples/",
            "file:///C:/DEV/Haskell/lib/HXmlToolbox-3.01/examples/mini1.xml",
        ),
        // More bad test by Rio
        ("?bar", "file:foo", "file:foo?bar"),
        ("#bar", "file:foo", "file:foo#bar"),
        ("/lv2.h", "file:foo", "file:/lv2.h"),
        ("/lv2.h", "file:foo", "file:/lv2.h"),
        ("///lv2.h", "file:foo", "file:///lv2.h"),
        ("lv2.h", "file:foo", "file:lv2.h"),
        ("s", "http://example.com", "http://example.com/s"),
        (".", "file:", "file:"),
        ("..", "file:", "file:"),
        ("./", "file:", "file:"),
        ("../", "file:", "file:"),
        ("./.", "file:", "file:"),
        ("../..", "file:", "file:"),
    ];

    for (relative, base, output) in examples {
        let base = Iri::parse(base).unwrap();
        let output = Iri::parse(output).unwrap();
        match base.resolve(relative) {
            Ok(result) => {
                assert_eq!(
                    result,
                    output,
                    "Resolving of {relative} against {base} is wrong. Found {result} and expecting {output}"
                );
                assert_eq!(result.scheme(), output.scheme());
                assert_eq!(result.authority(), output.authority());
                assert_eq!(result.path(), output.path());
                assert_eq!(result.query(), output.query());
                assert_eq!(result.fragment(), output.fragment());
            }
            Err(error) => panic!(
                "Resolving of {} against {} failed with error: {}",
                relative, base, error
            ),
        }
        let result = base.resolve_unchecked(relative);
        assert_eq!(
            result.as_str(),
            output,
            "Lenient resolving of {relative} against {base} is wrong. Found {result} and expecting {output}"
        );
    }
}

#[test]
fn test_resolve_relative_iri_unchecked() {
    // Tests from https://github.com/apache/jena/blob/main/jena-iri/src/test/resources/org/apache/jena/iri/test.xml
    // License: https://github.com/apache/jena/blob/main/LICENSE
    // Notice: https://github.com/apache/jena/blob/main/NOTICE

    let examples = [
        ("../foo", "http://host/", "http://host/foo"),
        ("../foo", "http://host/xyz", "http://host/foo"),
        ("d/z?x=a", "http://www.example.org/a/b/c/d", "http://www.example.org/a/b/c/d/z?x=a"),
        ("http://example.com/A", "http://www.example.org/a/b/c/d", "http://example.com/A"),
        ("", "http://www.example.org/a/b/c/d/", "http://www.example.org/a/b/c/d/"),
        (".", "http://www.example.org/a/b/c/d/", "http://www.example.org/a/b/c/d/"),
        ("../../C/D", "http://www.example.org/a/b/c/d/", "http://www.example.org/a/b/C/D"),
        ("../../c/d/", "http://www.example.org/a/b/c/d/", "http://www.example.org/a/b/c/d/"),
        ("../../c/d/X#bar", "http://www.example.org/a/b/c/d/", "http://www.example.org/a/b/c/d/X#bar"),
        ("../../c/d/e/f/g/", "http://www.example.org/a/b/c/d/", "http://www.example.org/a/b/c/d/e/f/g/"),
        ("../../c/d/z?x=a", "http://www.example.org/a/b/c/d/", "http://www.example.org/a/b/c/d/z?x=a"),
        ("http://ex.org/../../c/d/z?x=a", "http://www.example.org./a/b/c/d/", "http://ex.org/c/d/z?x=a"),
        ("http://ex.org/c/./d/z?x=a", "http://www.example.org/a/b/c/d/", "http://ex.org/c/d/z?x=a"),
        ("http://example.org/#André", "http://www.w3.org/2000/10/rdf-tests/rdfcore/rdf-charmod-uris/test001.rdf", "http://example.org/#André"),
        ("http://example.org/#Andr%C3%A9", "http://www.w3.org/2000/10/rdf-tests/rdfcore/rdf-charmod-uris/test002.rdf", "http://example.org/#Andr%C3%A9"),
        ("#Dürst", "http://www.w3.org/2000/10/rdf-tests/rdfcore/rdfms-difference-between-ID-and-about/test2.rdf", "http://www.w3.org/2000/10/rdf-tests/rdfcore/rdfms-difference-between-ID-and-about/test2.rdf#Dürst"),
        ("#", "base:x", "base:x#"),
        ("", "file:///C:/Documents and Settings/jjchplb/Local Settings/Temp/test-load-with-41.rdf", "file:///C:/Documents and Settings/jjchplb/Local Settings/Temp/test-load-with-41.rdf"),
        ("eh:/a", "file:///C:/Documents and Settings/jjchplb/Local Settings/Temp/test-load-with-41.rdf", "eh:/a"),
        ("#", "file:///C:/eclipse/workspace/jena2/", "file:///C:/eclipse/workspace/jena2/#"),
        ("", "file:///C:/eclipse/workspace/jena2/", "file:///C:/eclipse/workspace/jena2/"),
        ("base", "file:///C:/eclipse/workspace/jena2/", "file:///C:/eclipse/workspace/jena2/base"),
        ("eh://R", "file:///C:/eclipse/workspace/jena2/", "eh://R"),
        ("eh:/O", "file:///C:/eclipse/workspace/jena2/", "eh:/O"),
        // we are not aware of file: scheme ("file:foo.n3", "file:///C:/eclipse/workspace/jena2/", "file:///C:/eclipse/workspace/jena2/foo.n3"),
        // we are not aware of file: scheme("file:model8.n3", "file:///C:/eclipse/workspace/jena2/", "file:///C:/eclipse/workspace/jena2/model8.n3"),
        ("rdf://test.com#", "file:///C:/eclipse/workspace/jena2/", "rdf://test.com#"),
        ("z", "file:///C:/eclipse/workspace/jena2/foo.n3", "file:///C:/eclipse/workspace/jena2/z"),
        ("", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Ask/manifest.ttl", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Ask/manifest.ttl"),
        ("r-base-prefix-3.ttl", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Basic/manifest.ttl", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Basic/r-base-prefix-3.ttl"),
        ("r-base-prefix-4.ttl", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Basic/manifest.ttl", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Basic/r-base-prefix-4.ttl"),
        ("mailto:bert@example.net", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Optional/result-opt-1.ttl", "mailto:bert@example.net"),
        ("Bound/manifest.n3", "file:///C:/eclipse/workspace/jena2/testing/ARQ/manifest-arq.ttl", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Bound/manifest.n3"),
        ("Construct/manifest.ttl", "file:///C:/eclipse/workspace/jena2/testing/ARQ/manifest-arq.ttl", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Construct/manifest.ttl"),
        ("Dataset/manifest.n3", "file:///C:/eclipse/workspace/jena2/testing/ARQ/manifest-arq.ttl", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Dataset/manifest.n3"),
        ("mailto:jlow@example.com", "file:///C:/eclipse/workspace/jena2/testing/DAWG-Approved/examples/ex2-4a.n3", "mailto:jlow@example.com"),
        ("ex11.2.3.2_0.rq", "file:///C:/eclipse/workspace/jena2/testing/DAWG/examples/manifest.n3", "file:///C:/eclipse/workspace/jena2/testing/DAWG/examples/ex11.2.3.2_0.rq"),
        ("urn:/*not_a_comment*/", "file:///C:/eclipse/workspace/jena2/testing/RDQL-ARQ/result-0-01.n3", "urn:/*not_a_comment*/"),
        ("#y1", "file:///C:/eclipse/workspace/jena2/testing/ontology/bugs/test_hk_06/b.owl", "file:///C:/eclipse/workspace/jena2/testing/ontology/bugs/test_hk_06/b.owl#y1"),
        ("", "file:///C:/eclipse/workspace/jena2/testing/ontology/bugs/test_hk_06/b.owl", "file:///C:/eclipse/workspace/jena2/testing/ontology/bugs/test_hk_06/b.owl"),
        ("foo#ClassAC", "file:///C:/eclipse/workspace/jena2/testing/ontology/bugs/test_hk_07A.owl", "file:///C:/eclipse/workspace/jena2/testing/ontology/bugs/foo#ClassAC"),
        // we are not aware of file: scheme ("file:testing/ontology/bugs/test_hk_07A.owl", "file:///C:/eclipse/workspace/jena2/testing/ontology/bugs/test_hk_07A.owl", "file:///C:/eclipse/workspace/jena2/testing/ontology/bugs/testing/ontology/bugs/test_hk_07A.owl"),
        ("jason6", "file:///C:/eclipse/workspace/jena2/testing/reasoners/bugs/sbug.rdf", "file:///C:/eclipse/workspace/jena2/testing/reasoners/bugs/jason6"),
        ("urn:x-propNum100", "file:///C:/eclipse/workspace/jena2/testing/reasoners/bugs/subpropertyModel.n3", "urn:x-propNum100"),
        ("eh:/V", "file:///C:/eclipse/workspace/jena2/testing/reasoners/bugs/unbroken.n3", "eh:/V"),
        ("eh:/a", "file:///C:/eclipse/workspace/jena2/testing/reasoners/bugs/unbroken.n3", "eh:/a"),
        ("", "file:C:\\DOCUME~1\\jjchplb\\LOCALS~1\\Temp\\hedgehog6739.rdf", "file:C:\\DOCUME~1\\jjchplb\\LOCALS~1\\Temp\\hedgehog6739.rdf"),
        ("http://spoo.net/O", "file:C:\\DOCUME~1\\jjchplb\\LOCALS~1\\Temp\\hedgehog6739.rdf", "http://spoo.net/O"),
        ("http://spoo.net/S", "file:C:\\DOCUME~1\\jjchplb\\LOCALS~1\\Temp\\hedgehog6739.rdf", "http://spoo.net/S"),
        ("urn:x-hp:eg/", "file:doc/inference/data/owlDemoSchema.xml", "urn:x-hp:eg/"),
        ("", "file:testing/abbreviated/relative-uris.rdf", "file:testing/abbreviated/relative-uris.rdf"),
        (".", "file:testing/abbreviated/relative-uris.rdf", "file:testing/abbreviated/"),
        ("../../C/D", "file:testing/abbreviated/relative-uris.rdf", "file:C/D"),
        ("//example.com/A", "file:testing/abbreviated/relative-uris.rdf", "file://example.com/A"),
        ("/A/B#foo/", "file:testing/abbreviated/relative-uris.rdf", "file:/A/B#foo/"),
        ("X#bar", "file:testing/abbreviated/relative-uris.rdf", "file:testing/abbreviated/X#bar"),
        ("e/f/g/", "file:testing/abbreviated/relative-uris.rdf", "file:testing/abbreviated/e/f/g/"),
        ("http://www.example.org/a/b/c/d/", "file:testing/abbreviated/relative-uris.rdf", "http://www.example.org/a/b/c/d/"),
        ("z?x=a", "file:testing/abbreviated/relative-uris.rdf", "file:testing/abbreviated/z?x=a"),
        ("", "file:testing/arp/error-msgs/test06.rdf", "file:testing/arp/error-msgs/test06.rdf"),
        ("#one", "file:testing/arp/qname-in-ID/bug74_0.rdf", "file:testing/arp/qname-in-ID/bug74_0.rdf#one"),
        ("#sw:test", "file:testing/arp/qname-in-ID/bug74_0.rdf", "file:testing/arp/qname-in-ID/bug74_0.rdf#sw:test"),
        ("http://localhost:8080/Repository/QueryAgent/UserOntology/qgen-example-1#", "file:testing/ontology/bugs/test_dk_01.xml", "http://localhost:8080/Repository/QueryAgent/UserOntology/qgen-example-1#"),
        ("owl#Thing", "file:testing/ontology/bugs/test_dk_01.xml", "file:testing/ontology/bugs/owl#Thing"),
        ("#__rest3", "file:testing/ontology/bugs/test_oh_01.owl", "file:testing/ontology/bugs/test_oh_01.owl#__rest3"),
        // Seems wrong ("file:testing/ontology/daml/test-instance-load-classes.daml", "file:testing/ontology/daml/test-instance-load.daml", "file:testing/ontology/daml/testing/ontology/daml/test-instance-load-classes.daml"),
        ("#Union2", "file:testing/ontology/owl/list-syntax/test-ldp.rdf", "file:testing/ontology/owl/list-syntax/test-ldp.rdf#Union2"),
        ("urn:foo", "file:testing/reasoners/bugs/cardFPTest.owl", "urn:foo"),
        ("http://decsai.ugr.es/~ontoserver/bacarex2.owl", "file:testing/reasoners/bugs/deleteBug.owl", "http://decsai.ugr.es/~ontoserver/bacarex2.owl"),
        ("#A", "file:testing/reasoners/bugs/equivalentClassTest.owl", "file:testing/reasoners/bugs/equivalentClassTest.owl#A"),
        ("NC:ispinfo", "http://bar.com/irrelevant", "NC:ispinfo"),
        ("NC:trickMe", "http://bar.com/irrelevant", "NC:trickMe"),
        ("chrome://messenger/content/mailPrefsOverlay.xul", "http://bar.com/irrelevant", "chrome://messenger/content/mailPrefsOverlay.xul"),
        ("domain:aol.com", "http://bar.com/irrelevant", "domain:aol.com"),
        ("http://foo.com/    ", "http://bar.com/irrelevant", "http://foo.com/    "),
        ("http://foo.com/   ", "http://bar.com/irrelevant", "http://foo.com/   "),
        ("http://foo.com/  ", "http://bar.com/irrelevant", "http://foo.com/  "),
        ("http://foo.com/ ", "http://bar.com/irrelevant", "http://foo.com/ "),
        ("http://foo.com/\t", "http://bar.com/irrelevant", "http://foo.com/\t"),
        ("http://foo.com/\n\n", "http://bar.com/irrelevant", "http://foo.com/\n\n"),
        ("http://foo.com/\r", "http://bar.com/irrelevant", "http://foo.com/\r"),
        ("http://foo.com/'", "http://bar.com/irrelevant", "http://foo.com/'"),
        ("http://foo.com/<b>boo", "http://bar.com/irrelevant", "http://foo.com/<b>boo"),
        ("http://foo.com/\"", "http://bar.com/irrelevant", "http://foo.com/\""),
        ("http://foo.com/", "http://bar.com/irrelevant", "http://foo.com/"),
        ("../foo", "http://host/", "http://host/foo"),
        ("../foo", "http://host/xyz", "http://host/foo"),
        ("d/z?x=a", "http://www.example.org/a/b/c/d", "http://www.example.org/a/b/c/d/z?x=a"),
        ("http://example.com/A", "http://www.example.org/a/b/c/d", "http://example.com/A"),
        ("", "http://www.example.org/a/b/c/d/", "http://www.example.org/a/b/c/d/"),
        (".", "http://www.example.org/a/b/c/d/", "http://www.example.org/a/b/c/d/"),
        ("../../C/D", "http://www.example.org/a/b/c/d/", "http://www.example.org/a/b/C/D"),
        ("../../c/d/", "http://www.example.org/a/b/c/d/", "http://www.example.org/a/b/c/d/"),
        ("../../c/d/X#bar", "http://www.example.org/a/b/c/d/", "http://www.example.org/a/b/c/d/X#bar"),
        ("../../c/d/e/f/g/", "http://www.example.org/a/b/c/d/", "http://www.example.org/a/b/c/d/e/f/g/"),
        ("../../c/d/z?x=a", "http://www.example.org/a/b/c/d/", "http://www.example.org/a/b/c/d/z?x=a"),
        ("http://ex.org/../../c/d/z?x=a", "http://www.example.org./a/b/c/d/", "http://ex.org/c/d/z?x=a"),
        ("http://ex.org/c/./d/z?x=a", "http://www.example.org/a/b/c/d/", "http://ex.org/c/d/z?x=a"),
        ("http://example.org/#André", "http://www.w3.org/2000/10/rdf-tests/rdfcore/rdf-charmod-uris/test001.rdf", "http://example.org/#André"),
        ("http://example.org/#Andr%C3%A9", "http://www.w3.org/2000/10/rdf-tests/rdfcore/rdf-charmod-uris/test002.rdf", "http://example.org/#Andr%C3%A9"),
        ("#Dürst", "http://www.w3.org/2000/10/rdf-tests/rdfcore/rdfms-difference-between-ID-and-about/test2.rdf", "http://www.w3.org/2000/10/rdf-tests/rdfcore/rdfms-difference-between-ID-and-about/test2.rdf#Dürst"),
        ("#", "base:x", "base:x#"),
        ("", "file:///C:/Documents and Settings/jjchplb/Local Settings/Temp/test-load-with-41.rdf", "file:///C:/Documents and Settings/jjchplb/Local Settings/Temp/test-load-with-41.rdf"),
        ("eh:/a", "file:///C:/Documents and Settings/jjchplb/Local Settings/Temp/test-load-with-41.rdf", "eh:/a"),
        ("#", "file:///C:/eclipse/workspace/jena2/", "file:///C:/eclipse/workspace/jena2/#"),
        ("", "file:///C:/eclipse/workspace/jena2/", "file:///C:/eclipse/workspace/jena2/"),
        ("base", "file:///C:/eclipse/workspace/jena2/", "file:///C:/eclipse/workspace/jena2/base"),
        ("eh://R", "file:///C:/eclipse/workspace/jena2/", "eh://R"),
        ("eh:/O", "file:///C:/eclipse/workspace/jena2/", "eh:/O"),
        // we are not aware of file: ("file:foo.n3", "file:///C:/eclipse/workspace/jena2/", "file:///C:/eclipse/workspace/jena2/foo.n3"),
        // we are not aware of file: ("file:model8.n3", "file:///C:/eclipse/workspace/jena2/", "file:///C:/eclipse/workspace/jena2/model8.n3"),
        // we are not aware of file: ("rdf://test.com#", "file:///C:/eclipse/workspace/jena2/", "rdf://test.com#"),
        ("z", "file:///C:/eclipse/workspace/jena2/foo.n3", "file:///C:/eclipse/workspace/jena2/z"),
        ("", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Ask/manifest.ttl", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Ask/manifest.ttl"),
        ("r-base-prefix-3.ttl", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Basic/manifest.ttl", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Basic/r-base-prefix-3.ttl"),
        ("r-base-prefix-4.ttl", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Basic/manifest.ttl", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Basic/r-base-prefix-4.ttl"),
        ("mailto:bert@example.net", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Optional/result-opt-1.ttl", "mailto:bert@example.net"),
        ("Bound/manifest.n3", "file:///C:/eclipse/workspace/jena2/testing/ARQ/manifest-arq.ttl", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Bound/manifest.n3"),
        ("Construct/manifest.ttl", "file:///C:/eclipse/workspace/jena2/testing/ARQ/manifest-arq.ttl", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Construct/manifest.ttl"),
        ("Dataset/manifest.n3", "file:///C:/eclipse/workspace/jena2/testing/ARQ/manifest-arq.ttl", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Dataset/manifest.n3"),
        ("mailto:jlow@example.com", "file:///C:/eclipse/workspace/jena2/testing/DAWG-Approved/examples/ex2-4a.n3", "mailto:jlow@example.com"),
        ("ex11.2.3.2_0.rq", "file:///C:/eclipse/workspace/jena2/testing/DAWG/examples/manifest.n3", "file:///C:/eclipse/workspace/jena2/testing/DAWG/examples/ex11.2.3.2_0.rq"),
        ("urn:/*not_a_comment*/", "file:///C:/eclipse/workspace/jena2/testing/RDQL-ARQ/result-0-01.n3", "urn:/*not_a_comment*/"),
        ("#y1", "file:///C:/eclipse/workspace/jena2/testing/ontology/bugs/test_hk_06/b.owl", "file:///C:/eclipse/workspace/jena2/testing/ontology/bugs/test_hk_06/b.owl#y1"),
        ("", "file:///C:/eclipse/workspace/jena2/testing/ontology/bugs/test_hk_06/b.owl", "file:///C:/eclipse/workspace/jena2/testing/ontology/bugs/test_hk_06/b.owl"),
        ("foo#ClassAC", "file:///C:/eclipse/workspace/jena2/testing/ontology/bugs/test_hk_07A.owl", "file:///C:/eclipse/workspace/jena2/testing/ontology/bugs/foo#ClassAC"),
        // we are not aware of file: ("file:testing/ontology/bugs/test_hk_07A.owl", "file:///C:/eclipse/workspace/jena2/testing/ontology/bugs/test_hk_07A.owl", "file:///C:/eclipse/workspace/jena2/testing/ontology/bugs/testing/ontology/bugs/test_hk_07A.owl"),
        ("jason6", "file:///C:/eclipse/workspace/jena2/testing/reasoners/bugs/sbug.rdf", "file:///C:/eclipse/workspace/jena2/testing/reasoners/bugs/jason6"),
        ("urn:x-propNum100", "file:///C:/eclipse/workspace/jena2/testing/reasoners/bugs/subpropertyModel.n3", "urn:x-propNum100"),
        ("eh:/V", "file:///C:/eclipse/workspace/jena2/testing/reasoners/bugs/unbroken.n3", "eh:/V"),
        ("eh:/a", "file:///C:/eclipse/workspace/jena2/testing/reasoners/bugs/unbroken.n3", "eh:/a"),
        ("", "file:C:\\DOCUME~1\\jjchplb\\LOCALS~1\\Temp\\hedgehog6739.rdf", "file:C:\\DOCUME~1\\jjchplb\\LOCALS~1\\Temp\\hedgehog6739.rdf"),
        ("http://spoo.net/O", "file:C:\\DOCUME~1\\jjchplb\\LOCALS~1\\Temp\\hedgehog6739.rdf", "http://spoo.net/O"),
        ("http://spoo.net/S", "file:C:\\DOCUME~1\\jjchplb\\LOCALS~1\\Temp\\hedgehog6739.rdf", "http://spoo.net/S"),
        ("urn:x-hp:eg/", "file:doc/inference/data/owlDemoSchema.xml", "urn:x-hp:eg/"),
        ("", "file:testing/abbreviated/relative-uris.rdf", "file:testing/abbreviated/relative-uris.rdf"),
        (".", "file:testing/abbreviated/relative-uris.rdf", "file:testing/abbreviated/"),
        ("../../C/D", "file:testing/abbreviated/relative-uris.rdf", "file:C/D"),
        ("//example.com/A", "file:testing/abbreviated/relative-uris.rdf", "file://example.com/A"),
        ("/A/B#foo/", "file:testing/abbreviated/relative-uris.rdf", "file:/A/B#foo/"),
        ("X#bar", "file:testing/abbreviated/relative-uris.rdf", "file:testing/abbreviated/X#bar"),
        ("e/f/g/", "file:testing/abbreviated/relative-uris.rdf", "file:testing/abbreviated/e/f/g/"),
        ("http://www.example.org/a/b/c/d/", "file:testing/abbreviated/relative-uris.rdf", "http://www.example.org/a/b/c/d/"),
        ("z?x=a", "file:testing/abbreviated/relative-uris.rdf", "file:testing/abbreviated/z?x=a"),
        ("", "file:testing/arp/error-msgs/test06.rdf", "file:testing/arp/error-msgs/test06.rdf"),
        ("#one", "file:testing/arp/qname-in-ID/bug74_0.rdf", "file:testing/arp/qname-in-ID/bug74_0.rdf#one"),
        ("#sw:test", "file:testing/arp/qname-in-ID/bug74_0.rdf", "file:testing/arp/qname-in-ID/bug74_0.rdf#sw:test"),
        ("http://localhost:8080/Repository/QueryAgent/UserOntology/qgen-example-1#", "file:testing/ontology/bugs/test_dk_01.xml", "http://localhost:8080/Repository/QueryAgent/UserOntology/qgen-example-1#"),
        ("owl#Thing", "file:testing/ontology/bugs/test_dk_01.xml", "file:testing/ontology/bugs/owl#Thing"),
        ("#__rest3", "file:testing/ontology/bugs/test_oh_01.owl", "file:testing/ontology/bugs/test_oh_01.owl#__rest3"),
        // we are not aware of file: ("file:testing/ontology/daml/test-instance-load-classes.daml", "file:testing/ontology/daml/test-instance-load.daml", "file:testing/ontology/daml/testing/ontology/daml/test-instance-load-classes.daml"),
        ("#Union2", "file:testing/ontology/owl/list-syntax/test-ldp.rdf", "file:testing/ontology/owl/list-syntax/test-ldp.rdf#Union2"),
        ("urn:foo", "file:testing/reasoners/bugs/cardFPTest.owl", "urn:foo"),
        ("http://decsai.ugr.es/~ontoserver/bacarex2.owl", "file:testing/reasoners/bugs/deleteBug.owl", "http://decsai.ugr.es/~ontoserver/bacarex2.owl"),
        ("#A", "file:testing/reasoners/bugs/equivalentClassTest.owl", "file:testing/reasoners/bugs/equivalentClassTest.owl#A"),
        ("NC:ispinfo", "http://bar.com/irrelevant", "NC:ispinfo"),
        ("NC:trickMe", "http://bar.com/irrelevant", "NC:trickMe"),
        ("chrome://messenger/content/mailPrefsOverlay.xul", "http://bar.com/irrelevant", "chrome://messenger/content/mailPrefsOverlay.xul"),
        ("domain:aol.com", "http://bar.com/irrelevant", "domain:aol.com"),
        ("http://foo.com/    ", "http://bar.com/irrelevant", "http://foo.com/    "),
        ("http://foo.com/   ", "http://bar.com/irrelevant", "http://foo.com/   "),
        ("http://foo.com/  ", "http://bar.com/irrelevant", "http://foo.com/  "),
        ("http://foo.com/ ", "http://bar.com/irrelevant", "http://foo.com/ "),
        ("http://foo.com/\t", "http://bar.com/irrelevant", "http://foo.com/\t"),
        ("http://foo.com/\n\n", "http://bar.com/irrelevant", "http://foo.com/\n\n"),
        ("http://foo.com/\r", "http://bar.com/irrelevant", "http://foo.com/\r"),
        ("http://foo.com/'", "http://bar.com/irrelevant", "http://foo.com/'"),
        ("http://foo.com/<b>boo", "http://bar.com/irrelevant", "http://foo.com/<b>boo"),
        ("http://foo.com/\"", "http://bar.com/irrelevant", "http://foo.com/\""),
        ("http://foo.com/", "http://bar.com/irrelevant", "http://foo.com/"),
        ("../foo", "http://host/", "http://host/foo"),
        ("../foo", "http://host/xyz", "http://host/foo"),
        ("d/z?x=a", "http://www.example.org/a/b/c/d", "http://www.example.org/a/b/c/d/z?x=a"),
        ("http://example.com/A", "http://www.example.org/a/b/c/d", "http://example.com/A"),
        ("", "http://www.example.org/a/b/c/d/", "http://www.example.org/a/b/c/d/"),
        (".", "http://www.example.org/a/b/c/d/", "http://www.example.org/a/b/c/d/"),
        ("../../C/D", "http://www.example.org/a/b/c/d/", "http://www.example.org/a/b/C/D"),
        ("../../c/d/", "http://www.example.org/a/b/c/d/", "http://www.example.org/a/b/c/d/"),
        ("../../c/d/X#bar", "http://www.example.org/a/b/c/d/", "http://www.example.org/a/b/c/d/X#bar"),
        ("../../c/d/e/f/g/", "http://www.example.org/a/b/c/d/", "http://www.example.org/a/b/c/d/e/f/g/"),
        ("../../c/d/z?x=a", "http://www.example.org/a/b/c/d/", "http://www.example.org/a/b/c/d/z?x=a"),
        ("http://ex.org/../../c/d/z?x=a", "http://www.example.org./a/b/c/d/", "http://ex.org/c/d/z?x=a"),
        ("http://ex.org/c/./d/z?x=a", "http://www.example.org/a/b/c/d/", "http://ex.org/c/d/z?x=a"),
        ("http://example.org/#André", "http://www.w3.org/2000/10/rdf-tests/rdfcore/rdf-charmod-uris/test001.rdf", "http://example.org/#André"),
        ("http://example.org/#Andr%C3%A9", "http://www.w3.org/2000/10/rdf-tests/rdfcore/rdf-charmod-uris/test002.rdf", "http://example.org/#Andr%C3%A9"),
        ("#Dürst", "http://www.w3.org/2000/10/rdf-tests/rdfcore/rdfms-difference-between-ID-and-about/test2.rdf", "http://www.w3.org/2000/10/rdf-tests/rdfcore/rdfms-difference-between-ID-and-about/test2.rdf#Dürst"),
        ("#", "base:x", "base:x#"),
        ("", "file:///C:/Documents and Settings/jjchplb/Local Settings/Temp/test-load-with-41.rdf", "file:///C:/Documents and Settings/jjchplb/Local Settings/Temp/test-load-with-41.rdf"),
        ("eh:/a", "file:///C:/Documents and Settings/jjchplb/Local Settings/Temp/test-load-with-41.rdf", "eh:/a"),
        ("#", "file:///C:/eclipse/workspace/jena2/", "file:///C:/eclipse/workspace/jena2/#"),
        ("", "file:///C:/eclipse/workspace/jena2/", "file:///C:/eclipse/workspace/jena2/"),
        ("base", "file:///C:/eclipse/workspace/jena2/", "file:///C:/eclipse/workspace/jena2/base"),
        ("eh://R", "file:///C:/eclipse/workspace/jena2/", "eh://R"),
        ("eh:/O", "file:///C:/eclipse/workspace/jena2/", "eh:/O"),
        // we are not aware of file: ("file:foo.n3", "file:///C:/eclipse/workspace/jena2/", "file:///C:/eclipse/workspace/jena2/foo.n3"),
        // we are not aware of file: ("file:model8.n3", "file:///C:/eclipse/workspace/jena2/", "file:///C:/eclipse/workspace/jena2/model8.n3"),
        ("rdf://test.com#", "file:///C:/eclipse/workspace/jena2/", "rdf://test.com#"),
        ("z", "file:///C:/eclipse/workspace/jena2/foo.n3", "file:///C:/eclipse/workspace/jena2/z"),
        ("", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Ask/manifest.ttl", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Ask/manifest.ttl"),
        ("r-base-prefix-3.ttl", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Basic/manifest.ttl", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Basic/r-base-prefix-3.ttl"),
        ("r-base-prefix-4.ttl", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Basic/manifest.ttl", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Basic/r-base-prefix-4.ttl"),
        ("mailto:bert@example.net", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Optional/result-opt-1.ttl", "mailto:bert@example.net"),
        ("Bound/manifest.n3", "file:///C:/eclipse/workspace/jena2/testing/ARQ/manifest-arq.ttl", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Bound/manifest.n3"),
        ("Construct/manifest.ttl", "file:///C:/eclipse/workspace/jena2/testing/ARQ/manifest-arq.ttl", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Construct/manifest.ttl"),
        ("Dataset/manifest.n3", "file:///C:/eclipse/workspace/jena2/testing/ARQ/manifest-arq.ttl", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Dataset/manifest.n3"),
        ("mailto:jlow@example.com", "file:///C:/eclipse/workspace/jena2/testing/DAWG-Approved/examples/ex2-4a.n3", "mailto:jlow@example.com"),
        ("ex11.2.3.2_0.rq", "file:///C:/eclipse/workspace/jena2/testing/DAWG/examples/manifest.n3", "file:///C:/eclipse/workspace/jena2/testing/DAWG/examples/ex11.2.3.2_0.rq"),
        ("urn:/*not_a_comment*/", "file:///C:/eclipse/workspace/jena2/testing/RDQL-ARQ/result-0-01.n3", "urn:/*not_a_comment*/"),
        ("#y1", "file:///C:/eclipse/workspace/jena2/testing/ontology/bugs/test_hk_06/b.owl", "file:///C:/eclipse/workspace/jena2/testing/ontology/bugs/test_hk_06/b.owl#y1"),
        ("", "file:///C:/eclipse/workspace/jena2/testing/ontology/bugs/test_hk_06/b.owl", "file:///C:/eclipse/workspace/jena2/testing/ontology/bugs/test_hk_06/b.owl"),
        ("foo#ClassAC", "file:///C:/eclipse/workspace/jena2/testing/ontology/bugs/test_hk_07A.owl", "file:///C:/eclipse/workspace/jena2/testing/ontology/bugs/foo#ClassAC"),
        // we are not aware of file: ("file:testing/ontology/bugs/test_hk_07A.owl", "file:///C:/eclipse/workspace/jena2/testing/ontology/bugs/test_hk_07A.owl", "file:///C:/eclipse/workspace/jena2/testing/ontology/bugs/testing/ontology/bugs/test_hk_07A.owl"),
        ("jason6", "file:///C:/eclipse/workspace/jena2/testing/reasoners/bugs/sbug.rdf", "file:///C:/eclipse/workspace/jena2/testing/reasoners/bugs/jason6"),
        ("urn:x-propNum100", "file:///C:/eclipse/workspace/jena2/testing/reasoners/bugs/subpropertyModel.n3", "urn:x-propNum100"),
        ("eh:/V", "file:///C:/eclipse/workspace/jena2/testing/reasoners/bugs/unbroken.n3", "eh:/V"),
        ("eh:/a", "file:///C:/eclipse/workspace/jena2/testing/reasoners/bugs/unbroken.n3", "eh:/a"),
        ("", "file:C:\\DOCUME~1\\jjchplb\\LOCALS~1\\Temp\\hedgehog6739.rdf", "file:C:\\DOCUME~1\\jjchplb\\LOCALS~1\\Temp\\hedgehog6739.rdf"),
        ("http://spoo.net/O", "file:C:\\DOCUME~1\\jjchplb\\LOCALS~1\\Temp\\hedgehog6739.rdf", "http://spoo.net/O"),
        ("http://spoo.net/S", "file:C:\\DOCUME~1\\jjchplb\\LOCALS~1\\Temp\\hedgehog6739.rdf", "http://spoo.net/S"),
        ("urn:x-hp:eg/", "file:doc/inference/data/owlDemoSchema.xml", "urn:x-hp:eg/"),
        ("", "file:testing/abbreviated/relative-uris.rdf", "file:testing/abbreviated/relative-uris.rdf"),
        (".", "file:testing/abbreviated/relative-uris.rdf", "file:testing/abbreviated/"),
        ("../../C/D", "file:testing/abbreviated/relative-uris.rdf", "file:C/D"),
        ("//example.com/A", "file:testing/abbreviated/relative-uris.rdf", "file://example.com/A"),
        ("/A/B#foo/", "file:testing/abbreviated/relative-uris.rdf", "file:/A/B#foo/"),
        ("X#bar", "file:testing/abbreviated/relative-uris.rdf", "file:testing/abbreviated/X#bar"),
        ("e/f/g/", "file:testing/abbreviated/relative-uris.rdf", "file:testing/abbreviated/e/f/g/"),
        ("http://www.example.org/a/b/c/d/", "file:testing/abbreviated/relative-uris.rdf", "http://www.example.org/a/b/c/d/"),
        ("z?x=a", "file:testing/abbreviated/relative-uris.rdf", "file:testing/abbreviated/z?x=a"),
        ("", "file:testing/arp/error-msgs/test06.rdf", "file:testing/arp/error-msgs/test06.rdf"),
        ("#one", "file:testing/arp/qname-in-ID/bug74_0.rdf", "file:testing/arp/qname-in-ID/bug74_0.rdf#one"),
        ("#sw:test", "file:testing/arp/qname-in-ID/bug74_0.rdf", "file:testing/arp/qname-in-ID/bug74_0.rdf#sw:test"),
        ("http://localhost:8080/Repository/QueryAgent/UserOntology/qgen-example-1#", "file:testing/ontology/bugs/test_dk_01.xml", "http://localhost:8080/Repository/QueryAgent/UserOntology/qgen-example-1#"),
        ("owl#Thing", "file:testing/ontology/bugs/test_dk_01.xml", "file:testing/ontology/bugs/owl#Thing"),
        ("#__rest3", "file:testing/ontology/bugs/test_oh_01.owl", "file:testing/ontology/bugs/test_oh_01.owl#__rest3"),
        // we are not aware of file: ("file:testing/ontology/daml/test-instance-load-classes.daml", "file:testing/ontology/daml/test-instance-load.daml", "file:testing/ontology/daml/testing/ontology/daml/test-instance-load-classes.daml"),
        ("#Union2", "file:testing/ontology/owl/list-syntax/test-ldp.rdf", "file:testing/ontology/owl/list-syntax/test-ldp.rdf#Union2"),
        ("urn:foo", "file:testing/reasoners/bugs/cardFPTest.owl", "urn:foo"),
        ("http://decsai.ugr.es/~ontoserver/bacarex2.owl", "file:testing/reasoners/bugs/deleteBug.owl", "http://decsai.ugr.es/~ontoserver/bacarex2.owl"),
        ("#A", "file:testing/reasoners/bugs/equivalentClassTest.owl", "file:testing/reasoners/bugs/equivalentClassTest.owl#A"),
        ("NC:ispinfo", "http://bar.com/irrelevant", "NC:ispinfo"),
        ("NC:trickMe", "http://bar.com/irrelevant", "NC:trickMe"),
        ("chrome://messenger/content/mailPrefsOverlay.xul", "http://bar.com/irrelevant", "chrome://messenger/content/mailPrefsOverlay.xul"),
        ("domain:aol.com", "http://bar.com/irrelevant", "domain:aol.com"),
        ("http://foo.com/    ", "http://bar.com/irrelevant", "http://foo.com/    "),
        ("http://foo.com/   ", "http://bar.com/irrelevant", "http://foo.com/   "),
        ("http://foo.com/  ", "http://bar.com/irrelevant", "http://foo.com/  "),
        ("http://foo.com/ ", "http://bar.com/irrelevant", "http://foo.com/ "),
        ("http://foo.com/\t", "http://bar.com/irrelevant", "http://foo.com/\t"),
        ("http://foo.com/\n\n", "http://bar.com/irrelevant", "http://foo.com/\n\n"),
        ("http://foo.com/\r", "http://bar.com/irrelevant", "http://foo.com/\r"),
        ("http://foo.com/'", "http://bar.com/irrelevant", "http://foo.com/'"),
        ("http://foo.com/<b>boo", "http://bar.com/irrelevant", "http://foo.com/<b>boo"),
        ("http://foo.com/\"", "http://bar.com/irrelevant", "http://foo.com/\""),
        ("http://foo.com/", "http://bar.com/irrelevant", "http://foo.com/"),
        ("../foo", "http://host/", "http://host/foo"),
        ("../foo", "http://host/xyz", "http://host/foo"),
        ("d/z?x=a", "http://www.example.org/a/b/c/d", "http://www.example.org/a/b/c/d/z?x=a"),
        ("http://example.com/A", "http://www.example.org/a/b/c/d", "http://example.com/A"),
        ("", "http://www.example.org/a/b/c/d/", "http://www.example.org/a/b/c/d/"),
        (".", "http://www.example.org/a/b/c/d/", "http://www.example.org/a/b/c/d/"),
        ("../../C/D", "http://www.example.org/a/b/c/d/", "http://www.example.org/a/b/C/D"),
        ("../../c/d/", "http://www.example.org/a/b/c/d/", "http://www.example.org/a/b/c/d/"),
        ("../../c/d/X#bar", "http://www.example.org/a/b/c/d/", "http://www.example.org/a/b/c/d/X#bar"),
        ("../../c/d/e/f/g/", "http://www.example.org/a/b/c/d/", "http://www.example.org/a/b/c/d/e/f/g/"),
        ("../../c/d/z?x=a", "http://www.example.org/a/b/c/d/", "http://www.example.org/a/b/c/d/z?x=a"),
        ("http://ex.org/../../c/d/z?x=a", "http://www.example.org./a/b/c/d/", "http://ex.org/c/d/z?x=a"),
        ("http://ex.org/c/./d/z?x=a", "http://www.example.org/a/b/c/d/", "http://ex.org/c/d/z?x=a"),
        ("http://example.org/#André", "http://www.w3.org/2000/10/rdf-tests/rdfcore/rdf-charmod-uris/test001.rdf", "http://example.org/#André"),
        ("http://example.org/#Andr%C3%A9", "http://www.w3.org/2000/10/rdf-tests/rdfcore/rdf-charmod-uris/test002.rdf", "http://example.org/#Andr%C3%A9"),
        ("#Dürst", "http://www.w3.org/2000/10/rdf-tests/rdfcore/rdfms-difference-between-ID-and-about/test2.rdf", "http://www.w3.org/2000/10/rdf-tests/rdfcore/rdfms-difference-between-ID-and-about/test2.rdf#Dürst"),
        ("#", "base:x", "base:x#"),
        ("", "file:///C:/Documents and Settings/jjchplb/Local Settings/Temp/test-load-with-41.rdf", "file:///C:/Documents and Settings/jjchplb/Local Settings/Temp/test-load-with-41.rdf"),
        ("eh:/a", "file:///C:/Documents and Settings/jjchplb/Local Settings/Temp/test-load-with-41.rdf", "eh:/a"),
        ("#", "file:///C:/eclipse/workspace/jena2/", "file:///C:/eclipse/workspace/jena2/#"),
        ("", "file:///C:/eclipse/workspace/jena2/", "file:///C:/eclipse/workspace/jena2/"),
        ("base", "file:///C:/eclipse/workspace/jena2/", "file:///C:/eclipse/workspace/jena2/base"),
        ("eh://R", "file:///C:/eclipse/workspace/jena2/", "eh://R"),
        ("eh:/O", "file:///C:/eclipse/workspace/jena2/", "eh:/O"),
        // we are not aware of file: ("file:foo.n3", "file:///C:/eclipse/workspace/jena2/", "file:///C:/eclipse/workspace/jena2/foo.n3"),
        // we are not aware of file: ("file:model8.n3", "file:///C:/eclipse/workspace/jena2/", "file:///C:/eclipse/workspace/jena2/model8.n3"),
        ("rdf://test.com#", "file:///C:/eclipse/workspace/jena2/", "rdf://test.com#"),
        ("z", "file:///C:/eclipse/workspace/jena2/foo.n3", "file:///C:/eclipse/workspace/jena2/z"),
        ("", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Ask/manifest.ttl", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Ask/manifest.ttl"),
        ("r-base-prefix-3.ttl", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Basic/manifest.ttl", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Basic/r-base-prefix-3.ttl"),
        ("r-base-prefix-4.ttl", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Basic/manifest.ttl", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Basic/r-base-prefix-4.ttl"),
        ("mailto:bert@example.net", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Optional/result-opt-1.ttl", "mailto:bert@example.net"),
        ("Bound/manifest.n3", "file:///C:/eclipse/workspace/jena2/testing/ARQ/manifest-arq.ttl", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Bound/manifest.n3"),
        ("Construct/manifest.ttl", "file:///C:/eclipse/workspace/jena2/testing/ARQ/manifest-arq.ttl", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Construct/manifest.ttl"),
        ("Dataset/manifest.n3", "file:///C:/eclipse/workspace/jena2/testing/ARQ/manifest-arq.ttl", "file:///C:/eclipse/workspace/jena2/testing/ARQ/Dataset/manifest.n3"),
        ("mailto:jlow@example.com", "file:///C:/eclipse/workspace/jena2/testing/DAWG-Approved/examples/ex2-4a.n3", "mailto:jlow@example.com"),
        ("ex11.2.3.2_0.rq", "file:///C:/eclipse/workspace/jena2/testing/DAWG/examples/manifest.n3", "file:///C:/eclipse/workspace/jena2/testing/DAWG/examples/ex11.2.3.2_0.rq"),
        ("urn:/*not_a_comment*/", "file:///C:/eclipse/workspace/jena2/testing/RDQL-ARQ/result-0-01.n3", "urn:/*not_a_comment*/"),
        ("#y1", "file:///C:/eclipse/workspace/jena2/testing/ontology/bugs/test_hk_06/b.owl", "file:///C:/eclipse/workspace/jena2/testing/ontology/bugs/test_hk_06/b.owl#y1"),
        ("", "file:///C:/eclipse/workspace/jena2/testing/ontology/bugs/test_hk_06/b.owl", "file:///C:/eclipse/workspace/jena2/testing/ontology/bugs/test_hk_06/b.owl"),
        ("foo#ClassAC", "file:///C:/eclipse/workspace/jena2/testing/ontology/bugs/test_hk_07A.owl", "file:///C:/eclipse/workspace/jena2/testing/ontology/bugs/foo#ClassAC"),
        // we are not aware of file: ("file:testing/ontology/bugs/test_hk_07A.owl", "file:///C:/eclipse/workspace/jena2/testing/ontology/bugs/test_hk_07A.owl", "file:///C:/eclipse/workspace/jena2/testing/ontology/bugs/testing/ontology/bugs/test_hk_07A.owl"),
        ("jason6", "file:///C:/eclipse/workspace/jena2/testing/reasoners/bugs/sbug.rdf", "file:///C:/eclipse/workspace/jena2/testing/reasoners/bugs/jason6"),
        ("urn:x-propNum100", "file:///C:/eclipse/workspace/jena2/testing/reasoners/bugs/subpropertyModel.n3", "urn:x-propNum100"),
        ("eh:/V", "file:///C:/eclipse/workspace/jena2/testing/reasoners/bugs/unbroken.n3", "eh:/V"),
        ("eh:/a", "file:///C:/eclipse/workspace/jena2/testing/reasoners/bugs/unbroken.n3", "eh:/a"),
        ("", "file:C:\\DOCUME~1\\jjchplb\\LOCALS~1\\Temp\\hedgehog6739.rdf", "file:C:\\DOCUME~1\\jjchplb\\LOCALS~1\\Temp\\hedgehog6739.rdf"),
        ("http://spoo.net/O", "file:C:\\DOCUME~1\\jjchplb\\LOCALS~1\\Temp\\hedgehog6739.rdf", "http://spoo.net/O"),
        ("http://spoo.net/S", "file:C:\\DOCUME~1\\jjchplb\\LOCALS~1\\Temp\\hedgehog6739.rdf", "http://spoo.net/S"),
        ("urn:x-hp:eg/", "file:doc/inference/data/owlDemoSchema.xml", "urn:x-hp:eg/"),
        ("", "file:testing/abbreviated/relative-uris.rdf", "file:testing/abbreviated/relative-uris.rdf"),
        (".", "file:testing/abbreviated/relative-uris.rdf", "file:testing/abbreviated/"),
        ("../../C/D", "file:testing/abbreviated/relative-uris.rdf", "file:C/D"),
        ("//example.com/A", "file:testing/abbreviated/relative-uris.rdf", "file://example.com/A"),
        ("/A/B#foo/", "file:testing/abbreviated/relative-uris.rdf", "file:/A/B#foo/"),
        ("X#bar", "file:testing/abbreviated/relative-uris.rdf", "file:testing/abbreviated/X#bar"),
        ("e/f/g/", "file:testing/abbreviated/relative-uris.rdf", "file:testing/abbreviated/e/f/g/"),
        ("http://www.example.org/a/b/c/d/", "file:testing/abbreviated/relative-uris.rdf", "http://www.example.org/a/b/c/d/"),
        ("z?x=a", "file:testing/abbreviated/relative-uris.rdf", "file:testing/abbreviated/z?x=a"),
        ("", "file:testing/arp/error-msgs/test06.rdf", "file:testing/arp/error-msgs/test06.rdf"),
        ("#one", "file:testing/arp/qname-in-ID/bug74_0.rdf", "file:testing/arp/qname-in-ID/bug74_0.rdf#one"),
        ("#sw:test", "file:testing/arp/qname-in-ID/bug74_0.rdf", "file:testing/arp/qname-in-ID/bug74_0.rdf#sw:test"),
        ("http://localhost:8080/Repository/QueryAgent/UserOntology/qgen-example-1#", "file:testing/ontology/bugs/test_dk_01.xml", "http://localhost:8080/Repository/QueryAgent/UserOntology/qgen-example-1#"),
        ("owl#Thing", "file:testing/ontology/bugs/test_dk_01.xml", "file:testing/ontology/bugs/owl#Thing"),
        ("#__rest3", "file:testing/ontology/bugs/test_oh_01.owl", "file:testing/ontology/bugs/test_oh_01.owl#__rest3"),
        // we are not aware of file: ("file:testing/ontology/daml/test-instance-load-classes.daml", "file:testing/ontology/daml/test-instance-load.daml", "file:testing/ontology/daml/testing/ontology/daml/test-instance-load-classes.daml"),
        ("#Union2", "file:testing/ontology/owl/list-syntax/test-ldp.rdf", "file:testing/ontology/owl/list-syntax/test-ldp.rdf#Union2"),
        ("urn:foo", "file:testing/reasoners/bugs/cardFPTest.owl", "urn:foo"),
        ("http://decsai.ugr.es/~ontoserver/bacarex2.owl", "file:testing/reasoners/bugs/deleteBug.owl", "http://decsai.ugr.es/~ontoserver/bacarex2.owl"),
        ("#A", "file:testing/reasoners/bugs/equivalentClassTest.owl", "file:testing/reasoners/bugs/equivalentClassTest.owl#A"),
        ("NC:ispinfo", "http://bar.com/irrelevant", "NC:ispinfo"),
        ("NC:trickMe", "http://bar.com/irrelevant", "NC:trickMe"),
        ("chrome://messenger/content/mailPrefsOverlay.xul", "http://bar.com/irrelevant", "chrome://messenger/content/mailPrefsOverlay.xul"),
        ("domain:aol.com", "http://bar.com/irrelevant", "domain:aol.com"),
        ("http://foo.com/    ", "http://bar.com/irrelevant", "http://foo.com/    "),
        ("http://foo.com/   ", "http://bar.com/irrelevant", "http://foo.com/   "),
        ("http://foo.com/  ", "http://bar.com/irrelevant", "http://foo.com/  "),
        ("http://foo.com/ ", "http://bar.com/irrelevant", "http://foo.com/ "),
        ("http://foo.com/\t", "http://bar.com/irrelevant", "http://foo.com/\t"),
        ("http://foo.com/\n\n", "http://bar.com/irrelevant", "http://foo.com/\n\n"),
        ("http://foo.com/\r", "http://bar.com/irrelevant", "http://foo.com/\r"),
        ("http://foo.com/'", "http://bar.com/irrelevant", "http://foo.com/'"),
        ("http://foo.com/<b>boo", "http://bar.com/irrelevant", "http://foo.com/<b>boo"),
        ("http://foo.com/\"", "http://bar.com/irrelevant", "http://foo.com/\""),
        ("http://foo.com/", "http://bar.com/irrelevant", "http://foo.com/")
    ];

    for (relative, base, output) in examples {
        let base = Iri::parse_unchecked(base);
        let result = base.resolve_unchecked(relative);
        assert_eq!(
            result.as_str(),
            output,
            "Resolving of {relative} against {base} is wrong. Found {result} and expecting {output}"
        )
    }
}

#[test]
fn test_relativize_iri() {
    let examples = [
        ("http:", "http:", ""),
        ("http://example.com", "http://example.com", ""),
        ("http://example.com/foo", "http://example.com/foo", ""),
        (
            "http://example.com/foo/bar",
            "http://example.com/foo/bar",
            "",
        ),
        (
            "http://example.com/foo/bar?bat",
            "http://example.com/foo/bar?bat",
            "",
        ),
        (
            "http://example.com/foo/bar?bat#baz",
            "http://example.com/foo/bar?bat#baz",
            "#baz",
        ),
        ("http:", "https:", "http:"),
        ("http://example.com", "http://example.org", "//example.com"),
        ("http://example.com/foo", "http://example.com/bar", "foo"),
        (
            "http://example.com/foo?bat",
            "http://example.com/foo?foo",
            "?bat",
        ),
        (
            "http://example.com/foo?bat#baz",
            "http://example.com/foo?bat#foo",
            "#baz",
        ),
        ("http://example.com", "http:", "//example.com"),
        ("http://example.com", "http://", "//example.com"),
        ("http://example.com", "http://example.org", "//example.com"),
        ("http://example.com/foo", "http://example.com/", "foo"),
        ("http://example.com/foo", "http://example.com/bar", "foo"),
        (
            "http://example.com/foo",
            "http://example.com/bar/baz",
            "/foo",
        ),
        (
            "http://example.com/foo/bar",
            "http://example.com/foo/baz",
            "bar",
        ),
        (
            "http://example.com/foo/bar",
            "http://example.com/foo",
            "/foo/bar",
        ),
        (
            "http://example.com/foo?bar",
            "http://example.com/foo?baz",
            "?bar",
        ),
        (
            "http://example.com?bar",
            "http://example.com/a",
            "//example.com?bar",
        ),
        ("http://example.com?bar", "http://example.com", "?bar"),
        (
            "http://example.com?bar",
            "http://example.com/",
            "//example.com?bar",
        ),
        (
            "http://example.com/foo#bar",
            "http://example.com/foo#baz",
            "#bar",
        ),
        ("http://example.com/foo/", "http://example.com/foo/bar", "."),
        ("http://example.com/:", "http://example.com/foo", "/:"),
        ("http:", "http://example.com", "http:"),
        ("http:?foo", "http://example.com", "http:?foo"),
        (
            "http://example.com",
            "http://example.com/foo",
            "//example.com",
        ),
        (
            "http://example.com",
            "http://example.com?query",
            "//example.com",
        ),
        (
            "http://example.com/foo",
            "http://example.com/foo?query",
            "foo",
        ),
        ("http:?query", "http://example.com?query", "http:?query"),
        ("http:/path", "http://example.com/foo", "http:/path"),
        (
            "http://example.com//a",
            "http://example.com/",
            "//example.com//a",
        ),
        ("urn:ab", "urn:", "ab"),
        ("urn:isbn:foo", "urn:", "urn:isbn:foo"),
        ("urn:is/bn:foo", "urn:", "is/bn:foo"),
        ("t:e/e/p", "t:e/s", "t:e/e/p"),
        ("htt:/foo/gp", "htt:/foo/", "gp"),
        ("htt:/gp", "htt:/", "gp"),
        ("x:", "x://foo", "x:"),
        ("x:", "x:02", "x:"),
        ("x:", "x:?foo", "x:"),
        ("x:", "x:?foo", "x:"),
        ("http://example.com", "http://example.com#foo", ""),
        ("http://example.com/a/", "http://example.com/a/b", "."),
        ("http://example.com/a/?c", "http://example.com/a/b", ".?c"),
    ];

    for (original, base, output) in examples {
        let original = Iri::parse(original).unwrap();
        let base = Iri::parse(base).unwrap();
        let output = IriRef::parse(output).unwrap();
        let actual = base.relativize(&original).unwrap();
        assert_eq!(
            actual, output,
            "Relativizing {original} against {base} gives {actual} and not {output}"
        );
        assert_eq!(actual.scheme(), output.scheme());
        assert_eq!(actual.authority(), output.authority());
        assert_eq!(actual.path(), output.path());
        assert_eq!(actual.query(), output.query());
        assert_eq!(actual.fragment(), output.fragment());
        let resolved = base.resolve(actual.as_str()).unwrap();
        assert_eq!(
            resolved, original,
            "Resolving {actual} against {base} gives {resolved} and not {original}"
        );
    }
}

#[test]
fn test_relativize_iri_fails() {
    let examples = [
        ("http://example.com/a/../b", "http://example.com/s"),
        ("http://example.com/a/..", "http://example.com/s"),
        ("http://example.com/./b", "http://example.com/s"),
        ("http://example.com/.", "http://example.com/s"),
        ("urn:.", "urn:"),
    ];

    for (iri, base) in examples {
        let iri = Iri::parse(iri).unwrap();
        let base = Iri::parse(base).unwrap();
        assert!(
            base.relativize(&iri).is_err(),
            "Relativize {iri} against {base} is not properly failing"
        );
        // We make sure it's not possible to relativize
        assert_ne!(base.resolve(iri.as_str()).unwrap(), iri);
    }
}

#[test]
fn test_eq() {
    let iri = Iri::parse("http://example.com").unwrap();
    assert_eq!(iri, iri);
    assert_eq!(iri, "http://example.com");
    assert_eq!("http://example.com", iri);
    assert_eq!(hash(iri), hash("http://example.com"));
}

fn hash(value: impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn test_str() {
    let iri = Iri::parse("http://example.com").unwrap();
    assert!(iri.starts_with("http://"));
}

#[cfg(feature = "serde")]
#[test]
fn test_iriref_serde_impl() {
    assert_tokens(
        &IriRef::parse("//example.com").unwrap(),
        &[Token::BorrowedStr("//example.com")],
    );
    assert_tokens(
        &IriRef::parse("//example.com".to_string()).unwrap(),
        &[Token::String("//example.com")],
    );
    assert_de_tokens(
        &IriRef::parse("//example.com".to_string()).unwrap(),
        &[Token::BorrowedStr("//example.com")],
    );
    assert_de_tokens_error::<IriRef<String>>(
        &[Token::String(":")],
        "No scheme found in an absolute IRI",
    );
}

#[cfg(feature = "serde")]
#[test]
fn test_iri_serde_impl() {
    assert_tokens(
        &Iri::parse("http://example.com").unwrap(),
        &[Token::BorrowedStr("http://example.com")],
    );
    assert_tokens(
        &Iri::parse("http://example.com".to_string()).unwrap(),
        &[Token::String("http://example.com")],
    );
    assert_de_tokens(
        &Iri::parse("http://example.com".to_string()).unwrap(),
        &[Token::BorrowedStr("http://example.com")],
    );
    assert_de_tokens_error::<Iri<String>>(
        &[Token::String(":")],
        "No scheme found in an absolute IRI",
    );
    assert_de_tokens_error::<Iri<String>>(
        &[Token::String("//example.com")],
        "No scheme found in an absolute IRI",
    );
}
