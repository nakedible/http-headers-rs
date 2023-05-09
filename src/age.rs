//! `Age` header
//!
//! ### RFC9111 5.1. Age
//!
//! The "Age" response header field conveys the sender's estimate of the time
//! since the response was generated or successfully validated at the origin
//! server. Age values are calculated as specified in Section 4.2.3.
//!
//! ```text
//! Age = delta-seconds
//! ```
//!
//! The Age field value is a non-negative integer, representing time in seconds
//! (see Section 1.2.2).
//!
//! Although it is defined as a singleton header field, a cache encountering a
//! message with a list-based Age field value SHOULD use the first member of the
//! field value, discarding subsequent ones.
//!
//! If the field value (after discarding additional members, as per above) is
//! invalid (e.g., it contains something other than a non-negative integer), a
//! cache SHOULD ignore the field.
//!
//! The presence of an Age header field implies that the response was not
//! generated or validated by the origin server for this request. However, lack
//! of an Age header field does not imply the origin was contacted.
//!
//! ### RFC9111 1.2.2. Delta Seconds
//!
//! The delta-seconds rule specifies a non-negative integer, representing time
//! in seconds.
//!
//! ```text
//! delta-seconds  = 1*DIGIT
//! ```
//!
//! A recipient parsing a delta-seconds value and converting it to binary form
//! ought to use an arithmetic type of at least 31 bits of non-negative integer
//! range. If a cache receives a delta-seconds value greater than the greatest
//! integer it can represent, or if any of its subsequent calculations
//! overflows, the cache MUST consider the value to be 2147483648 (231) or the
//! greatest positive integer it can conveniently represent.
//!
//! Note: The value 2147483648 is here for historical reasons, represents
//! infinity (over 68 years), and does not need to be stored in binary form; an
//! implementation could produce it as a string if any overflow occurs, even if
//! the calculations are performed with an arithmetic type incapable of directly
//! representing that number. What matters here is that an overflow be detected
//! and not treated as a negative value in later calculations.

use headers::{Header, HeaderName, HeaderValue};
use logos::Logos;

use crate::util::parse_u64;

#[derive(Clone, Debug, PartialEq)]
struct Age(u64);

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t]+")]
enum AgeToken {
    #[regex(b"[0-9]+")]
    Int,
    #[token(b",")]
    Comma,
}

fn process_header(i: &[u8]) -> Result<Option<Age>, ()> {
    let mut l = AgeToken::lexer(i);
    let mut ret = None;
    while let Some(t) = l.next() {
        match t? {
            AgeToken::Comma if ret.is_some() => return Ok(ret),
            AgeToken::Comma => {}
            AgeToken::Int if ret.is_none() => {
                ret = Some(Age(parse_u64::<true>(l.slice())?));
            }
            AgeToken::Int => return Err(()),
        };
    }
    Ok(ret)
}

impl Header for Age {
    fn name() -> &'static HeaderName {
        &http::header::AGE
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        I: Iterator<Item = &'i HeaderValue>,
    {
        while let Some(h) = values.next() {
            if let Some(ret) =
                process_header(h.as_bytes()).map_err(|_| headers::Error::invalid())?
            {
                return Ok(ret);
            }
        }
        Err(headers::Error::invalid())
    }

    fn encode<E>(&self, values: &mut E)
    where
        E: Extend<HeaderValue>,
    {
        let value = HeaderValue::from_str(&self.0.to_string()).unwrap();
        values.extend(std::iter::once(value));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use headers::{HeaderMap, HeaderMapExt};

    #[test]
    fn test_decode() {
        let decode = |vals| {
            let mut h = HeaderMap::new();
            for &val in vals {
                h.append("Age", HeaderValue::from_static(val));
            }
            h.typed_get::<Age>()
        };

        assert_eq!(decode(["0"].iter()), Some(Age(0)));
        assert_eq!(decode(["123"].iter()), Some(Age(123)));
        assert_eq!(decode(["000000000000000123"].iter()), Some(Age(123)));
        assert_eq!(
            decode(["123123123123123123"].iter()),
            Some(Age(123123123123123123))
        );
        assert_eq!(decode(["000000000000000000123"].iter()), Some(Age(123)));
        assert_eq!(
            decode(["123123123123123123123"].iter()),
            Some(Age(u64::MAX))
        );
        assert_eq!(decode([", ,,,   ,  123, ,    ,"].iter()), Some(Age(123)));
        assert_eq!(decode(["123, 234"].iter()), Some(Age(123)));
        assert_eq!(decode(["123", ""].iter()), Some(Age(123)));
        assert_eq!(decode(["123", ",, ,    ,"].iter()), Some(Age(123)));
        assert_eq!(
            decode([", ,,,   ,  123, ,    ,", ",, ,,   ,"].iter()),
            Some(Age(123))
        );
        assert_eq!(decode(["123, a"].iter()), Some(Age(123)));
        assert_eq!(decode([", ,,,   ,  123, ,a   ,"].iter()), Some(Age(123)));
        assert_eq!(
            decode([", ,,,   ,  123, ,   ,", "a"].iter()),
            Some(Age(123))
        );
        assert_eq!(
            decode([", ,,,   ,  123, ,   ,", ",, a, ,,   ,"].iter()),
            Some(Age(123))
        );
        assert_eq!(decode(["-0"].iter()), None);
        assert_eq!(decode(["-123"].iter()), None);
        assert_eq!(decode(["-000000000000000123"].iter()), None);
        assert_eq!(decode(["-123123123123123123"].iter()), None);
        assert_eq!(decode(["-000000000000000000123"].iter()), None);
        assert_eq!(decode(["-123123123123123123123"].iter()), None);
        assert_eq!(decode(["123 234"].iter()), None);
        assert_eq!(decode(["a"].iter()), None);
        assert_eq!(decode(["a, 123"].iter()), None);
        assert_eq!(decode([",, a, 123,"].iter()), None);
        assert_eq!(decode(["a", "123"].iter()), None);
    }

    #[test]
    fn test_encode() {
        let encode = |val| {
            let mut h = HeaderMap::new();
            h.typed_insert(val);
            h["age"].clone()
        };

        assert_eq!(encode(Age(0)), "0");
        assert_eq!(encode(Age(123)), "123");
        assert_eq!(encode(Age(123123123123123123)), "123123123123123123");
    }
}
