use logos::Logos;
use headers::{Header, HeaderName, HeaderValue};

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
            AgeToken::Comma => {},
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
            println!("{:?}", h);
            if let Some(ret) = process_header(h.as_bytes()).map_err(|_| headers::Error::invalid())? {
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
        let decode = |val| {
            let h = HeaderMap::from_iter([(HeaderName::from_static("age"), HeaderValue::from_static(val))]);
            h.typed_get::<Age>()
        };
        
        assert_eq!(decode("0"), Some(Age(0)));
        assert_eq!(decode("123"), Some(Age(123)));
        assert_eq!(decode("000000000000000123"), Some(Age(123)));
        assert_eq!(decode("123123123123123123"), Some(Age(123123123123123123)));
        assert_eq!(decode("000000000000000000123"), Some(Age(123)));
        assert_eq!(decode("123123123123123123123"), Some(Age(u64::MAX)));
        assert_eq!(decode(", ,,,   ,  123, ,    ,"), Some(Age(123)));
        assert_eq!(decode("123, 234"), Some(Age(123)));
        assert_eq!(decode("123, a"), Some(Age(123)));
        assert_eq!(decode(", ,,,   ,  123, ,a   ,"), Some(Age(123)));
        assert_eq!(decode("-0"), None);
        assert_eq!(decode("-123"), None);
        assert_eq!(decode("-000000000000000123"), None);
        assert_eq!(decode("-123123123123123123"), None);
        assert_eq!(decode("-000000000000000000123"), None);
        assert_eq!(decode("-123123123123123123123"), None);
        assert_eq!(decode("a"), None);
        assert_eq!(decode("a, 123"), None);
        assert_eq!(decode(",, a, 123,"), None);
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
