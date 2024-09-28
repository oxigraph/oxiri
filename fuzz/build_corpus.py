import hashlib
from pathlib import Path

data = ['', '\n', ' ', '#', '#\x00', '#?%2F', '#Animal', '#bar', '#ccc', '#here', '#s', '#\ue000', '%', '%2F', '%AZ',
        '%ZZ', '.', '..', '../', '../..', '../../', '../../../../g', '../../../g', '../../d', '../../g',
        '../../x%2Fabc', '../abc', '../b/c', '../g', '../r', '../x%2Fabc', '..g', './', './#', './#blort', './../g',
        './g', './g/.', './p=q:r', './q:r', '.foo', '.g', '/', '/\x00', '/.', '/../g', '/./g', '/.foo', '//', '///g',
        '///lv2.h', '//[2010:836B:4179::836B:4179]', '//example.org/aaa/bbb#ccc', '//g', '//g/x', '//͏@[]', '//\uffff',
        '/a/b/c/./../../g', '/aaa/bbb#ccc', '/cgi-bin/query?query=hello#fragment', '/demo.txt', '/example/x/abc', '/g',
        '/hello/demo.txt', '/lv2.h', '/r', '/x%2Fabc', '/x/y?q', '/\ue000', '/\uf8ff', '/\ufdef', '/\uffff', ':a/b',
        ';x', '?\x00', '?%2F', '?bar', '?pp/rr', '?y', '?\ue000', '?\ufdef', '?\uffff', 'A C', 'A%Z', "A'C", 'A<C',
        'A>C', 'A[**]C', 'A[C', 'A\\C', 'A]C', 'A^C', 'A`C', 'A{C', 'A|C', 'A}C', 'HTTP://EXAMPLE.ORG/AAA/BBB#CCC',
        '[2010:836B:4179::836B:4179]', 'aaa%2Fbbb', 'abc', 'about:', 'bar:abc', 'bbb#ccc', 'beepbeep\x07\x07', 'dav:',
        'demo.txt', 'demo/hello.txt', 'demo/hello.txt?query=hello#fragment',
        'file:///C:/DEV/Haskell/lib/HXmlToolbox-3.01/examples/', 'file:///foo/bar', 'file://foo',
        'file://meetings.example.com/cal#m1', 'ftp://ex/x/q/r', 'ftp://ftp.is.co.za/rfc/rfc1808.txt',
        'ftp://www.yahoo.com/', 'ftp://www.yahoo.com/hello', 'g', 'g#s', 'g#s/../x', 'g#s/./x', 'g.', 'g..', 'g/',
        'g/../h', 'g/./h', 'g:h', 'g;x', 'g;x=1/../y', 'g;x=1/./y', 'g;x?y#s', 'g?y', 'g?y#s', 'g?y/../x', 'g?y/./x',
        'hello/demo.txt', 'http:', 'http://1.2.3.4/', 'http://[/', 'http://[1080:0:0:0:8:800:200C:417A]/index.html',
        'http://[1080::8:800:200C:417A]/foo', 'http://[2010:836B:4179::836B:4179]', 'http://[3ffe:2a00:100:7031::1]',
        'http://[::192.9.5.5]/ipng', 'http://[::1]a/', 'http://[::FFFF:129.144.52.38]:80/index.html',
        'http://[FEDC:AA98:7654:3210:FEDC:AA98:7654:3210]:80/index.html', 'http://[xyz]/', 'http://]/',
        'http://a.example/?AZaz\ue000\uf8ffÀÖØöø˿ͰͽͿ\u1fff\u200c\u200d⁰\u218fⰀ\u2fef、\ud7ff﨎﷏ﷰ\uffef',
        'http://a.example/AZazÀÖØöø˿ͰͽͿ\u1fff\u200c\u200d⁰\u218fⰀ\u2fef、\ud7ff﨎﷏ﷰ\uffef', 'http://a/?\ue000',
        'http://example.123./aaa/bbb#ccc', 'http://example.com', 'http://example.com#toto', 'http://example.com/',
        'http://example.com/#toto', 'http://example.com/#\ue000', 'http://example.com/foo',
        'http://example.com/foo#toto', 'http://example.com/foo/bar', 'http://example.com/foo/bar#toto',
        'http://example.com/foo/bar/', 'http://example.com/foo/bar/#toto', 'http://example.com/foo/bar/.././baz',
        'http://example.com/foo/bar/?q=1&r=2', 'http://example.com/foo/bar/?q=1&r=2#toto',
        'http://example.com/foo/bar?q=1&r=2', 'http://example.com/foo/bar?q=1&r=2#toto', 'http://example.com/\ue000',
        'http://example.org', 'http://example.org./aaa/bbb#ccc', 'http://example.org/[2010:836B:4179::836B:4179]',
        'http://example.org/aaa%2Fbbb#ccc', 'http://example.org/aaa%2fbbb#ccc', 'http://example.org/aaa/bbb#ccc',
        'http://example.org/abc#[2010:836B:4179::836B:4179]', 'http://example.org/xxx/[qwerty]#a[b]',
        'http://example.org:/aaa/bbb#ccc', 'http://example.org:80/aaa/bbb#ccc', 'http://example/Andr&#567;',
        'http://example/a/b#c/../d', 'http://example/a/b?c/../d', 'http://example/x/abc',
        'http://w3c.org:80path1/path2', 'http://www yahoo.com', 'http://www.ietf.org/rfc/rfc2396.txt',
        'http://www.yaho%6f.com', 'http://www.yahoo.com', 'http://www.yahoo.com#bottom', 'http://www.yahoo.com/',
        'http://www.yahoo.com/hello world/', 'http://www.yahoo.com/hello%00world/',
        'http://www.yahoo.com/hello%20world/', 'http://www.yahoo.com/hello+world/', 'http://www.yahoo.com/stuff',
        'http://www.yahoo.com/stuff/', 'http://www.yahoo.com/yelp.html#', 'http://www.yahoo.com/yelp.html#"',
        'http://www.yahoo.com/yelp.html#bottom', 'http://www.yahoo.com?name=%00%01', 'http://www.yahoo.com?name=obi',
        'http://www.yahoo.com?name=obi&', 'http://www.yahoo.com?name=obi&type=',
        'http://www.yahoo.com?name=obi+wan&status=jedi', 'http://www.yahoo.com?onery', 'http:g', 'http:this',
        'https://www.yahoo.com/', 'ldap://[2001:db8::7]/c=GB?objectClass?one', 'lv2.h', 'mailto:John.Doe@example.com',
        'mailto:local@domain.org', 'mailto:local@domain.org#frag', 'mailto:user@host?subject=blah', 'mini1.xml',
        'news:comp.infosystems.www.servers.unix', 'q%3Ar', 'q/r', 'q/r#', 'q/r#s', 'q/r#s/t', 's',
        'tel:+1-816-555-1212', 'telnet://192.0.2.16:80/', 'urn:oasis:names:specification:docbook:dtd:xml:4.1.2', 'y/z',
        'y?q', 'z/', '\ue000']

for workset in ('parse', 'resolve', 'relativize'):
    corpus_dir = Path('corpus') / workset
    corpus_dir.mkdir(parents=True, exist_ok=True)
    for l in data:
        hash = hashlib.sha256()
        hash.update(l.encode())
        (corpus_dir / hash.hexdigest()).write_text(l)
