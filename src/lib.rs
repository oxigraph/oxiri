//! Utilities to validate and resolve IRIs following [RFC 3987](https://www.ietf.org/rfc/rfc3987).
//!
//! ```
//! use oxiri::Iri;
//!
//! // Parse and validate base IRI
//! let base_iri = Iri::parse("http://foo.com/bar/baz").unwrap();
//!
//! // Validate and resolve relative IRI
//! let iri = base_iri.resolve("bat#foo").unwrap();
//! assert_eq!(iri.as_str(), "http://foo.com/bar/bat#foo");
//!
//! // Extract IRI components
//! assert_eq!(iri.scheme(), Some("http"));
//! assert_eq!(iri.authority(), Some("foo.com"));
//! assert_eq!(iri.path(), "/bar/bat");
//! assert_eq!(iri.query(), None);
//! assert_eq!(iri.fragment(), Some("foo"));
//! ```
#![deny(
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused_qualifications
)]

use std::borrow::{Borrow, Cow};
use std::cmp::Ordering;
use std::error::Error;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::net::{AddrParseError, Ipv6Addr};
use std::ops::Deref;
use std::str::{Chars, FromStr};

/// A [RFC 3987](https://www.ietf.org/rfc/rfc3987) IRI.
///
/// ```
/// use oxiri::Iri;
///
/// // Parse and validate base IRI
/// let base_iri = Iri::parse("http://foo.com/bar/baz").unwrap();
///
/// // Validate and resolve relative IRI
/// let iri = base_iri.resolve("bat#foo").unwrap();
/// assert_eq!(iri.into_inner(), "http://foo.com/bar/bat#foo");
/// ```
pub type Iri<T> = ParsedIri<T, false>;

#[derive(Clone, Copy)]
pub struct ParsedIri<T, const ABSOLUTE: bool> {
    iri: T,
    positions: IriElementsPositions,
}

impl<T: Deref<Target = str>, const ABSOLUTE: bool> ParsedIri<T, ABSOLUTE> {
    /// Parses and validates the IRI following [RFC 3987](https://www.ietf.org/rfc/rfc3987) `IRI` syntax.
    ///
    /// This operation keeps internally the `iri` parameter and does not allocate.
    ///
    /// ```
    /// use oxiri::Iri;
    ///
    /// Iri::parse("http://foo.com/bar/baz").unwrap();
    /// ```
    pub fn parse(iri: T) -> Result<Self, IriParseError> {
        let mode: ParseMode<'_> = ParseMode::Absolute;
        let positions = IriParser::parse(&iri, mode, &mut VoidOutputBuffer::default())?;
        Ok(Self { iri, positions })
    }

    /// Parses and validates the IRI following [RFC 3987](https://www.ietf.org/rfc/rfc3987) `irelative-ref` syntax.
    ///
    /// This operation keeps internally the `iri` parameter and does not allocate.
    ///
    /// ```
    /// use oxiri::Iri;
    ///
    /// Iri::parse("http://foo.com/bar/baz").unwrap();
    /// ```
    pub fn parse_relative(iri: T) -> Result<Self, IriParseError> {
        let mode: ParseMode<'_> = ParseMode::Relative;
        let positions = IriParser::parse(&iri, mode, &mut VoidOutputBuffer::default())?;
        Ok(Self { iri, positions })
    }

    /// Validates and resolved a relative IRI against the current IRI
    /// following [RFC 3986](https://www.ietf.org/rfc/rfc3986) relative URI resolution algorithm.
    ///
    /// ```
    /// use oxiri::Iri;
    ///
    /// let base_iri = Iri::parse("http://foo.com/bar/baz").unwrap();
    /// let iri = base_iri.resolve("bat#foo").unwrap();
    /// assert_eq!(iri.into_inner(), "http://foo.com/bar/bat#foo");
    /// ```
    pub fn resolve(&self, iri: &str) -> Result<ParsedIri<String, ABSOLUTE>, IriParseError> {
        let mut target_buffer = String::with_capacity(self.iri.len() + iri.len());
        let mode = ParseMode::WithBase(self.as_ref().into_iri_ref());
        let positions = IriParser::parse(iri, mode, &mut target_buffer)?;
        Ok(ParsedIri {
            iri: target_buffer,
            positions,
        })
    }

    /// Validates and resolved a relative IRI against the current IRI
    /// following [RFC 3986](https://www.ietf.org/rfc/rfc3986) relative URI resolution algorithm.
    ///
    /// It outputs the resolved IRI into `target_buffer` to avoid any memory allocation.
    ///
    /// ```
    /// use oxiri::Iri;
    ///
    /// let base_iri = Iri::parse("http://foo.com/bar/baz").unwrap();
    /// let mut result = String::default();
    /// let iri = base_iri.resolve_into("bat#foo", &mut result).unwrap();
    /// assert_eq!(result, "http://foo.com/bar/bat#foo");
    /// ```
    pub fn resolve_into(&self, iri: &str, target_buffer: &mut String) -> Result<(), IriParseError> {
        IriParser::parse(
            iri,
            ParseMode::WithBase(self.as_ref().into_iri_ref()),
            target_buffer,
        )?;
        Ok(())
    }

    /// Returns an IRI borrowing this IRI's text
    #[inline]
    pub fn as_ref(&self) -> ParsedIri<&str, ABSOLUTE> {
        ParsedIri {
            iri: &self.iri,
            positions: self.positions,
        }
    }

    /// Convert into an IRI reference (i.e. allowed to be relative).
    #[inline]
    pub fn into_iri_ref(self) -> ParsedIri<T, false> {
        ParsedIri {
            iri: self.iri,
            positions: self.positions,
        }
    }

    /// Returns the underlying IRI representation.
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.iri
    }

    /// Returns the underlying IRI representation.
    #[inline]
    pub fn into_inner(self) -> T {
        self.iri
    }

    /// Whether this IRI is an absolute IRI reference or not.
    ///
    /// NB: this can only return false if this IRI was parsed with [`Iri::parse_relative`]
    #[inline]
    pub fn is_absolute(&self) -> bool {
        self.positions.scheme_end != 0
    }

    /// Returns the IRI scheme if it exists.
    ///
    /// Beware: the scheme case is not normalized. Use case insensitive comparisons if you look for a specific scheme.
    ///
    /// NB: only relative IRI references have no scheme
    /// (see [`Iri::parse_relative`] and [`Iri::is_absolute`]).
    ///
    /// ```
    /// use oxiri::Iri;
    ///
    /// let iri = Iri::parse("hTTp://example.com").unwrap();
    /// assert_eq!(Some("hTTp"), iri.scheme());
    /// ```
    #[inline]
    pub fn scheme(&self) -> Option<&str> {
        if self.positions.scheme_end == 0 {
            None
        } else {
            Some(&self.iri[..self.positions.scheme_end - 1])
        }
    }

    /// Returns the IRI authority if it exists.
    ///
    /// Beware: the host case is not normalized. Use case insensitive comparisons if you look for a specific host.
    ///
    /// ```
    /// use oxiri::Iri;
    ///
    /// let http = Iri::parse("http://foo:pass@example.com:80/my/path").unwrap();
    /// assert_eq!(http.authority(), Some("foo:pass@example.com:80"));
    ///
    /// let mailto = Iri::parse("mailto:foo@bar.com").unwrap();
    /// assert_eq!(mailto.authority(), None);
    /// ```
    #[inline]
    pub fn authority(&self) -> Option<&str> {
        if self.positions.scheme_end + 2 > self.positions.authority_end {
            None
        } else {
            Some(&self.iri[self.positions.scheme_end + 2..self.positions.authority_end])
        }
    }

    /// Returns the IRI path.
    ///
    /// ```
    /// use oxiri::Iri;
    ///
    /// let http = Iri::parse("http://foo:pass@example.com:80/my/path?foo=bar").unwrap();
    /// assert_eq!(http.path(), "/my/path");
    ///
    /// let mailto = Iri::parse("mailto:foo@bar.com").unwrap();
    /// assert_eq!(mailto.path(), "foo@bar.com");
    /// ```
    #[inline]
    pub fn path(&self) -> &str {
        &self.iri[self.positions.authority_end..self.positions.path_end]
    }

    /// Returns the IRI query if it exists.
    ///
    /// ```
    /// use oxiri::Iri;
    ///
    /// let iri = Iri::parse("http://example.com/my/path?query=foo#frag").unwrap();
    /// assert_eq!(iri.query(), Some("query=foo"));
    /// ```
    #[inline]
    pub fn query(&self) -> Option<&str> {
        if self.positions.path_end >= self.positions.query_end {
            None
        } else {
            Some(&self.iri[self.positions.path_end + 1..self.positions.query_end])
        }
    }

    /// Returns the IRI fragment if it exists.
    ///
    /// ```
    /// use oxiri::Iri;
    ///
    /// let iri = Iri::parse("http://example.com/my/path?query=foo#frag").unwrap();
    /// assert_eq!(iri.fragment(), Some("frag"));
    /// ```
    #[inline]
    pub fn fragment(&self) -> Option<&str> {
        if self.positions.query_end >= self.iri.len() {
            None
        } else {
            Some(&self.iri[self.positions.query_end + 1..])
        }
    }
}

impl<Lft: PartialEq<Rhs>, Rhs, const A1: bool, const A2: bool> PartialEq<ParsedIri<Rhs, A2>>
    for ParsedIri<Lft, A1>
{
    #[inline]
    fn eq(&self, other: &ParsedIri<Rhs, A2>) -> bool {
        self.iri.eq(&other.iri)
    }
}

impl<T: PartialEq<str>, const A: bool> PartialEq<str> for ParsedIri<T, A> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.iri.eq(other)
    }
}

impl<'a, T: PartialEq<&'a str>, const A: bool> PartialEq<&'a str> for ParsedIri<T, A> {
    #[inline]
    fn eq(&self, other: &&'a str) -> bool {
        self.iri.eq(other)
    }
}

impl<T: PartialEq<String>, const A: bool> PartialEq<String> for ParsedIri<T, A> {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.iri.eq(other)
    }
}

impl<'a, T: PartialEq<Cow<'a, str>>, const A: bool> PartialEq<Cow<'a, str>> for ParsedIri<T, A> {
    #[inline]
    fn eq(&self, other: &Cow<'a, str>) -> bool {
        self.iri.eq(other)
    }
}

impl<T: PartialEq<str>, const A: bool> PartialEq<ParsedIri<T, A>> for str {
    #[inline]
    fn eq(&self, other: &ParsedIri<T, A>) -> bool {
        other.iri.eq(self)
    }
}

impl<'a, T: PartialEq<&'a str>, const A: bool> PartialEq<ParsedIri<T, A>> for &'a str {
    #[inline]
    fn eq(&self, other: &ParsedIri<T, A>) -> bool {
        other.iri.eq(self)
    }
}

impl<T: PartialEq<String>, const A: bool> PartialEq<ParsedIri<T, A>> for String {
    #[inline]
    fn eq(&self, other: &ParsedIri<T, A>) -> bool {
        other.iri.eq(self)
    }
}

impl<'a, T: PartialEq<Cow<'a, str>>, const A: bool> PartialEq<ParsedIri<T, A>> for Cow<'a, str> {
    #[inline]
    fn eq(&self, other: &ParsedIri<T, A>) -> bool {
        other.iri.eq(self)
    }
}

impl<T: Eq, const A: bool> Eq for ParsedIri<T, A> {}

impl<T: Hash, const A: bool> Hash for ParsedIri<T, A> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.iri.hash(state)
    }
}

impl<T: PartialOrd, const A: bool> PartialOrd for ParsedIri<T, A> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.iri.partial_cmp(&other.iri)
    }
}

impl<T: Ord, const A: bool> Ord for ParsedIri<T, A> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.iri.cmp(&other.iri)
    }
}

impl<T: Deref<Target = str>, const A: bool> Deref for ParsedIri<T, A> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.iri.deref()
    }
}

impl<T: AsRef<str>, const A: bool> AsRef<str> for ParsedIri<T, A> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.iri.as_ref()
    }
}

impl<T: Borrow<str>, const A: bool> Borrow<str> for ParsedIri<T, A> {
    #[inline]
    fn borrow(&self) -> &str {
        self.iri.borrow()
    }
}

impl<T: fmt::Debug, const A: bool> fmt::Debug for ParsedIri<T, A> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.iri.fmt(f)
    }
}

impl<T: fmt::Display, const A: bool> fmt::Display for ParsedIri<T, A> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.iri.fmt(f)
    }
}

impl<const A: bool> FromStr for ParsedIri<String, A> {
    type Err = IriParseError;

    #[inline]
    fn from_str(iri: &str) -> Result<Self, IriParseError> {
        Self::parse(iri.to_owned())
    }
}

impl<'a, const A: bool> From<ParsedIri<&'a str, A>> for ParsedIri<String, A> {
    #[inline]
    fn from(iri: ParsedIri<&'a str, A>) -> Self {
        Self {
            iri: iri.iri.into(),
            positions: iri.positions,
        }
    }
}

impl<'a, const A: bool> From<ParsedIri<Cow<'a, str>, A>> for ParsedIri<String, A> {
    #[inline]
    fn from(iri: ParsedIri<Cow<'a, str>, A>) -> Self {
        Self {
            iri: iri.iri.into(),
            positions: iri.positions,
        }
    }
}

impl<const A: bool> From<ParsedIri<Box<str>, A>> for ParsedIri<String, A> {
    #[inline]
    fn from(iri: ParsedIri<Box<str>, A>) -> Self {
        Self {
            iri: iri.iri.into(),
            positions: iri.positions,
        }
    }
}

impl<'a, const A: bool> From<ParsedIri<&'a str, A>> for ParsedIri<Cow<'a, str>, A> {
    #[inline]
    fn from(iri: ParsedIri<&'a str, A>) -> Self {
        Self {
            iri: iri.iri.into(),
            positions: iri.positions,
        }
    }
}

impl<'a, const A: bool> From<ParsedIri<String, A>> for ParsedIri<Cow<'a, str>, A> {
    #[inline]
    fn from(iri: ParsedIri<String, A>) -> Self {
        Self {
            iri: iri.iri.into(),
            positions: iri.positions,
        }
    }
}

impl<'a, const A: bool> From<&'a ParsedIri<String, A>> for ParsedIri<&'a str, A> {
    #[inline]
    fn from(iri: &'a ParsedIri<String, A>) -> Self {
        Self {
            iri: &iri.iri,
            positions: iri.positions,
        }
    }
}

impl<'a, const A: bool> From<&'a ParsedIri<Cow<'a, str>, A>> for ParsedIri<&'a str, A> {
    #[inline]
    fn from(iri: &'a ParsedIri<Cow<'a, str>, A>) -> Self {
        Self {
            iri: &iri.iri,
            positions: iri.positions,
        }
    }
}

/// An error raised during [`Iri`](struct.Iri.html) validation.
#[derive(Debug)]
pub struct IriParseError {
    kind: IriParseErrorKind,
}

impl fmt::Display for IriParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            IriParseErrorKind::NoScheme => write!(f, "No scheme found in an absolute IRI"),
            IriParseErrorKind::InvalidHostCharacter(c) => {
                write!(f, "Invalid character '{}' in host", c)
            }
            IriParseErrorKind::InvalidHostIp(e) => write!(f, "Invalid host IP ({})", e),
            IriParseErrorKind::InvalidPortCharacter(c) => write!(f, "Invalid character '{}'", c),
            IriParseErrorKind::InvalidIriCodePoint(c) => {
                write!(f, "Invalid IRI code point '{}'", c)
            }
            IriParseErrorKind::InvalidPercentEncoding(cs) => write!(
                f,
                "Invalid IRI percent encoding '{}'",
                cs.iter().flatten().cloned().collect::<String>()
            ),
        }
    }
}

impl Error for IriParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let IriParseErrorKind::InvalidHostIp(e) = &self.kind {
            Some(e)
        } else {
            None
        }
    }
}

#[derive(Debug)]
enum IriParseErrorKind {
    NoScheme,
    InvalidHostCharacter(char),
    InvalidHostIp(AddrParseError),
    InvalidPortCharacter(char),
    InvalidIriCodePoint(char),
    InvalidPercentEncoding([Option<char>; 3]),
}

#[derive(Debug, Clone, Copy)]
struct IriElementsPositions {
    scheme_end: usize,
    authority_end: usize,
    path_end: usize,
    query_end: usize,
}

trait OutputBuffer {
    fn push(&mut self, c: char);

    fn push_str(&mut self, s: &str);

    fn clear(&mut self);

    fn truncate(&mut self, new_len: usize);

    fn len(&self) -> usize;

    fn as_str(&self) -> &str;
}

#[derive(Default)]
struct VoidOutputBuffer {
    len: usize,
}

impl OutputBuffer for VoidOutputBuffer {
    #[inline]
    fn push(&mut self, c: char) {
        self.len += c.len_utf8();
    }

    #[inline]
    fn push_str(&mut self, s: &str) {
        self.len += s.len();
    }

    #[inline]
    fn clear(&mut self) {
        self.len = 0;
    }

    #[inline]
    fn truncate(&mut self, new_len: usize) {
        self.len = new_len;
    }

    #[inline]
    fn len(&self) -> usize {
        self.len
    }

    #[inline]
    fn as_str(&self) -> &str {
        ""
    }
}

impl OutputBuffer for String {
    #[inline]
    fn push(&mut self, c: char) {
        self.push(c);
    }

    #[inline]
    fn push_str(&mut self, s: &str) {
        self.push_str(s);
    }

    #[inline]
    fn clear(&mut self) {
        self.clear();
    }

    #[inline]
    fn truncate(&mut self, new_len: usize) {
        self.truncate(new_len);
    }

    #[inline]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn as_str(&self) -> &str {
        self.as_str()
    }
}

struct ParserInput<'a> {
    value: Chars<'a>,
    position: usize,
}
impl<'a> ParserInput<'a> {
    #[inline]
    fn next(&mut self) -> Option<char> {
        if let Some(head) = self.value.next() {
            self.position += head.len_utf8();
            Some(head)
        } else {
            None
        }
    }

    #[inline]
    fn front(&self) -> Option<char> {
        self.value.clone().next()
    }

    #[inline]
    fn starts_with(&self, c: char) -> bool {
        self.value.as_str().starts_with(c)
    }
}

enum ParseMode<'a> {
    Absolute,
    Relative,
    WithBase(ParsedIri<&'a str, false>),
}

/// parser implementing https://url.spec.whatwg.org/#concept-basic-url-parser without the normalization or backward compatibility bits to comply with RFC 3987
///
/// A sub function takes care of each state
struct IriParser<'a, O: OutputBuffer> {
    iri: &'a str,
    mode: ParseMode<'a>,
    input: ParserInput<'a>,
    output: &'a mut O,
    output_positions: IriElementsPositions,
    input_scheme_end: usize,
}

impl<'a, O: OutputBuffer> IriParser<'a, O> {
    fn parse(
        iri: &'a str,
        mode: ParseMode<'a>,
        output: &'a mut O,
    ) -> Result<IriElementsPositions, IriParseError> {
        let mut parser = Self {
            iri,
            mode,
            input: ParserInput {
                value: iri.chars(),
                position: 0,
            },
            output,
            output_positions: IriElementsPositions {
                scheme_end: 0,
                authority_end: 0,
                path_end: 0,
                query_end: 0,
            },
            input_scheme_end: 0,
        };
        parser.parse_scheme_start()?;
        Ok(parser.output_positions)
    }

    fn parse_scheme_start(&mut self) -> Result<(), IriParseError> {
        match self.input.front() {
            Some(':') => self.parse_error(IriParseErrorKind::NoScheme),
            Some(c) if c.is_ascii_alphabetic() => self.parse_scheme(),
            _ => self.parse_relative(),
        }
    }

    fn parse_scheme(&mut self) -> Result<(), IriParseError> {
        loop {
            let c = self.input.next();
            match c {
                Some(c) if c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.' => {
                    self.output.push(c)
                }
                Some(':') => {
                    self.output.push(':');
                    self.output_positions.scheme_end = self.output.len();
                    self.input_scheme_end = self.input.position;
                    return if self.input.starts_with('/') {
                        self.input.next();
                        self.output.push('/');
                        self.parse_path_or_authority()
                    } else {
                        self.output_positions.authority_end = self.output.len();
                        self.parse_path()
                    };
                }
                _ => {
                    self.input = ParserInput {
                        value: self.iri.chars(),
                        position: 0,
                    }; // reset
                    self.output.clear();
                    return self.parse_relative();
                }
            }
        }
    }

    fn parse_path_or_authority(&mut self) -> Result<(), IriParseError> {
        if self.input.starts_with('/') {
            self.input.next();
            self.output.push('/');
            self.parse_authority()
        } else {
            self.output_positions.authority_end = self.output.len() - 1;
            self.parse_path()
        }
    }

    fn parse_relative(&mut self) -> Result<(), IriParseError> {
        use ParseMode::*;
        match self.mode {
            WithBase(base) => match self.input.front() {
                None => {
                    self.output.push_str(&base.iri[..base.positions.query_end]);
                    self.output_positions.scheme_end = base.positions.scheme_end;
                    self.output_positions.authority_end = base.positions.authority_end;
                    self.output_positions.path_end = base.positions.path_end;
                    self.output_positions.query_end = base.positions.query_end;
                    Ok(())
                }
                Some('/') => {
                    self.input.next();
                    self.parse_relative_slash(&base)
                }
                Some('?') => {
                    self.input.next();
                    self.output.push_str(&base.iri[..base.positions.path_end]);
                    self.output.push('?');
                    self.output_positions.scheme_end = base.positions.scheme_end;
                    self.output_positions.authority_end = base.positions.authority_end;
                    self.output_positions.path_end = base.positions.path_end;
                    self.parse_query()
                }
                Some('#') => {
                    self.input.next();
                    self.output.push_str(&base.iri[..base.positions.query_end]);
                    self.output_positions.scheme_end = base.positions.scheme_end;
                    self.output_positions.authority_end = base.positions.authority_end;
                    self.output_positions.path_end = base.positions.path_end;
                    self.output_positions.query_end = base.positions.query_end;
                    self.output.push('#');
                    self.parse_fragment()
                }
                _ => {
                    self.output.push_str(&base.iri[..base.positions.path_end]);
                    self.output_positions.scheme_end = base.positions.scheme_end;
                    self.output_positions.authority_end = base.positions.authority_end;
                    self.output_positions.path_end = base.positions.path_end;
                    self.remove_last_segment_leaving_slash();
                    self.parse_path()
                }
            },
            Relative => {
                self.output_positions.scheme_end = 0;
                self.input_scheme_end = 0;
                if self.input.starts_with('/') {
                    self.input.next();
                    self.output.push('/');
                    self.parse_path_or_authority()
                } else {
                    self.output_positions.authority_end = 0;
                    self.parse_path()
                }
            }
            Absolute => self.parse_error(IriParseErrorKind::NoScheme),
        }
    }

    fn parse_relative_slash(
        &mut self,
        base: &ParsedIri<&'a str, false>,
    ) -> Result<(), IriParseError> {
        if self.input.starts_with('/') {
            self.input.next();
            self.output.push_str(&base.iri[..base.positions.scheme_end]);
            self.output_positions.scheme_end = base.positions.scheme_end;
            self.output.push('/');
            self.output.push('/');
            self.parse_authority()
        } else {
            self.output
                .push_str(&base.iri[..base.positions.authority_end]);
            self.output.push('/');
            self.output_positions.scheme_end = base.positions.scheme_end;
            self.output_positions.authority_end = base.positions.authority_end;
            self.parse_path()
        }
    }

    fn parse_authority(&mut self) -> Result<(), IriParseError> {
        // @ are not allowed in IRI authorities so not need to take care of ambiguities
        loop {
            let c = self.input.next();
            match c {
                Some('@') => {
                    self.output.push('@');
                    return self.parse_host();
                }
                None | Some('[') | Some('/') | Some('?') | Some('#') => {
                    self.input = ParserInput {
                        value: self.iri[self.input_scheme_end + 2..].chars(),
                        position: self.input_scheme_end + 2,
                    };
                    self.output.truncate(self.output_positions.scheme_end + 2);
                    return self.parse_host();
                }
                Some(c) => {
                    self.read_url_codepoint_or_echar(c)?;
                }
            }
        }
    }

    fn parse_host(&mut self) -> Result<(), IriParseError> {
        if self.input.starts_with('[') {
            // IP v6
            while let Some(c) = self.input.next() {
                self.output.push(c);
                if c == ']' {
                    if let Err(error) = Ipv6Addr::from_str(
                        &self.iri[self.input_scheme_end + 3..self.input.position - 1],
                    ) {
                        return self.parse_error(IriParseErrorKind::InvalidHostIp(error));
                    }

                    let c = self.input.next();
                    return match c {
                        Some(':') => {
                            self.output.push(':');
                            self.parse_port()
                        }
                        None | Some('/') | Some('?') | Some('#') => {
                            self.output_positions.authority_end = self.output.len();
                            self.parse_path_start(c)
                        }
                        Some(c) => self.parse_error(IriParseErrorKind::InvalidHostCharacter(c)),
                    };
                }
            }
            self.parse_error(IriParseErrorKind::InvalidHostCharacter('['))
        } else {
            // Other host
            loop {
                let c = self.input.next();
                match c {
                    Some(':') => {
                        self.output.push(':');
                        return self.parse_port();
                    }
                    None | Some('/') | Some('?') | Some('#') => {
                        self.output_positions.authority_end = self.output.len();
                        return self.parse_path_start(c);
                    }
                    Some(c) => self.read_url_codepoint_or_echar(c)?,
                }
            }
        }
    }

    fn parse_port(&mut self) -> Result<(), IriParseError> {
        loop {
            let c = self.input.next();
            match c {
                Some(c) if c.is_ascii_digit() => self.output.push(c),
                Some('/') | Some('?') | Some('#') | None => {
                    self.output_positions.authority_end = self.output.len();
                    return self.parse_path_start(c);
                }
                Some(c) => return self.parse_error(IriParseErrorKind::InvalidPortCharacter(c)),
            }
        }
    }

    fn parse_path_start(&mut self, c: Option<char>) -> Result<(), IriParseError> {
        match c {
            None => {
                self.output_positions.path_end = self.output.len();
                self.output_positions.query_end = self.output.len();
                Ok(())
            }
            Some('?') => {
                self.output_positions.path_end = self.output.len();
                self.output.push('?');
                self.parse_query()
            }
            Some('#') => {
                self.output_positions.path_end = self.output.len();
                self.output_positions.query_end = self.output.len();
                self.output.push('#');
                self.parse_fragment()
            }
            Some('/') => {
                self.output.push('/');
                self.parse_path()
            }
            Some(c) => {
                self.read_url_codepoint_or_echar(c)?;
                self.parse_path()
            }
        }
    }

    fn parse_path(&mut self) -> Result<(), IriParseError> {
        loop {
            let c = self.input.next();
            match c {
                None | Some('/') | Some('?') | Some('#') => {
                    if self.output.as_str().ends_with("/..") {
                        self.remove_last_segment();
                        self.remove_last_segment();
                        self.output.push('/');
                    } else if self.output.as_str().ends_with("/.") {
                        self.remove_last_segment();
                        self.output.push('/');
                    } else if c == Some('/') {
                        self.output.push('/');
                    }

                    if c == Some('?') {
                        self.output_positions.path_end = self.output.len();
                        self.output.push('?');
                        return self.parse_query();
                    } else if c == Some('#') {
                        self.output_positions.path_end = self.output.len();
                        self.output_positions.query_end = self.output.len();
                        self.output.push('#');
                        return self.parse_fragment();
                    } else if c == None {
                        self.output_positions.path_end = self.output.len();
                        self.output_positions.query_end = self.output.len();
                        return Ok(());
                    }
                }
                Some(c) => self.read_url_codepoint_or_echar(c)?,
            }
        }
    }

    fn parse_query(&mut self) -> Result<(), IriParseError> {
        while let Some(c) = self.input.next() {
            if c == '#' {
                self.output_positions.query_end = self.output.len();
                self.output.push('#');
                return self.parse_fragment();
            } else {
                self.read_url_query_codepoint_or_echar(c)?
            }
        }
        self.output_positions.query_end = self.output.len();
        Ok(())
    }

    fn parse_fragment(&mut self) -> Result<(), IriParseError> {
        while let Some(c) = self.input.next() {
            self.read_url_codepoint_or_echar(c)?
        }
        Ok(())
    }

    fn remove_last_segment(&mut self) {
        let last_slash_position = self.output.as_str()[self.output_positions.authority_end..]
            .rfind('/')
            .unwrap_or(0);
        self.output
            .truncate(last_slash_position + self.output_positions.authority_end)
    }

    fn remove_last_segment_leaving_slash(&mut self) {
        let last_slash_position =
            self.output.as_str()[self.output_positions.authority_end..].rfind('/');
        let truncate_point = self.output_positions.authority_end
            + match last_slash_position {
                None => 0,
                Some(pos) => pos + 1,
            };
        self.output.truncate(truncate_point)
    }

    #[inline]
    fn read_url_codepoint_or_echar(&mut self, c: char) -> Result<(), IriParseError> {
        if c == '%' {
            self.read_echar()
        } else if is_url_code_point(c) {
            self.output.push(c);
            Ok(())
        } else {
            self.parse_error(IriParseErrorKind::InvalidIriCodePoint(c))
        }
    }

    #[inline]
    fn read_url_query_codepoint_or_echar(&mut self, c: char) -> Result<(), IriParseError> {
        if c == '%' {
            self.read_echar()
        } else if is_url_query_code_point(c) {
            self.output.push(c);
            Ok(())
        } else {
            self.parse_error(IriParseErrorKind::InvalidIriCodePoint(c))
        }
    }

    #[inline]
    fn read_echar(&mut self) -> Result<(), IriParseError> {
        let c1 = self.input.next();
        let c2 = self.input.next();
        if c1.map_or(false, |c| c.is_ascii_hexdigit())
            && c2.map_or(false, |c| c.is_ascii_hexdigit())
        {
            self.output.push('%');
            self.output.push(c1.unwrap());
            self.output.push(c2.unwrap());
            Ok(())
        } else {
            self.parse_error(IriParseErrorKind::InvalidPercentEncoding([
                Some('%'),
                c1,
                c2,
            ]))
        }
    }

    #[inline]
    fn parse_error<T>(&self, kind: IriParseErrorKind) -> Result<T, IriParseError> {
        Err(IriParseError { kind })
    }
}

fn is_url_code_point(c: char) -> bool {
    matches!(c,
        'a'..='z'
        | 'A'..='Z'
        | '0'..='9'
        | '!'
        | '$'
        | '&'
        | '\''
        | '('
        | ')'
        | '*'
        | '+'
        | ','
        | '-'
        | '.'
        | '/'
        | ':'
        | ';'
        | '='
        | '?'
        | '@'
        | '_'
        | '~'
        | '\u{A0}'..='\u{D7FF}'
        | '\u{FDF0}'..='\u{FFFD}'
        | '\u{10000}'..='\u{1FFFD}'
        | '\u{20000}'..='\u{2FFFD}'
        | '\u{30000}'..='\u{3FFFD}'
        | '\u{40000}'..='\u{4FFFD}'
        | '\u{50000}'..='\u{5FFFD}'
        | '\u{60000}'..='\u{6FFFD}'
        | '\u{70000}'..='\u{7FFFD}'
        | '\u{80000}'..='\u{8FFFD}'
        | '\u{90000}'..='\u{9FFFD}'
        | '\u{A0000}'..='\u{AFFFD}'
        | '\u{B0000}'..='\u{BFFFD}'
        | '\u{C0000}'..='\u{CFFFD}'
        | '\u{D0000}'..='\u{DFFFD}'
        | '\u{E1000}'..='\u{EFFFD}'
    )
}

fn is_url_query_code_point(c: char) -> bool {
    matches!(c,
        'a'..='z'
        | 'A'..='Z'
        | '0'..='9'
        | '!'
        | '$'
        | '&'
        | '\''
        | '('
        | ')'
        | '*'
        | '+'
        | ','
        | '-'
        | '.'
        | '/'
        | ':'
        | ';'
        | '='
        | '?'
        | '@'
        | '_'
        | '~'
        | '\u{A0}'..='\u{D7FF}'
        | '\u{E000}'..='\u{FDCF}'
        | '\u{FDF0}'..='\u{FFFD}'
        | '\u{10000}'..='\u{1FFFD}'
        | '\u{20000}'..='\u{2FFFD}'
        | '\u{30000}'..='\u{3FFFD}'
        | '\u{40000}'..='\u{4FFFD}'
        | '\u{50000}'..='\u{5FFFD}'
        | '\u{60000}'..='\u{6FFFD}'
        | '\u{70000}'..='\u{7FFFD}'
        | '\u{80000}'..='\u{8FFFD}'
        | '\u{90000}'..='\u{9FFFD}'
        | '\u{A0000}'..='\u{AFFFD}'
        | '\u{B0000}'..='\u{BFFFD}'
        | '\u{C0000}'..='\u{CFFFD}'
        | '\u{D0000}'..='\u{DFFFD}'
        | '\u{E1000}'..='\u{EFFFD}'
        | '\u{F0000}'..='\u{FFFFD}'
        | '\u{100000}'..='\u{10FFFD}'
    )
}
