use headers::{Header, HeaderName, HeaderValue};
use httpdate::HttpDate;

#[derive(Clone, Debug, PartialEq)]
struct Expires(HttpDate);

impl Header for Expires {
    fn name() -> &'static HeaderName {
        &http::header::EXPIRES
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        I: Iterator<Item = &'i HeaderValue>,
    {
        let h = values.next().ok_or(headers::Error::invalid())?;
        let ret = Expires(
            h.to_str()
                .map_err(|_| headers::Error::invalid())?
                .parse()
                .map_err(|_| headers::Error::invalid())?,
        );
        if values.next().is_none() {
            Ok(ret)
        } else {
            Err(headers::Error::invalid())
        }
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
                h.append("Expires", HeaderValue::from_static(val));
            }
            h.typed_get::<Expires>()
        };

        assert_eq!(decode(["Sun, 06 Nov 1994 08:49:37 GMT"].iter()), Some(Expires("Sun Nov  6 08:49:37 1994".parse().unwrap())));
        assert_eq!(decode(["Sunday, 06-Nov-94 08:49:37 GMT"].iter()), Some(Expires("Sun Nov  6 08:49:37 1994".parse().unwrap())));
        assert_eq!(decode(["Sun Nov  6 08:49:37 1994"].iter()), Some(Expires("Sun Nov  6 08:49:37 1994".parse().unwrap())));
        assert_eq!(decode(["Friday, 06-Nov-70 08:49:37 GMT"].iter()), Some(Expires("Fri Nov  6 08:49:37 1970".parse().unwrap())));
        assert_eq!(decode(["Wednesday, 06-Nov-69 08:49:37 GMT"].iter()), Some(Expires("Wed Nov  6 08:49:37 2069".parse().unwrap())));
        assert_eq!(decode(["Sun, 06 Nov 1994 08:49:37"].iter()), None);
        assert_eq!(decode(["Sun 06 Nov 1994 08:49:37 GMT"].iter()), None);
        assert_eq!(decode(["Win, 06 Nov 1994 08:49:37 GMT"].iter()), None);
        assert_eq!(decode(["Sun, 06 Now 1994 08:49:37 GMT"].iter()), None);
        assert_eq!(decode(["Mon, 06 Nov 1994 08:49:37 GMT"].iter()), None);
        assert_eq!(decode(["Sun, 31 Nov 1994 08:49:37 GMT"].iter()), None);
        assert_eq!(decode(["Sun, 06 Nov 1994 25:49:37 GMT"].iter()), None);
        assert_eq!(decode(["Sun, 06 Nov 1994 08:60:37 GMT"].iter()), None);
        assert_eq!(decode(["Sun, 06 Nov 1994 08:49:60 GMT"].iter()), None);
    }

    #[test]
    fn test_encode() {
        let encode = |val| {
            let mut h = HeaderMap::new();
            h.typed_insert(val);
            h["expires"].clone()
        };

        assert_eq!(encode(Expires("Sun Nov  6 08:49:37 1994".parse().unwrap())), "Sun, 06 Nov 1994 08:49:37 GMT");
    }
}
