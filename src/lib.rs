//! Utilities to validate and resolve IRIs following [RFC 3987](https://www.ietf.org/rfc/rfc3987).
//!
//! Example:
//! ```
//! use oxiri::Iri;
//!
//! // Parse and validate base IRI
//! let base_iri = Iri::parse("http://foo.com/bar/baz").unwrap();
//!
//! // Validate and resolve relative IRI
//! let iri = base_iri.resolve("bat#foo").unwrap();
//! assert_eq!("http://foo.com/bar/bat#foo", iri.into_inner())
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

use std::error::Error;
use std::fmt;
use std::net::{AddrParseError, Ipv6Addr};
use std::ops::Deref;
use std::str::{Chars, FromStr};

/// A [RFC 3987](https://www.ietf.org/rfc/rfc3987) IRI.
///
/// Example:
/// ```
/// use oxiri::Iri;
///
/// // Parse and validate base IRI
/// let base_iri = Iri::parse("http://foo.com/bar/baz").unwrap();
///
/// // Validate and resolve relative IRI
/// let iri = base_iri.resolve("bat#foo").unwrap();
/// assert_eq!("http://foo.com/bar/bat#foo", iri.into_inner());
/// ```
#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub struct Iri<T: Deref<Target = str>> {
    iri: T,
    positions: IriElementsPositions,
}

impl<T: Deref<Target = str>> Iri<T> {
    /// Parses and validates the IRI following [RFC 3987](https://www.ietf.org/rfc/rfc3987) IRI syntax.
    ///
    /// This operation keeps internally the `iri` parameter and does not allocate.
    ///
    /// Example:
    /// ```
    /// use oxiri::Iri;
    ///
    /// Iri::parse("http://foo.com/bar/baz").unwrap();
    /// ```
    pub fn parse(iri: T) -> Result<Self, IriParseError> {
        let base: Option<&Iri<&str>> = None;
        let positions = IriParser::parse(&iri, base, &mut VoidOutputBuffer::default())?;
        Ok(Self { iri, positions })
    }

    /// Validates and resolved a relative IRI against the current IRI
    /// following [RFC 3986](https://www.ietf.org/rfc/rfc3986) relative URI resolution algorithm.
    ///
    /// Example:
    /// ```
    /// use oxiri::Iri;
    ///
    /// let base_iri = Iri::parse("http://foo.com/bar/baz").unwrap();
    /// let iri = base_iri.resolve("bat#foo").unwrap();
    /// assert_eq!("http://foo.com/bar/bat#foo", iri.into_inner());
    /// ```
    pub fn resolve(&self, iri: &str) -> Result<Iri<String>, IriParseError> {
        let mut target_buffer = String::with_capacity(self.iri.len() + iri.len());
        let positions = IriParser::parse(iri, Some(&self), &mut target_buffer)?;
        Ok(Iri {
            iri: target_buffer,
            positions,
        })
    }

    /// Validates and resolved a relative IRI against the current IRI
    /// following [RFC 3986](https://www.ietf.org/rfc/rfc3986) relative URI resolution algorithm.
    ///
    /// It outputs the resolved IRI into `target_buffer` to avoid any memory allocation.
    ///
    /// Example:
    /// ```
    /// use oxiri::Iri;
    ///
    /// let base_iri = Iri::parse("http://foo.com/bar/baz").unwrap();
    /// let mut result = String::default();
    /// let iri = base_iri.resolve_into("bat#foo", &mut result).unwrap();
    /// assert_eq!("http://foo.com/bar/bat#foo", result);
    /// ```
    pub fn resolve_into(&self, iri: &str, target_buffer: &mut String) -> Result<(), IriParseError> {
        IriParser::parse(iri, Some(&self), target_buffer)?;
        Ok(())
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
}

impl<T: Deref<Target = str>> AsRef<T> for Iri<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        &self.iri
    }
}

impl<T: Deref<Target = str> + fmt::Display> fmt::Display for Iri<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.iri.fmt(f)
    }
}

impl FromStr for Iri<String> {
    type Err = IriParseError;

    #[inline]
    fn from_str(iri: &str) -> Result<Self, IriParseError> {
        Self::parse(iri.to_owned())
    }
}

/// An error raised during `Iri` validation.
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

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
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

/// parser implementing https://url.spec.whatwg.org/#concept-basic-url-parser without the normalization or backward compatibility bits to comply with RFC 3987
///
/// A sub function takes care of each state
struct IriParser<'a, BC: Deref<Target = str>, O: OutputBuffer> {
    iri: &'a str,
    base: Option<&'a Iri<BC>>,
    input: ParserInput<'a>,
    output: &'a mut O,
    output_positions: IriElementsPositions,
    input_scheme_end: usize,
}

impl<'a, BC: Deref<Target = str>, O: OutputBuffer> IriParser<'a, BC, O> {
    fn parse(
        iri: &'a str,
        base: Option<&'a Iri<BC>>,
        output: &'a mut O,
    ) -> Result<IriElementsPositions, IriParseError> {
        let mut parser = Self {
            iri,
            base,
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
        if let Some(base) = self.base {
            match self.input.front() {
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
                    self.parse_relative_slash()
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
                    self.remove_last_segment();
                    self.output.push('/');
                    self.parse_path()
                }
            }
        } else {
            self.parse_error(IriParseErrorKind::NoScheme)
        }
    }

    fn parse_relative_slash(&mut self) -> Result<(), IriParseError> {
        let base = self.base.unwrap();
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
                    self.output.push(c);
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
                self.output.push(c);
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
                self.read_url_codepoint_or_echar(c)?
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

    #[inline]
    fn read_url_codepoint_or_echar(&mut self, c: char) -> Result<(), IriParseError> {
        if c == '%' {
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
        } else if is_url_code_point(c) {
            self.output.push(c);
            Ok(())
        } else {
            self.parse_error(IriParseErrorKind::InvalidIriCodePoint(c))
        }
    }

    #[inline]
    fn parse_error<T>(&self, kind: IriParseErrorKind) -> Result<T, IriParseError> {
        Err(IriParseError { kind })
    }
}

fn is_url_code_point(c: char) -> bool {
    match c {
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
        | '\u{100000}'..='\u{10FFFD}' => true,
        _ => false,
    }
}
