//! Validated ticker symbol type.
//!
//! [`Symbol`] is a "parse, don't validate" newtype: the only way to build
//! one is [`Symbol::parse`], so any function that accepts a `&Symbol` can
//! rely on the input already being normalized and safe to use.

/// A normalized, validated ticker symbol such as `AAPL` or `BRK.B`.
///
/// The inner string is private, so a `Symbol` can only come from
/// [`Symbol::parse`]. That gives every holder three guarantees:
///
/// - It is uppercase, so `aapl` and `AAPL` are the same value and share one
///   cache entry.
/// - It contains only ASCII letters, digits, `.`, and `-`, so it is safe to
///   put in a URL query or a cache key without further escaping.
/// - It is between 1 and 12 characters long.
///
/// It implements `Hash` and `Eq` so it can be used directly as a cache key.
///
/// # What it does not guarantee
///
/// `Symbol` checks the *shape* of the input, not whether the ticker exists.
/// `ZZZZ` and `---` both parse successfully. Whether FMP knows the symbol is
/// decided upstream and surfaces as [`AppError::NotFound`].
///
/// [`AppError::NotFound`]: crate::error::AppError::NotFound
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol(String);

impl Symbol {
    /// Parses raw user input into a [`Symbol`], or returns `None` if it
    /// isn't a plausible ticker.
    ///
    /// Processing happens in this order:
    ///
    /// 1. Leading and trailing whitespace is trimmed.
    /// 2. ASCII letters are uppercased.
    /// 3. The result must be non-empty, at most 12 characters, and made up
    ///    only of ASCII letters, digits, `.`, and `-`.
    ///
    /// Trimming means `" aapl "` is accepted as `AAPL`, but whitespace
    /// *inside* the input (`"A APL"`) is rejected. Non-ASCII characters are
    /// rejected too, even ones that look like letters.
    ///
    /// Returns `None` rather than an error type because there is only one
    /// way to fail. Callers map it to
    /// [`AppError::BadSymbol`](crate::error::AppError::BadSymbol).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// assert_eq!(Symbol::parse("aapl").unwrap().as_str(), "AAPL");
    /// assert_eq!(Symbol::parse("  brk.b ").unwrap().as_str(), "BRK.B");
    /// assert_eq!(Symbol::parse("RDS-A").unwrap().as_str(), "RDS-A");
    ///
    /// assert!(Symbol::parse("").is_none());               // empty
    /// assert!(Symbol::parse("A APL").is_none());          // inner space
    /// assert!(Symbol::parse("AAPL;DROP").is_none());      // bad character
    /// assert!(Symbol::parse("ABCDEFGHIJKLM").is_none());  // 13 chars
    /// ```
    pub fn parse(raw: &str) -> Option<Self> {
        let s = raw.trim().to_ascii_uppercase();
        let ok = !s.is_empty()
            && s.len() <= 12
            && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-');
        ok.then_some(Self(s))
    }

    /// Returns the normalized symbol as a string slice.
    ///
    /// The value is always uppercase and already validated, so it can be
    /// passed straight to the FMP client.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}