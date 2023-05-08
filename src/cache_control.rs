use bytes::{BufMut, BytesMut};
use headers::{Header, HeaderName, HeaderValue};
use logos::Logos;

use crate::util::parse_u64;

#[derive(Clone, Default, Debug)]
pub struct CacheControl {
    max_age: Option<u64>,
    max_stale: Option<u64>,
    min_fresh: Option<u64>,
    no_cache: Option<Vec<HeaderName>>,
    no_store: bool,
    no_transform: bool,
    only_if_cached: bool,
    must_revalidate: bool,
    must_understand: bool,
    transform: bool,
    private: Option<Vec<HeaderName>>,
    proxy_revalidate: bool,
    public: bool,
    s_maxage: Option<u64>,
    immutable: bool,
    stale_while_revalidate: Option<u64>,
    stale_if_error: Option<u64>,
    other: Vec<(String, Option<Vec<u8>>)>,
}

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t]+")]
enum CCToken {
    #[token(b"max-age")]
    MaxAge,
    #[token(b"max-stale")]
    MaxStale,
    #[token(b"min-fresh")]
    MinFresh,
    #[token(b"no-cache")]
    NoCache,
    #[token(b"no-store")]
    NoStore,
    #[token(b"no-transform")]
    NoTransform,
    #[token(b"only-if-cached")]
    OnlyIfCached,
    #[token(b"must-revalidate")]
    MustRevalidate,
    #[token(b"must-understand")]
    MustUnderstand,
    #[token(b"transform")]
    Transform,
    #[token(b"private")]
    Private,
    #[token(b"proxy-revalidate")]
    ProxyRevalidate,
    #[token(b"public")]
    Public,
    #[token(b"s-maxage")]
    SMaxage,
    #[token(b"immutable")]
    Immutable,
    #[token(b"stale-while-revalidate")]
    StaleWhileRevalidate,
    #[token(b"stale-if-error")]
    StaleIfError,
    #[regex(b"[!#$%&'*+.^_`|~0-9a-zA-Z-]+", |l| Some(std::str::from_utf8(l.slice()).unwrap().to_owned()))]
    Other(String),
    #[token(b",")]
    Comma,
    #[token(b"=")]
    Equals,
}

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t]+")]
enum CCArg {
    #[regex(b"[!#$%&'*+.^_`|~0-9a-zA-Z-]+")]
    Token,
    #[regex(br#""([^"\\]*)""#, priority = 2)]
    Quoted,
    #[regex(br#""([^"\\]|\\.)*""#, priority = 1)]
    QuotedEscape,
}

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t]+")]
enum TokenList {
    #[regex(b"[!#$%&'*+.^_`|~0-9a-zA-Z-]+")]
    Token,
    #[token(b",")]
    Comma,
}

#[derive(Logos, Debug, PartialEq)]
enum Integer {
    #[regex(b"[0-9]+")]
    Int,
}

fn header_list(i: &[u8]) -> Result<Vec<HeaderName>, ()> {
    let mut ret = Vec::default();
    let mut l = TokenList::lexer(i);
    let mut comma = true;
    while let Some(t) = l.next() {
        match t? {
            TokenList::Comma => comma = true,
            TokenList::Token if comma => {
                ret.push(HeaderName::from_bytes(l.slice()).unwrap());
                comma = false;
            }
            TokenList::Token => return Err(()),
        };
    }
    Ok(ret)
}

fn process_directive(
    cc: &mut CacheControl,
    directive: CCToken,
    argument: Option<&[u8]>,
    raw_argument: Option<&[u8]>,
) -> Result<(), ()> {
    match (directive, argument) {
        (CCToken::MaxAge, Some(v)) => cc.max_age = Some(parse_u64::<true>(v)?),
        (CCToken::MaxStale, Some(v)) => cc.max_stale = Some(parse_u64::<true>(v)?),
        (CCToken::MinFresh, Some(v)) => cc.min_fresh = Some(parse_u64::<true>(v)?),
        (CCToken::NoCache, Some(v)) => cc.no_cache = Some(header_list(v)?),
        (CCToken::NoCache, None) => cc.no_cache = None,
        (CCToken::NoStore, None) => cc.no_store = true,
        (CCToken::NoTransform, None) => cc.no_transform = true,
        (CCToken::OnlyIfCached, None) => cc.only_if_cached = true,
        (CCToken::MustRevalidate, None) => cc.must_revalidate = true,
        (CCToken::MustUnderstand, None) => cc.must_understand = true,
        (CCToken::Transform, None) => cc.transform = true,
        (CCToken::Private, Some(v)) => cc.private = Some(header_list(v)?),
        (CCToken::Private, None) => cc.private = None,
        (CCToken::ProxyRevalidate, None) => cc.proxy_revalidate = true,
        (CCToken::Public, None) => cc.public = true,
        (CCToken::SMaxage, Some(v)) => cc.s_maxage = Some(parse_u64::<true>(v)?),
        (CCToken::Immutable, None) => cc.immutable = true,
        (CCToken::StaleWhileRevalidate, Some(v)) => {
            cc.stale_while_revalidate = Some(parse_u64::<true>(v)?)
        }
        (CCToken::StaleIfError, Some(v)) => cc.stale_if_error = Some(parse_u64::<true>(v)?),
        (CCToken::Other(s), Some(_)) => cc.other.push((s, Some(raw_argument.unwrap().to_vec()))),
        (CCToken::Other(s), None) => cc.other.push((s, None)),
        _ => return Err(()),
    };
    Ok(())
}

fn process_argument<'source>(
    cc: &mut CacheControl,
    directive: CCToken,
    l: logos::Lexer<'source, CCToken>,
) -> Result<logos::Lexer<'source, CCToken>, ()> {
    let mut subl = l.morph::<CCArg>();
    match subl.next().ok_or(())?? {
        CCArg::Token => {
            process_directive(cc, directive, Some(subl.slice()), Some(subl.slice())).ok();
        }
        CCArg::Quoted => {
            let slice = subl.slice();
            process_directive(
                cc,
                directive,
                Some(&slice[1..slice.len() - 1]),
                Some(subl.slice()),
            )
            .ok();
        }
        CCArg::QuotedEscape => {
            let slice = subl.slice();
            let mut vec = slice[1..slice.len() - 1].to_vec();
            vec.retain(|&c| c != b'\\');
            process_directive(cc, directive, Some(&vec[..]), Some(subl.slice())).ok();
        }
    }
    Ok(subl.morph::<CCToken>())
}

fn process_header(cc: &mut CacheControl, i: &[u8]) -> Result<(), ()> {
    let mut l = CCToken::lexer(i);
    let mut cur = None;
    while let Some(t) = l.next() {
        match t? {
            CCToken::Comma if cur.is_some() => {
                process_directive(cc, std::mem::take(&mut cur).unwrap(), None, None).ok();
            }
            CCToken::Comma => {}
            CCToken::Equals if cur.is_some() => {
                l = process_argument(cc, std::mem::take(&mut cur).unwrap(), l)?;
            }
            CCToken::Equals => {
                return Err(());
            }
            _ if cur.is_some() => {
                return Err(());
            }
            directive => {
                cur = Some(directive);
            }
        }
    }
    if cur.is_some() {
        process_directive(cc, std::mem::take(&mut cur).unwrap(), None, None).ok();
    }
    Ok(())
}

fn put_bool(ret: &mut BytesMut, v: &bool, s: &[u8]) {
    if *v {
        if !ret.is_empty() {
            ret.put(&b", "[..]);
        }
        ret.put_slice(s);
    }
}

fn put_u64(ret: &mut BytesMut, v: &Option<u64>, s: &[u8]) {
    if let Some(v) = v {
        if !ret.is_empty() {
            ret.put(&b", "[..]);
        }
        ret.put_slice(s);
        ret.put_u8(b'=');
        ret.put(v.to_string().as_bytes());
    }
}

fn put_headerlist(ret: &mut BytesMut, v: &Option<Vec<HeaderName>>, s: &[u8]) {
    if let Some(v) = v {
        if !ret.is_empty() {
            ret.put(&b", "[..]);
        }
        ret.put_slice(s);
        if !v.is_empty() {
            ret.put_u8(b'=');
            ret.put_u8(b'"');
            let mut first = true;
            for h in v {
                if !first {
                    ret.put_u8(b',');
                    first = false;
                }
                ret.put_slice(h.as_ref());
            }
            ret.put_u8(b'"');
        }
    }
}

fn put_other(ret: &mut BytesMut, v: &Vec<(String, Option<Vec<u8>>)>) {
    for (h, p) in v {
        if !ret.is_empty() {
            ret.put(&b", "[..]);
        }
        ret.put(h.as_bytes());
        if let Some(v) = p {
            ret.put_u8(b'=');
            ret.put(v.as_slice());
        }
    }
}

impl Header for CacheControl {
    fn name() -> &'static HeaderName {
        &http::header::CACHE_CONTROL
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        I: Iterator<Item = &'i HeaderValue>,
    {
        let mut ret = Self::default();

        while let Some(h) = values.next() {
            process_header(&mut ret, h.as_bytes()).map_err(|_| headers::Error::invalid())?
        }
        // FIXME: if no tokens at all, error
        Ok(ret)
    }

    fn encode<E>(&self, values: &mut E)
    where
        E: Extend<HeaderValue>,
    {
        let mut ret = BytesMut::new();
        put_u64(&mut ret, &self.max_age, b"max-age");
        put_u64(&mut ret, &self.max_stale, b"max-stale");
        put_u64(&mut ret, &self.min_fresh, b"min-fresh");
        put_headerlist(&mut ret, &self.no_cache, b"no-cache");
        put_bool(&mut ret, &self.no_store, b"no-store");
        put_bool(&mut ret, &self.no_transform, b"no-transform");
        put_bool(&mut ret, &self.only_if_cached, b"only-if-cached");
        put_bool(&mut ret, &self.must_revalidate, b"must-revalidate");
        put_bool(&mut ret, &self.transform, b"transform");
        put_headerlist(&mut ret, &self.private, b"private");
        put_bool(&mut ret, &self.proxy_revalidate, b"proxy-revalidate");
        put_bool(&mut ret, &self.public, b"public");
        put_u64(&mut ret, &self.s_maxage, b"s-maxage");
        put_bool(&mut ret, &self.immutable, b"immutable");
        put_u64(
            &mut ret,
            &self.stale_while_revalidate,
            b"stale-while-revalidate",
        );
        put_u64(&mut ret, &self.stale_if_error, b"stale-if-error");
        put_other(&mut ret, &self.other);
        if !ret.is_empty() {
            let value = HeaderValue::from_maybe_shared(ret.freeze()).unwrap();
            values.extend(std::iter::once(value));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use headers::{HeaderMap, HeaderMapExt};

    #[test]
    fn test_header_list() {
        assert_eq!(
            header_list(b"foo,bar,bazzz"),
            Ok(vec![
                "foo".try_into().unwrap(),
                "bar".try_into().unwrap(),
                "bazzz".try_into().unwrap()
            ])
        );
        assert_eq!(
            header_list(b",,, , foo, bar, bazzz ,    , "),
            Ok(vec![
                "foo".try_into().unwrap(),
                "bar".try_into().unwrap(),
                "bazzz".try_into().unwrap()
            ])
        );
        assert_eq!(header_list(b""), Ok(vec![]));
        assert_eq!(header_list(b"foo bar"), Err(()));
    }

    #[test]
    fn it_works() {
        let mut h = HeaderMap::new();
        h.insert(
            "Cache-Control",
            HeaderValue::from_static(
                r#",,, , , ,, ,,,,  no-store ,, , ,,zak     ="\\\"" ,, ,immutable     , ,     ,"#,
            ),
        );
        let cc = h.typed_get::<CacheControl>().unwrap();
        //assert_eq!(format!("{:?}", cc), "");
        h.remove("Cache-Control");
        h.typed_insert(cc);
        assert_eq!(h["Cache-Control"], r#"no-store, immutable, zak="\\\"""#);
    }
}
