//! `Age` header

use logos::Logos;

use crate::util::{parse_u16, parse_u8};

#[derive(Logos, Debug, PartialEq)]
enum DayNameToken {
    #[token(b"Mon, ", |_| 1)]
    #[token(b"Tue, ", |_| 2)]
    #[token(b"Web, ", |_| 3)]
    #[token(b"Thu, ", |_| 4)]
    #[token(b"Fri, ", |_| 5)]
    #[token(b"Sat, ", |_| 6)]
    #[token(b"Sun, ", |_| 7)]
    Imf(u8),

    #[token(b"Monday, ", |_| 1)]
    #[token(b"Tuesday, ", |_| 2)]
    #[token(b"Wednesday, ", |_| 3)]
    #[token(b"Thursday, ", |_| 4)]
    #[token(b"Friday, ", |_| 5)]
    #[token(b"Saturday, ", |_| 6)]
    #[token(b"Sunday, ", |_| 7)]
    Rfc(u8),

    #[token(b"Mon ", |_| 1)]
    #[token(b"Tue ", |_| 2)]
    #[token(b"Web ", |_| 3)]
    #[token(b"Thu ", |_| 4)]
    #[token(b"Fri ", |_| 5)]
    #[token(b"Sat ", |_| 6)]
    #[token(b"Sun ", |_| 7)]
    Asc(u8),
}

#[derive(Logos, Debug, PartialEq)]
enum DateToken {
    #[token(b" ")]
    Space,
    #[token(b"-")]
    Minus,
    #[token(b":")]
    Colon,
    #[regex(b" [0-9]", |l| parse_u8::<false>(&l.slice()[1..]))]
    Digit1(u8),
    #[regex(b"[0-9][0-9]", |l| parse_u8::<false>(l.slice()))]
    Digit2(u8),
    #[regex(b"[0-9][0-9][0-9][0-9]", |l| parse_u16::<false>(l.slice()))]
    Digit4(u16),
    #[token(b"Jan", |_| 1)]
    #[token(b"Feb", |_| 2)]
    #[token(b"Mar", |_| 3)]
    #[token(b"Apr", |_| 4)]
    #[token(b"May", |_| 5)]
    #[token(b"Jun", |_| 6)]
    #[token(b"Jul", |_| 7)]
    #[token(b"Aug", |_| 8)]
    #[token(b"Sep", |_| 9)]
    #[token(b"Oct", |_| 10)]
    #[token(b"Nov", |_| 11)]
    #[token(b"Dec", |_| 12)]
    Mon(u8),
    #[token(b"GMT")]
    Gmt,
    None,
}

fn process_date(i: &[u8]) -> Result<(), ()> {
    let mut l = DayNameToken::lexer(i);
    let weekday = l.next().ok_or(())??;
    let mut l = l.morph::<DateToken>();
    match weekday {
        DayNameToken::Imf(weekday) => {
            let a = [(); 13].map(|_| l.next().unwrap_or(Ok(DateToken::None)).unwrap());
            match a {
                [
                    DateToken::Digit2(day),
                    DateToken::Space,
                    DateToken::Mon(month),
                    DateToken::Space,
                    DateToken::Digit4(year),
                    DateToken::Space,
                    DateToken::Digit2(hour),
                    DateToken::Colon,
                    DateToken::Digit2(minute),
                    DateToken::Colon,
                    DateToken::Digit2(second),
                    DateToken::Space,
                    DateToken::Gmt,
                ] => Ok(()),
                _ => Err(()),
            }
        }
        DayNameToken::Rfc(weekday) => {
            let a = [(); 13].map(|_| l.next().unwrap_or(Ok(DateToken::None)).unwrap());
            match a {
                [
                    DateToken::Digit2(day),
                    DateToken::Minus,
                    DateToken::Mon(month),
                    DateToken::Minus,
                    DateToken::Digit2(year),
                    DateToken::Space,
                    DateToken::Digit2(hour),
                    DateToken::Colon,
                    DateToken::Digit2(minute),
                    DateToken::Colon,
                    DateToken::Digit2(second),
                    DateToken::Space,
                    DateToken::Gmt,                
                ] => Ok(()),
                _ => Err(()),
            }
        }
        DayNameToken::Asc(weekday) => {
            let a = [(); 11].map(|_| l.next().unwrap_or(Ok(DateToken::None)).unwrap());
            match a {
                [
                    DateToken::Mon(month),
                    DateToken::Space,
                    DateToken::Digit2(day),
                    DateToken::Space,
                    DateToken::Digit2(hour),
                    DateToken::Colon,
                    DateToken::Digit2(minute),
                    DateToken::Colon,
                    DateToken::Digit2(second),
                    DateToken::Space,
                    DateToken::Digit4(year),
                ] => Ok(()),
                _ => Err(()),
            }
        }
    }
}
