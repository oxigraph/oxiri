#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![deny(unsafe_code)]

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::{Borrow, Cow};
use std::cmp::Ordering;
use std::convert::{TryFrom, TryInto};
use std::error::Error;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::net::{AddrParseError, Ipv6Addr};
use std::ops::Deref;
use std::str::{Chars, FromStr};

/// A [RFC 3987](https://www.ietf.org/rfc/rfc3987.html) IRI reference.
///
/// Instances of this type may be absolute or relative,
/// unlike [`Iri`].
///
/// ```
/// use oxiri::{Iri, IriRef};
///
/// // Parse and validate base IRI
/// let base_iri = IriRef::parse("../bar/baz")?;
///
/// // Validate and resolve relative IRI
/// let iri = base_iri.resolve("bat#foo")?;
/// assert_eq!(iri.into_inner(), "../bar/bat#foo");
///
/// // IriRef's *can* also be absolute.
/// assert!(IriRef::parse("http://foo.com/bar/baz").is_ok());
///
/// // It is possible to build an IriRef from an Iri object
/// IriRef::from(Iri::parse("http://foo.com/bar")?);
/// # Result::<(), oxiri::IriParseError>::Ok(())
/// ```
#[derive(Clone, Copy)]
pub struct IriRef<T> {
    iri: T,
    positions: IriElementsPositions,
}

impl<T: Deref<Target = str>> IriRef<T> {
    /// Parses and validates the IRI-reference following the grammar from [RFC 3987](https://www.ietf.org/rfc/rfc3987.html).
    ///
    /// This operation keeps internally the `iri` parameter and does not allocate.
    ///
    /// Use [`parse_unchecked`](Self::parse_unchecked) if you already know the IRI is valid to get faster processing.
    ///
    /// ```
    /// use oxiri::IriRef;
    ///
    /// IriRef::parse("//foo.com/bar/baz")?;
    /// # Result::<(), oxiri::IriParseError>::Ok(())
    /// ```
    pub fn parse(iri: T) -> Result<Self, IriParseError> {
        let positions = IriParser::<_, false>::parse(&iri, None, &mut VoidOutputBuffer::default())?;
        Ok(Self { iri, positions })
    }

    /// Variant of [`parse`](Self::parse) that assumes that the IRI is valid to skip validation.
    ///
    /// ```
    /// use oxiri::IriRef;
    ///
    /// IriRef::parse_unchecked("//foo.com/bar/baz");
    /// ```
    pub fn parse_unchecked(iri: T) -> Self {
        let positions =
            IriParser::<_, true>::parse(&iri, None, &mut VoidOutputBuffer::default()).unwrap();
        Self { iri, positions }
    }

    /// Validates and resolved a relative IRI against the current IRI
    /// following [RFC 3986](https://www.ietf.org/rfc/rfc3986.html) relative URI resolution algorithm.
    ///
    /// Use [`resolve_unchecked`](Self::resolve_unchecked) if you already know the IRI is valid to get faster processing.
    ///
    /// ```
    /// use oxiri::IriRef;
    ///
    /// let base_iri = IriRef::parse("//foo.com/bar/baz")?;
    /// let iri = base_iri.resolve("bat#foo")?;
    /// assert_eq!(iri.into_inner(), "//foo.com/bar/bat#foo");
    /// # Result::<(), oxiri::IriParseError>::Ok(())
    /// ```
    pub fn resolve(&self, iri: &str) -> Result<IriRef<String>, IriParseError> {
        let mut target_buffer = String::with_capacity(self.iri.len() + iri.len());
        let positions = IriParser::<_, false>::parse(iri, Some(self.as_ref()), &mut target_buffer)?;
        Ok(IriRef {
            iri: target_buffer,
            positions,
        })
    }

    /// Variant of [`resolve`](Self::resolve) that assumes that the IRI is valid to skip validation.
    ///
    /// ```
    /// use oxiri::IriRef;
    ///
    /// let base_iri = IriRef::parse_unchecked("//foo.com/bar/baz");
    /// let iri = base_iri.resolve_unchecked("bat#foo");
    /// assert_eq!(iri.into_inner(), "//foo.com/bar/bat#foo");
    /// ```
    pub fn resolve_unchecked(&self, iri: &str) -> IriRef<String> {
        let mut target_buffer = String::with_capacity(self.iri.len() + iri.len());
        let positions =
            IriParser::<_, true>::parse(iri, Some(self.as_ref()), &mut target_buffer).unwrap();
        IriRef {
            iri: target_buffer,
            positions,
        }
    }

    /// Validates and resolved a relative IRI against the current IRI
    /// following [RFC 3986](https://www.ietf.org/rfc/rfc3986.html) relative URI resolution algorithm.
    ///
    /// It outputs the resolved IRI into `target_buffer` to avoid any memory allocation.
    ///
    /// Use [`resolve_into_unchecked`](Self::resolve_into_unchecked) if you already know the IRI is valid to get faster processing.
    ///
    /// ```
    /// use oxiri::IriRef;
    ///
    /// let base_iri = IriRef::parse("//foo.com/bar/baz")?;
    /// let mut result = String::default();
    /// let iri = base_iri.resolve_into("bat#foo", &mut result)?;
    /// assert_eq!(result, "//foo.com/bar/bat#foo");
    /// # Result::<(), oxiri::IriParseError>::Ok(())
    /// ```
    pub fn resolve_into(&self, iri: &str, target_buffer: &mut String) -> Result<(), IriParseError> {
        IriParser::<_, false>::parse(iri, Some(self.as_ref()), target_buffer)?;
        Ok(())
    }

    /// Variant of [`resolve_into`](Self::resolve_into) that assumes that the IRI is valid to skip validation.
    ///
    /// ```
    /// use oxiri::IriRef;
    ///
    /// let base_iri = IriRef::parse_unchecked("//foo.com/bar/baz");
    /// let mut result = String::default();
    /// let iri = base_iri.resolve_into_unchecked("bat#foo", &mut result);
    /// assert_eq!(result, "//foo.com/bar/bat#foo");
    /// ```
    pub fn resolve_into_unchecked(&self, iri: &str, target_buffer: &mut String) {
        IriParser::<_, true>::parse(iri, Some(self.as_ref()), target_buffer).unwrap();
    }

    /// Returns an `IriRef` borrowing this IRI's text.
    #[inline]
    pub fn as_ref(&self) -> IriRef<&str> {
        IriRef {
            iri: &self.iri,
            positions: self.positions,
        }
    }

    /// Returns the underlying IRI representation.
    ///
    /// ```
    /// use oxiri::IriRef;
    ///
    /// let iri = IriRef::parse("//example.com/foo")?;
    /// assert_eq!(iri.as_str(), "//example.com/foo");
    /// # Result::<(), oxiri::IriParseError>::Ok(())
    /// ```
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.iri
    }

    /// Returns the underlying IRI representation.
    ///
    /// ```
    /// use oxiri::IriRef;
    ///
    /// let iri = IriRef::parse("//example.com/foo")?;
    /// assert_eq!(iri.into_inner(), "//example.com/foo");
    /// # Result::<(), oxiri::IriParseError>::Ok(())
    /// ```
    #[inline]
    pub fn into_inner(self) -> T {
        self.iri
    }

    /// Whether this IRI is an absolute IRI reference or not.
    ///
    /// ```
    /// use oxiri::IriRef;
    ///
    /// assert!(IriRef::parse("http://example.com/foo")?.is_absolute());
    /// assert!(!IriRef::parse("/foo")?.is_absolute());
    /// # Result::<(), oxiri::IriParseError>::Ok(())
    /// ```
    #[inline]
    pub fn is_absolute(&self) -> bool {
        self.positions.scheme_end != 0
    }

    /// Returns the IRI scheme if it exists.
    ///
    /// Beware: the scheme case is not normalized. Use case insensitive comparisons if you look for a specific scheme.
    /// ```
    /// use oxiri::IriRef;
    ///
    /// let iri = IriRef::parse("hTTp://example.com")?;
    /// assert_eq!(iri.scheme(), Some("hTTp"));
    /// # Result::<(), oxiri::IriParseError>::Ok(())
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
    /// use oxiri::IriRef;
    ///
    /// let http = IriRef::parse("http://foo:pass@example.com:80/my/path")?;
    /// assert_eq!(http.authority(), Some("foo:pass@example.com:80"));
    ///
    /// let mailto = IriRef::parse("mailto:foo@bar.com")?;
    /// assert_eq!(mailto.authority(), None);
    /// # Result::<(), oxiri::IriParseError>::Ok(())
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
    /// use oxiri::IriRef;
    ///
    /// let http = IriRef::parse("http://foo:pass@example.com:80/my/path?foo=bar")?;
    /// assert_eq!(http.path(), "/my/path");
    ///
    /// let mailto = IriRef::parse("mailto:foo@bar.com")?;
    /// assert_eq!(mailto.path(), "foo@bar.com");
    /// # Result::<(), oxiri::IriParseError>::Ok(())
    /// ```
    #[inline]
    pub fn path(&self) -> &str {
        &self.iri[self.positions.authority_end..self.positions.path_end]
    }

    /// Returns the IRI query if it exists.
    ///
    /// ```
    /// use oxiri::IriRef;
    ///
    /// let iri = IriRef::parse("http://example.com/my/path?query=foo#frag")?;
    /// assert_eq!(iri.query(), Some("query=foo"));
    /// # Result::<(), oxiri::IriParseError>::Ok(())
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
    /// use oxiri::IriRef;
    ///
    /// let iri = IriRef::parse("http://example.com/my/path?query=foo#frag")?;
    /// assert_eq!(iri.fragment(), Some("frag"));
    /// # Result::<(), oxiri::IriParseError>::Ok(())
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

impl<Lft: PartialEq<Rhs>, Rhs> PartialEq<IriRef<Rhs>> for IriRef<Lft> {
    #[inline]
    fn eq(&self, other: &IriRef<Rhs>) -> bool {
        self.iri.eq(&other.iri)
    }
}

impl<T: PartialEq<str>> PartialEq<str> for IriRef<T> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.iri.eq(other)
    }
}

impl<'a, T: PartialEq<&'a str>> PartialEq<&'a str> for IriRef<T> {
    #[inline]
    fn eq(&self, other: &&'a str) -> bool {
        self.iri.eq(other)
    }
}

impl<T: PartialEq<String>> PartialEq<String> for IriRef<T> {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.iri.eq(other)
    }
}

impl<'a, T: PartialEq<Cow<'a, str>>> PartialEq<Cow<'a, str>> for IriRef<T> {
    #[inline]
    fn eq(&self, other: &Cow<'a, str>) -> bool {
        self.iri.eq(other)
    }
}

impl<T: PartialEq<str>> PartialEq<IriRef<T>> for str {
    #[inline]
    fn eq(&self, other: &IriRef<T>) -> bool {
        other.iri.eq(self)
    }
}

impl<'a, T: PartialEq<&'a str>> PartialEq<IriRef<T>> for &'a str {
    #[inline]
    fn eq(&self, other: &IriRef<T>) -> bool {
        other.iri.eq(self)
    }
}

impl<T: PartialEq<String>> PartialEq<IriRef<T>> for String {
    #[inline]
    fn eq(&self, other: &IriRef<T>) -> bool {
        other.iri.eq(self)
    }
}

impl<'a, T: PartialEq<Cow<'a, str>>> PartialEq<IriRef<T>> for Cow<'a, str> {
    #[inline]
    fn eq(&self, other: &IriRef<T>) -> bool {
        other.iri.eq(self)
    }
}

impl<T: Eq> Eq for IriRef<T> {}

impl<T: Hash> Hash for IriRef<T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.iri.hash(state)
    }
}

impl<Lft: PartialOrd<Rhs>, Rhs> PartialOrd<IriRef<Rhs>> for IriRef<Lft> {
    #[inline]
    fn partial_cmp(&self, other: &IriRef<Rhs>) -> Option<Ordering> {
        self.iri.partial_cmp(&other.iri)
    }
}

impl<T: Ord> Ord for IriRef<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.iri.cmp(&other.iri)
    }
}

impl<T: Deref<Target = str>> Deref for IriRef<T> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.iri.deref()
    }
}

impl<T: AsRef<str>> AsRef<str> for IriRef<T> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.iri.as_ref()
    }
}

impl<T: Borrow<str>> Borrow<str> for IriRef<T> {
    #[inline]
    fn borrow(&self) -> &str {
        self.iri.borrow()
    }
}

impl<T: fmt::Debug> fmt::Debug for IriRef<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.iri.fmt(f)
    }
}

impl<T: fmt::Display> fmt::Display for IriRef<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.iri.fmt(f)
    }
}

impl FromStr for IriRef<String> {
    type Err = IriParseError;

    #[inline]
    fn from_str(iri: &str) -> Result<Self, IriParseError> {
        Self::parse(iri.to_owned())
    }
}

impl<'a> From<IriRef<&'a str>> for IriRef<String> {
    #[inline]
    fn from(iri: IriRef<&'a str>) -> Self {
        Self {
            iri: iri.iri.into(),
            positions: iri.positions,
        }
    }
}

impl<'a> From<IriRef<Cow<'a, str>>> for IriRef<String> {
    #[inline]
    fn from(iri: IriRef<Cow<'a, str>>) -> Self {
        Self {
            iri: iri.iri.into(),
            positions: iri.positions,
        }
    }
}

impl From<IriRef<Box<str>>> for IriRef<String> {
    #[inline]
    fn from(iri: IriRef<Box<str>>) -> Self {
        Self {
            iri: iri.iri.into(),
            positions: iri.positions,
        }
    }
}

impl<'a> From<IriRef<&'a str>> for IriRef<Cow<'a, str>> {
    #[inline]
    fn from(iri: IriRef<&'a str>) -> Self {
        Self {
            iri: iri.iri.into(),
            positions: iri.positions,
        }
    }
}

impl<'a> From<IriRef<String>> for IriRef<Cow<'a, str>> {
    #[inline]
    fn from(iri: IriRef<String>) -> Self {
        Self {
            iri: iri.iri.into(),
            positions: iri.positions,
        }
    }
}

impl<'a> From<&'a IriRef<String>> for IriRef<&'a str> {
    #[inline]
    fn from(iri: &'a IriRef<String>) -> Self {
        Self {
            iri: &iri.iri,
            positions: iri.positions,
        }
    }
}

impl<'a> From<&'a IriRef<Cow<'a, str>>> for IriRef<&'a str> {
    #[inline]
    fn from(iri: &'a IriRef<Cow<'a, str>>) -> Self {
        Self {
            iri: &iri.iri,
            positions: iri.positions,
        }
    }
}

#[cfg(feature = "serde")]
impl<T: Serialize> Serialize for IriRef<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.iri.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: Deref<Target = str> + Deserialize<'de>> Deserialize<'de> for IriRef<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;

        Self::parse(T::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}

/// A [RFC 3987](https://www.ietf.org/rfc/rfc3987.html) IRI.
///
/// Instances of this type are guaranteed to be absolute,
/// unlike [`IriRef`].
///
/// ```
/// use std::convert::TryFrom;
/// use oxiri::{Iri, IriRef};
///
/// // Parse and validate base IRI
/// let base_iri = Iri::parse("http://foo.com/bar/baz")?;
///
/// // Validate and resolve relative IRI
/// let iri = base_iri.resolve("bat#foo")?;
/// assert_eq!(iri.into_inner(), "http://foo.com/bar/bat#foo");
///
/// // Iri::parse will err on relative IRIs.
/// assert!(Iri::parse("../bar/baz").is_err());
///
/// // It is possible to build an Iri from an IriRef object
/// Iri::try_from(IriRef::parse("http://foo.com/bar")?)?;
/// # Result::<(), oxiri::IriParseError>::Ok(())
/// ```
#[derive(Clone, Copy)]
pub struct Iri<T>(IriRef<T>);

impl<T: Deref<Target = str>> Iri<T> {
    /// Parses and validates the IRI following the grammar from [RFC 3987](https://www.ietf.org/rfc/rfc3987.html).
    ///
    /// This operation keeps internally the `iri` parameter and does not allocate.
    ///
    /// Use [`parse_unchecked`](Self::parse_unchecked) if you already know the IRI is valid to get faster processing.
    ///
    /// ```
    /// use oxiri::Iri;
    ///
    /// Iri::parse("http://foo.com/bar/baz")?;
    /// # Result::<(), oxiri::IriParseError>::Ok(())
    /// ```
    pub fn parse(iri: T) -> Result<Self, IriParseError> {
        IriRef::parse(iri)?.try_into()
    }

    /// Variant of [`parse`](Self::parse) that assumes that the IRI is valid to skip validation.
    ///
    /// ```
    /// use oxiri::Iri;
    ///
    /// Iri::parse_unchecked("http://foo.com/bar/baz");
    /// ```
    pub fn parse_unchecked(iri: T) -> Self {
        Iri(IriRef::parse_unchecked(iri))
    }

    /// Validates and resolved a relative IRI against the current IRI
    /// following [RFC 3986](https://www.ietf.org/rfc/rfc3986.html) relative URI resolution algorithm.
    ///
    /// Use [`resolve_unchecked`](Self::resolve_unchecked) if you already know the IRI is valid to get faster processing.
    ///
    /// ```
    /// use oxiri::Iri;
    ///
    /// let base_iri = Iri::parse("http://foo.com/bar/baz")?;
    /// let iri = base_iri.resolve("bat#foo")?;
    /// assert_eq!(iri.into_inner(), "http://foo.com/bar/bat#foo");
    /// # Result::<(), oxiri::IriParseError>::Ok(())
    /// ```
    pub fn resolve(&self, iri: &str) -> Result<Iri<String>, IriParseError> {
        Ok(Iri(self.0.resolve(iri)?))
    }

    /// Variant of [`resolve`](Self::resolve) that assumes that the IRI is valid to skip validation.
    ///
    /// ```
    /// use oxiri::Iri;
    ///
    /// let base_iri = Iri::parse_unchecked("http://foo.com/bar/baz");
    /// let iri = base_iri.resolve_unchecked("bat#foo");
    /// assert_eq!(iri.into_inner(), "http://foo.com/bar/bat#foo");
    /// ```
    pub fn resolve_unchecked(&self, iri: &str) -> Iri<String> {
        Iri(self.0.resolve_unchecked(iri))
    }

    /// Validates and resolved a relative IRI against the current IRI
    /// following [RFC 3986](https://www.ietf.org/rfc/rfc3986.html) relative URI resolution algorithm.
    ///
    /// It outputs the resolved IRI into `target_buffer` to avoid any memory allocation.
    ///
    /// Use [`resolve_into_unchecked`](Self::resolve_into_unchecked) if you already know the IRI is valid to get faster processing.
    ///
    /// ```
    /// use oxiri::Iri;
    ///
    /// let base_iri = Iri::parse("http://foo.com/bar/baz")?;
    /// let mut result = String::default();
    /// let iri = base_iri.resolve_into("bat#foo", &mut result)?;
    /// assert_eq!(result, "http://foo.com/bar/bat#foo");
    /// # Result::<(), oxiri::IriParseError>::Ok(())
    /// ```
    pub fn resolve_into(&self, iri: &str, target_buffer: &mut String) -> Result<(), IriParseError> {
        self.0.resolve_into(iri, target_buffer)
    }

    /// Variant of [`resolve_into`](Self::resolve_into) that assumes that the IRI is valid to skip validation.
    ///
    /// ```
    /// use oxiri::Iri;
    ///
    /// let base_iri = Iri::parse_unchecked("http://foo.com/bar/baz");
    /// let mut result = String::default();
    /// let iri = base_iri.resolve_into_unchecked("bat#foo", &mut result);
    /// assert_eq!(result, "http://foo.com/bar/bat#foo");
    /// ```
    pub fn resolve_into_unchecked(&self, iri: &str, target_buffer: &mut String) {
        self.0.resolve_into_unchecked(iri, target_buffer)
    }

    /// Returns an IRI that, when resolved against the current IRI returns `abs`.
    ///
    /// This function returns an error
    /// if is not possible to build a relative IRI that can resolve to the same IRI.
    /// For example, when the path contains `/../`.
    ///
    /// Note that the output of this function might change in minor releases.
    ///
    /// ```
    /// use oxiri::Iri;
    ///
    /// let base_iri = Iri::parse("http://foo.com/bar/baz")?;
    /// let iri = Iri::parse("http://foo.com/bar/bat#foo")?;
    /// let relative_iri = base_iri.relativize(&iri)?;
    /// assert_eq!(relative_iri, "bat#foo");
    /// # Result::<(), Box<dyn std::error::Error>>::Ok(())
    /// ```
    pub fn relativize<T2: Deref<Target = str>>(
        &self,
        abs: &Iri<T2>,
    ) -> Result<IriRef<String>, IriRelativizeError> {
        let base = self;
        let abs_authority = abs.authority();
        let base_authority = base.authority();
        let abs_path = abs.path();
        let base_path = base.path();
        let abs_query = abs.query();
        let base_query = base.query();

        // We validate the path, resolving algorithm eats /. and /.. in hierarchical path
        for segment in abs_path.split('/').skip(1) {
            if matches!(segment, "." | "..") {
                return Err(IriRelativizeError {});
            }
        }

        if abs.scheme() != base.scheme()
            || abs_authority.is_none() && base_authority.is_some()
            || abs_path
                // Might confuse with a scheme
                .split_once(':')
                .map_or(false, |(candidate_scheme, _)| {
                    !candidate_scheme.contains('/')
                })
        {
            return Ok(IriRef {
                iri: abs.0.to_string(),
                positions: abs.0.positions,
            });
        }
        if abs_authority != base_authority
            // the resolution algorithm does not handle empty paths:
            || abs_path.is_empty() && (!base_path.is_empty() || base_query.is_some())
            // confusion with authority:
            || abs_path.starts_with("//")
        {
            return Ok(IriRef {
                iri: abs.0[abs.0.positions.scheme_end..].to_string(),
                positions: IriElementsPositions {
                    scheme_end: 0,
                    authority_end: abs.0.positions.authority_end - abs.0.positions.scheme_end,
                    path_end: abs.0.positions.path_end - abs.0.positions.scheme_end,
                    query_end: abs.0.positions.query_end - abs.0.positions.scheme_end,
                },
            });
        }
        if abs_path != base_path || abs_query.is_none() && base_query.is_some() {
            let number_of_shared_characters = abs_path
                .bytes()
                .zip(base_path.bytes())
                .take_while(|(l, r)| l == r)
                .count();
            // We decrease until finding a /
            let number_of_shared_characters = abs_path[..number_of_shared_characters]
                .rfind('/')
                .map_or(0, |n| n + 1);
            return if abs_path[number_of_shared_characters..].contains('/')
                || base_path[number_of_shared_characters..].contains('/')
                || abs_path[number_of_shared_characters..].is_empty()
                || abs_path[number_of_shared_characters..].contains(':')
            {
                // We output the full path because we have a / or an empty end
                Ok(IriRef {
                    iri: abs.0[abs.0.positions.authority_end..].to_string(),
                    positions: IriElementsPositions {
                        scheme_end: 0,
                        authority_end: 0,
                        path_end: abs.0.positions.path_end - abs.0.positions.authority_end,
                        query_end: abs.0.positions.query_end - abs.0.positions.authority_end,
                    },
                })
            } else {
                // We just override the last element
                Ok(IriRef {
                    iri: abs.0[abs.0.positions.authority_end + number_of_shared_characters..]
                        .to_string(),
                    positions: IriElementsPositions {
                        scheme_end: 0,
                        authority_end: 0,
                        path_end: abs.0.positions.path_end
                            - abs.0.positions.authority_end
                            - number_of_shared_characters,
                        query_end: abs.0.positions.query_end
                            - abs.0.positions.authority_end
                            - number_of_shared_characters,
                    },
                })
            };
        }
        if abs_query != base_query {
            return Ok(IriRef {
                iri: abs.0[abs.0.positions.path_end..].to_string(),
                positions: IriElementsPositions {
                    scheme_end: 0,
                    authority_end: 0,
                    path_end: 0,
                    query_end: abs.0.positions.query_end - abs.0.positions.path_end,
                },
            });
        }
        Ok(IriRef {
            iri: abs.0[abs.0.positions.query_end..].to_string(),
            positions: IriElementsPositions {
                scheme_end: 0,
                authority_end: 0,
                path_end: 0,
                query_end: 0,
            },
        })
    }

    /// Returns an IRI borrowing this IRI's text
    #[inline]
    pub fn as_ref(&self) -> Iri<&str> {
        Iri(self.0.as_ref())
    }

    /// Returns the underlying IRI representation.
    ///
    /// ```
    /// use oxiri::Iri;
    ///
    /// let iri = Iri::parse("http://example.com/foo")?;
    /// assert_eq!(iri.as_str(), "http://example.com/foo");
    /// # Result::<(), oxiri::IriParseError>::Ok(())
    /// ```
    #[inline]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Returns the underlying IRI representation.
    ///
    /// ```
    /// use oxiri::Iri;
    ///
    /// let iri = Iri::parse("http://example.com/foo")?;
    /// assert_eq!(iri.into_inner(), "http://example.com/foo");
    /// # Result::<(), oxiri::IriParseError>::Ok(())
    /// ```
    #[inline]
    pub fn into_inner(self) -> T {
        self.0.into_inner()
    }

    /// Returns the IRI scheme.
    ///
    /// Beware: the scheme case is not normalized. Use case insensitive comparisons if you look for a specific scheme.
    ///
    /// ```
    /// use oxiri::Iri;
    ///
    /// let iri = Iri::parse("hTTp://example.com")?;
    /// assert_eq!(iri.scheme(), "hTTp");
    /// # Result::<(), oxiri::IriParseError>::Ok(())
    /// ```
    #[inline]
    pub fn scheme(&self) -> &str {
        self.0.scheme().expect("The IRI should be absolute")
    }

    /// Returns the IRI authority if it exists.
    ///
    /// Beware: the host case is not normalized. Use case insensitive comparisons if you look for a specific host.
    ///
    /// ```
    /// use oxiri::Iri;
    ///
    /// let http = Iri::parse("http://foo:pass@example.com:80/my/path")?;
    /// assert_eq!(http.authority(), Some("foo:pass@example.com:80"));
    ///
    /// let mailto = Iri::parse("mailto:foo@bar.com")?;
    /// assert_eq!(mailto.authority(), None);
    /// # Result::<(), oxiri::IriParseError>::Ok(())
    /// ```
    #[inline]
    pub fn authority(&self) -> Option<&str> {
        self.0.authority()
    }

    /// Returns the IRI path.
    ///
    /// ```
    /// use oxiri::Iri;
    ///
    /// let http = Iri::parse("http://foo:pass@example.com:80/my/path?foo=bar")?;
    /// assert_eq!(http.path(), "/my/path");
    ///
    /// let mailto = Iri::parse("mailto:foo@bar.com")?;
    /// assert_eq!(mailto.path(), "foo@bar.com");
    /// # Result::<(), oxiri::IriParseError>::Ok(())
    /// ```
    #[inline]
    pub fn path(&self) -> &str {
        self.0.path()
    }

    /// Returns the IRI query if it exists.
    ///
    /// ```
    /// use oxiri::Iri;
    ///
    /// let iri = Iri::parse("http://example.com/my/path?query=foo#frag")?;
    /// assert_eq!(iri.query(), Some("query=foo"));
    /// # Result::<(), oxiri::IriParseError>::Ok(())
    /// ```
    #[inline]
    pub fn query(&self) -> Option<&str> {
        self.0.query()
    }

    /// Returns the IRI fragment if it exists.
    ///
    /// ```
    /// use oxiri::Iri;
    ///
    /// let iri = Iri::parse("http://example.com/my/path?query=foo#frag")?;
    /// assert_eq!(iri.fragment(), Some("frag"));
    /// # Result::<(), oxiri::IriParseError>::Ok(())
    /// ```
    #[inline]
    pub fn fragment(&self) -> Option<&str> {
        self.0.fragment()
    }
}

impl<Lft: PartialEq<Rhs>, Rhs> PartialEq<Iri<Rhs>> for Iri<Lft> {
    #[inline]
    fn eq(&self, other: &Iri<Rhs>) -> bool {
        self.0.eq(&other.0)
    }
}

impl<Lft: PartialEq<Rhs>, Rhs> PartialEq<IriRef<Rhs>> for Iri<Lft> {
    #[inline]
    fn eq(&self, other: &IriRef<Rhs>) -> bool {
        self.0.eq(other)
    }
}

impl<Lft: PartialEq<Rhs>, Rhs> PartialEq<Iri<Rhs>> for IriRef<Lft> {
    #[inline]
    fn eq(&self, other: &Iri<Rhs>) -> bool {
        self.eq(&other.0)
    }
}

impl<T: PartialEq<str>> PartialEq<str> for Iri<T> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.0.eq(other)
    }
}

impl<'a, T: PartialEq<&'a str>> PartialEq<&'a str> for Iri<T> {
    #[inline]
    fn eq(&self, other: &&'a str) -> bool {
        self.0.eq(other)
    }
}

impl<T: PartialEq<String>> PartialEq<String> for Iri<T> {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.0.eq(other)
    }
}

impl<'a, T: PartialEq<Cow<'a, str>>> PartialEq<Cow<'a, str>> for Iri<T> {
    #[inline]
    fn eq(&self, other: &Cow<'a, str>) -> bool {
        self.0.eq(other)
    }
}

impl<T: PartialEq<str>> PartialEq<Iri<T>> for str {
    #[inline]
    fn eq(&self, other: &Iri<T>) -> bool {
        self.eq(&other.0)
    }
}

impl<'a, T: PartialEq<&'a str>> PartialEq<Iri<T>> for &'a str {
    #[inline]
    fn eq(&self, other: &Iri<T>) -> bool {
        self.eq(&other.0)
    }
}

impl<T: PartialEq<String>> PartialEq<Iri<T>> for String {
    #[inline]
    fn eq(&self, other: &Iri<T>) -> bool {
        self.eq(&other.0)
    }
}

impl<'a, T: PartialEq<Cow<'a, str>>> PartialEq<Iri<T>> for Cow<'a, str> {
    #[inline]
    fn eq(&self, other: &Iri<T>) -> bool {
        self.eq(&other.0)
    }
}

impl<T: Eq> Eq for Iri<T> {}

impl<T: Hash> Hash for Iri<T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<Lft: PartialOrd<Rhs>, Rhs> PartialOrd<Iri<Rhs>> for Iri<Lft> {
    #[inline]
    fn partial_cmp(&self, other: &Iri<Rhs>) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<Lft: PartialOrd<Rhs>, Rhs> PartialOrd<IriRef<Rhs>> for Iri<Lft> {
    #[inline]
    fn partial_cmp(&self, other: &IriRef<Rhs>) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl<Lft: PartialOrd<Rhs>, Rhs> PartialOrd<Iri<Rhs>> for IriRef<Lft> {
    #[inline]
    fn partial_cmp(&self, other: &Iri<Rhs>) -> Option<Ordering> {
        self.partial_cmp(&other.0)
    }
}

impl<T: Ord> Ord for Iri<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T: Deref<Target = str>> Deref for Iri<T> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.0.deref()
    }
}

impl<T: AsRef<str>> AsRef<str> for Iri<T> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl<T: Borrow<str>> Borrow<str> for Iri<T> {
    #[inline]
    fn borrow(&self) -> &str {
        self.0.borrow()
    }
}

impl<T: fmt::Debug> fmt::Debug for Iri<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: fmt::Display> fmt::Display for Iri<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for Iri<String> {
    type Err = IriParseError;

    #[inline]
    fn from_str(iri: &str) -> Result<Self, IriParseError> {
        Self::parse(iri.to_owned())
    }
}

impl<'a> From<Iri<&'a str>> for Iri<String> {
    #[inline]
    fn from(iri: Iri<&'a str>) -> Self {
        Self(iri.0.into())
    }
}

impl<'a> From<Iri<Cow<'a, str>>> for Iri<String> {
    #[inline]
    fn from(iri: Iri<Cow<'a, str>>) -> Self {
        Self(iri.0.into())
    }
}

impl From<Iri<Box<str>>> for Iri<String> {
    #[inline]
    fn from(iri: Iri<Box<str>>) -> Self {
        Self(iri.0.into())
    }
}

impl<'a> From<Iri<&'a str>> for Iri<Cow<'a, str>> {
    #[inline]
    fn from(iri: Iri<&'a str>) -> Self {
        Self(iri.0.into())
    }
}

impl<'a> From<Iri<String>> for Iri<Cow<'a, str>> {
    #[inline]
    fn from(iri: Iri<String>) -> Self {
        Self(iri.0.into())
    }
}

impl<'a> From<&'a Iri<String>> for Iri<&'a str> {
    #[inline]
    fn from(iri: &'a Iri<String>) -> Self {
        Self(iri.0.as_ref())
    }
}

impl<'a> From<&'a Iri<Cow<'a, str>>> for Iri<&'a str> {
    #[inline]
    fn from(iri: &'a Iri<Cow<'a, str>>) -> Self {
        Self(iri.0.as_ref())
    }
}

impl<T: Deref<Target = str>> From<Iri<T>> for IriRef<T> {
    fn from(iri: Iri<T>) -> Self {
        iri.0
    }
}

impl<T: Deref<Target = str>> TryFrom<IriRef<T>> for Iri<T> {
    type Error = IriParseError;

    fn try_from(iri: IriRef<T>) -> Result<Self, IriParseError> {
        if iri.is_absolute() {
            Ok(Self(iri))
        } else {
            Err(IriParseError {
                kind: IriParseErrorKind::NoScheme,
            })
        }
    }
}

#[cfg(feature = "serde")]
impl<T: Serialize> Serialize for Iri<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: Deref<Target = str> + Deserialize<'de>> Deserialize<'de> for Iri<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        IriRef::deserialize(deserializer)?
            .try_into()
            .map_err(D::Error::custom)
    }
}

/// An error raised during [`Iri`] or [`IriRef`] validation.
#[derive(Debug)]
pub struct IriParseError {
    kind: IriParseErrorKind,
}

impl fmt::Display for IriParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            IriParseErrorKind::NoScheme => write!(f, "No scheme found in an absolute IRI"),
            IriParseErrorKind::InvalidHostCharacter(c) => {
                write!(f, "Invalid character '{c}' in host")
            }
            IriParseErrorKind::InvalidHostIp(e) => write!(f, "Invalid host IP ({e})"),
            IriParseErrorKind::InvalidPortCharacter(c) => write!(f, "Invalid character '{c}'"),
            IriParseErrorKind::InvalidIriCodePoint(c) => {
                write!(f, "Invalid IRI code point '{c}'")
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

/// An error raised when calling [`Iri::relativize`].
///
/// It can happen when it is not possible to build a relative IRI that can resolve to the same IRI.
/// For example, when the path contains `/../`.
#[derive(Debug)]
pub struct IriRelativizeError {}

impl fmt::Display for IriRelativizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "It is not possible to make this IRI relative because it contains `/..` or `/.`"
        )
    }
}

impl Error for IriRelativizeError {}

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

/// parser implementing https://url.spec.whatwg.org/#concept-basic-url-parser without the normalization or backward compatibility bits to comply with RFC 3987
///
/// A sub function takes care of each state
struct IriParser<'a, O: OutputBuffer, const UNCHECKED: bool> {
    iri: &'a str,
    base: Option<IriRef<&'a str>>,
    input: ParserInput<'a>,
    output: &'a mut O,
    output_positions: IriElementsPositions,
    input_scheme_end: usize,
}

impl<'a, O: OutputBuffer, const UNCHECKED: bool> IriParser<'a, O, UNCHECKED> {
    fn parse(
        iri: &'a str,
        base: Option<IriRef<&'a str>>,
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
            Some(':') => {
                if UNCHECKED {
                    self.parse_scheme()
                } else {
                    self.parse_error(IriParseErrorKind::NoScheme)
                }
            }
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
                    self.remove_last_segment();
                    if self.output.len() > base.positions.scheme_end {
                        // We have some path or authority, we keep a base '/'
                        self.output.push('/');
                    }
                    self.parse_relative_path()
                }
            }
        } else {
            self.output_positions.scheme_end = 0;
            self.input_scheme_end = 0;
            if self.input.starts_with('/') {
                self.input.next();
                self.output.push('/');
                self.parse_path_or_authority()
            } else {
                self.output_positions.authority_end = 0;
                self.parse_relative_path()
            }
        }
    }

    fn parse_relative_path(&mut self) -> Result<(), IriParseError> {
        while let Some(c) = self.input.front() {
            if matches!(c, '/' | '?' | '#') {
                break;
            }
            self.input.next();
            self.read_url_codepoint_or_echar(c, |c| is_iunreserved_or_sub_delims(c) || c == '@')?;
        }
        self.parse_path()
    }

    fn parse_relative_slash(&mut self, base: &IriRef<&'a str>) -> Result<(), IriParseError> {
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
                    self.read_url_codepoint_or_echar(c, |c| {
                        is_iunreserved_or_sub_delims(c) || c == ':'
                    })?;
                }
            }
        }
    }

    fn parse_host(&mut self) -> Result<(), IriParseError> {
        if self.input.starts_with('[') {
            // IP v6
            let start_position = self.input.position;
            while let Some(c) = self.input.next() {
                self.output.push(c);
                if c == ']' {
                    let ip = &self.iri[start_position + 1..self.input.position - 1];
                    if !UNCHECKED {
                        if ip.starts_with('v') || ip.starts_with('V') {
                            self.validate_ip_v_future(ip)?;
                        } else if let Err(error) = Ipv6Addr::from_str(ip) {
                            return self.parse_error(IriParseErrorKind::InvalidHostIp(error));
                        }
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
                        Some(c) => {
                            if UNCHECKED {
                                self.output.push(c);
                                continue;
                            } else {
                                self.parse_error(IriParseErrorKind::InvalidHostCharacter(c))
                            }
                        }
                    };
                }
            }
            if UNCHECKED {
                // We consider it's valid even if it's not finished
                self.output_positions.authority_end = self.output.len();
                self.parse_path_start(None)
            } else {
                self.parse_error(IriParseErrorKind::InvalidHostCharacter('['))
            }
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
                    Some(c) => self.read_url_codepoint_or_echar(c, is_iunreserved_or_sub_delims)?,
                }
            }
        }
    }

    fn parse_port(&mut self) -> Result<(), IriParseError> {
        loop {
            let c = self.input.next();
            match c {
                Some('/') | Some('?') | Some('#') | None => {
                    self.output_positions.authority_end = self.output.len();
                    return self.parse_path_start(c);
                }
                Some(c) => {
                    if UNCHECKED || c.is_ascii_digit() {
                        self.output.push(c)
                    } else {
                        return self.parse_error(IriParseErrorKind::InvalidPortCharacter(c));
                    }
                }
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
                self.read_url_codepoint_or_echar(c, |c| {
                    is_iunreserved_or_sub_delims(c) || matches!(c, ':' | '@')
                })?;
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
                    } else if c.is_none() {
                        self.output_positions.path_end = self.output.len();
                        self.output_positions.query_end = self.output.len();
                        return Ok(());
                    }
                }
                Some(c) => self.read_url_codepoint_or_echar(c, |c| {
                    is_iunreserved_or_sub_delims(c) || matches!(c, ':' | '@')
                })?,
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
                self.read_url_codepoint_or_echar(c, |c| {
                    is_iunreserved_or_sub_delims(c) || matches!(c, ':' | '@' | '/' | '?' | '\u{E000}'..='\u{F8FF}' | '\u{F0000}'..='\u{FFFFD}' | '\u{100000}'..='\u{10FFFD}')
                })?
            }
        }
        self.output_positions.query_end = self.output.len();
        Ok(())
    }

    fn parse_fragment(&mut self) -> Result<(), IriParseError> {
        while let Some(c) = self.input.next() {
            self.read_url_codepoint_or_echar(c, |c| {
                is_iunreserved_or_sub_delims(c) || matches!(c, ':' | '@' | '/' | '?')
            })?;
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

    fn read_url_codepoint_or_echar(
        &mut self,
        c: char,
        valid: impl Fn(char) -> bool,
    ) -> Result<(), IriParseError> {
        if UNCHECKED || valid(c) {
            self.output.push(c);
            Ok(())
        } else if c == '%' {
            self.read_echar()
        } else {
            self.parse_error(IriParseErrorKind::InvalidIriCodePoint(c))
        }
    }

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

    fn parse_error<T>(&self, kind: IriParseErrorKind) -> Result<T, IriParseError> {
        Err(IriParseError { kind })
    }

    // IPvFuture      = "v" 1*HEXDIG "." 1*( unreserved / sub-delims / ":" )
    fn validate_ip_v_future(&self, ip: &str) -> Result<(), IriParseError> {
        let mut chars = ip.chars();

        let c = chars.next().ok_or(IriParseError {
            kind: IriParseErrorKind::InvalidHostCharacter(']'),
        })?;
        if !matches!(c, 'v' | 'V') {
            return self.parse_error(IriParseErrorKind::InvalidHostCharacter(c));
        };

        let mut with_a_version = false;
        for c in &mut chars {
            if c == '.' {
                break;
            } else if c.is_ascii_hexdigit() {
                with_a_version = true;
            } else {
                return self.parse_error(IriParseErrorKind::InvalidHostCharacter(c));
            }
        }
        if !with_a_version {
            return self.parse_error(IriParseErrorKind::InvalidHostCharacter(
                chars.next().unwrap_or(']'),
            ));
        }

        if chars.as_str().is_empty() {
            return self.parse_error(IriParseErrorKind::InvalidHostCharacter(']'));
        };
        for c in chars {
            if !is_unreserved_or_sub_delims(c) && c != ':' {
                return self.parse_error(IriParseErrorKind::InvalidHostCharacter(c));
            }
        }

        Ok(())
    }
}

fn is_iunreserved_or_sub_delims(c: char) -> bool {
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
        | ';'
        | '='
        | '_'
        | '~'
        | '\u{A0}'..='\u{D7FF}'
        | '\u{F900}'..='\u{FDCF}'
        | '\u{FDF0}'..='\u{FFEF}'
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

fn is_unreserved_or_sub_delims(c: char) -> bool {
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
        | ';'
        | '='
        | '_'
        | '~'
    )
}
