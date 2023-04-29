use chumsky::prelude::*;

pub fn ows<'a>() -> impl Parser<'a, &'a [u8], ()> {
    text::inline_whitespace()
}

// pub fn param<'a>() -> impl Parser<'a, &'a [u8], Vec<[u8]>> {
//     let token = {
//         one_of(b"!#$%&'*+.^_`|~0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ")
//             .repeated()
//             .slice()
//     };
//     let double_quoted = none_of(b"\"\\").repeated().padded_by(just(b'"')).slice();
//     let double_quoted_e = none_of(b"\"\\")
//         .ignored()
//         .or(just(b'\\').then(any()).ignored())
//         .repeated()
//         .padded_by(just(b'"'))
//         .slice();

//     token
//         .ignored()
//         .map_slice(|s: &[u8]| s.to_vec())
//         .or(double_quoted.ignored().map_slice(|s: &[u8]| s.to_vec()))
//         .or(double_quoted_e.ignored().map_slice(|s: &[u8]| {
//             let mut ret = s.to_vec();
//             ret.retain(|&c| c != b'\\');
//             ret
//         }))
// }

// pub fn argparse<'a, O>(
//     a: &'static [u8],
//     b: impl Parser<'a, Vec<[u8]>, O>,
// ) -> impl Parser<'a, &'a [u8], O> {
//     just(a)
//         .ignore_then(ows())
//         .ignore_then(just(b"="))
//         .ignore_then(ows())
//         .ignore_then(b.nested_in(param()))
// }

// pub fn cache_control<'a>() -> impl Parser<'a, &'a [u8], ()> {
//     let delta_seconds =
//         text::int(10).map(|i| std::str::from_utf8(i).unwrap().parse::<u64>().unwrap());

//     let max_age = argparse(b"max-age", delta_seconds);
//     let max_stale = argparse(b"max-stale", delta_seconds);

//     max_age.ignored()
// }
