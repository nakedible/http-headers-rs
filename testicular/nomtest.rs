use nom::branch::alt;
use nom::bytes::complete::{
    escaped_transform, tag, take_till, take_while, take_while1, take_while_m_n,
};
use nom::character::complete::{char, u64 as parser_u64};
use nom::combinator::{all_consuming, complete, into, map, map_opt, map_res, opt, value};
use nom::error::{Error, ParseError};
use nom::multi::separated_list0;
use nom::sequence::{delimited, pair, preceded, terminated};
use nom::{IResult, InputLength, Parser};

pub fn fully<'a, 'b, I, O, E>(i: I, f: impl Parser<I, O, E>) -> Option<O>
where
    I: InputLength + 'a,
    O: 'b,
    E: ParseError<I>,
{
    match all_consuming(f)(i) {
        Ok((_, ret)) => Some(ret),
        Err(_) => None,
    }
}

// RFC5234
// ABNF: ``
//          SP             =  %x20
// ``````
const SP: u8 = 0x20;
// ABNF: ```
//          HTAB           =  %x09
//                        ; horizontal tab
// ```
const HTAB: u8 = 0x09;

// ABNF: ```
//          WSP            =  SP / HTAB
//                                 ; white space
// ```
pub fn is_wsp(chr: u8) -> bool {
    matches!(chr, SP | HTAB)
}

pub fn is_obs_text(chr: u8) -> bool {
    matches!(chr, 0x80u8..=0xFFu8)
}

pub fn is_vchar(chr: u8) -> bool {
    matches!(chr, 0x21u8..=0x7Eu8)
}

// RFC9112
// ABNF: ```
//      tchar          = "!" / "#" / "$" / "%" / "&" / "'" / "*"
//                     / "+" / "-" / "." / "^" / "_" / "`" / "|" / "~"
//                     / DIGIT / ALPHA
//                     ; any VCHAR, except delimiters
// ```
#[rustfmt::skip]
pub fn is_tchar(chr: u8) -> bool {
    matches!(chr, b'!' | b'#' | b'$' | b'%' | b'&' | b'\'' | b'*' |
                  b'+' | b'-' | b'.' | b'^' | b'_' | b'`' | b'|' | b'~' |
                  b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z')
}

pub fn is_dqtext(chr: u8) -> bool {
    matches!(chr, HTAB | SP | 0x21u8 | 0x23u8..=0x5Bu8 | 0x5Du8..=0x7E | 0x80u8..=0xFFu8)
}

// ABNF: ```
//      token          = 1*tchar
// ```
pub fn token(i: &[u8]) -> IResult<&[u8], &[u8]> {
    take_while1(is_tchar)(i)
}

pub fn token_str(i: &[u8]) -> IResult<&[u8], &str> {
    token.map(|b| std::str::from_utf8(b).unwrap()).parse(i)
}

pub fn quoted_string_plain(i: &[u8]) -> IResult<&[u8], &[u8]> {
    delimited(char('"'), take_while(is_dqtext), char('"'))(i)
}

pub fn quoted_char(i: &[u8]) -> IResult<&[u8], &[u8]> {
    take_while_m_n(
        1,
        1,
        |chr| matches!(chr, HTAB | SP | 0x21u8..=0x7Eu8 | 0x80u8..=0xFFu8),
    )(i)
}

pub fn quoted_string_escaped(i: &[u8]) -> IResult<&[u8], Vec<u8>> {
    delimited(
        char('"'),
        escaped_transform(take_while1(is_dqtext), '\\', quoted_char),
        char('"'),
    )(i)
}

// ABNF: ```
//      OWS            = *( SP / HTAB )
//                     ; optional whitespace
// ```
pub fn ows(i: &[u8]) -> IResult<&[u8], &[u8]> {
    take_while(is_wsp)(i)
}

pub fn list_delim(i: &[u8]) -> IResult<&[u8], ()> {
    value((), take_while(|c| matches!(c, SP | HTAB | b',')))(i)
}

pub fn list_sep(i: &[u8]) -> IResult<&[u8], ()> {
    value((), delimited(ows, char(','), list_delim))(i)
}

// pub fn list<'a, 'b, O>(f: impl Parser<&'a [u8], O, Error<&'a [u8]>>) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Vec<O>, Error<&'a [u8]>> {
//     move |input| delimited(list_delim, separated_list0(list_sep, f), list_delim).parse(input.clone())
// }

// pub fn hox(i: &[u8]) -> IResult<&[u8], Vec<&str>> {
//     list(token_str)(i)
// }

pub fn delta_seconds(i: &[u8]) -> IResult<&[u8], u64> {
    parser_u64(i)
}

pub fn list<'a, O>(
    f: impl FnMut(&[u8]) -> IResult<&[u8], O>,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Vec<O>> {
    delimited(list_delim, separated_list0(list_sep, f), list_delim)
}

// pub fn parameter_value<'a, O: 'a>(
//     f: impl for<'c> Fn(&'c [u8]) -> IResult<&'c [u8], O>,
// ) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], O> + 'a {
//     alt((
//         token.and_then(fully(f)),
//         quoted_string_plain.and_then(fully(f)),
//         map_opt(quoted_string_escaped, move |v| fully(&v[..], f)),
//     ))
// }

// pub fn max_age(i: &[u8]) -> IResult<&[u8], u64> {
//     preceded(tag(b"max-age="), parameter_value(delta_seconds))(i)
// }

// pub fn no_cache(i: &[u8]) -> IResult<&[u8], Vec<u64>> {
//     preceded(tag(b"no-cache="), parameter_value(list(delta_seconds)))(i)
// }

// pub fn cache_control(i: &[u8]) -> IResult<&[u8], ()> {
//     let list = |f| delimited(list_delim, separated_list0(list_sep, f), list_delim);
//     let parameter_value = |f| {
//         alt((
//             token.and_then(f),
//             quoted_string_plain.and_then(f),
//             quoted_string_escaped.map(|v| &v[..]).and_then(f),
//         ))
//     };
//     let max_age = preceded(tag(b"max_age="), parameter_value(delta_seconds)).map(|s| {});
//     let no_cache = preceded(
//         tag(b"no-cache"),
//         opt(preceded(char('='), parameter_value(list(token)))),
//     )
//     .map(|s| {});
//     let no_store = tag(b"no-store").map(|s| {});
//     alt((max_age, no_cache, no_store))(i)
// }

// #[derive(Clone, Default, Debug)]
// pub struct CacheControl {
//     max_age: Option<u64>,
//     max_stale: Option<u64>,
//     min_fresh: Option<u64>,
//     no_cache: Option<Vec<String>>,
//     no_store: bool,
//     no_transform: bool,
//     only_if_cached: bool,
//     must_revalidate: bool,
//     must_understand: bool,
//     transform: bool,
//     private: Option<Vec<String>>,
//     proxy_revalidate: bool,
//     public: bool,
//     s_maxage: Option<u64>,
//     other: Vec<String>,
// }

// pub fn cache_control_token<'a, 'b: 'a>(
//     a: &'a mut CacheControl,
// ) -> impl FnMut(&'b [u8]) -> IResult<&'b [u8], ()> + 'a {
//     alt((
//         tag(b"no-store").map(|_| {
//             a.no_store = true;
//         }),
//         tag(b"no-transform").map(|_| {
//             a.no_transform = true;
//         }),
//         preceded(tag(b"max-age="), delta_seconds).map(|s| {
//             a.max_age = Some(s);
//         }),
//     ))
// }

// pub fn cache_control(i: &[u8]) -> IResult<&[u8], CacheControl> {
//     let mut c = CacheControl::default();
//     let p = |a: &mut CacheControl| {
//         alt((
//             tag(b"no-store").map(|_| {
//                 a.no_store = true;
//             }),
//             tag(b"no-transform").map(|_| {
//                 a.no_transform = true;
//             }),
//             preceded(tag(b"max-age="), delta_seconds).map(|s| {
//                 a.max_age = Some(s);
//             }),
//         ))
//     };
//     value(CacheControl::default(), p(&mut c))(i.clone())
// }

// pub fn cache_control_token(i: &[u8]) -> IResult<&[u8], CacheControlToken> {
//     alt((
//         preceded(
//             tag(b"max-age="),
//             map(delta_seconds, CacheControlToken::MaxAge),
//         ),
//         value(CacheControlToken::NoStore, tag(b"hox")),
//     ))(i)
// }

// // hax
// pub fn cache_control(i: &[u8]) -> IResult<&[u8], Vec<(&str, Option<&str>)>> {
//     let list = |f| delimited(list_delim, separated_list0(list_sep, f), list_delim);
//     list(pair(token_str, opt(preceded(tag(b"="), token_str))))(i)
// }
