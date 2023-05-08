// Sorry, Rust doesn't have proper num traits built-in, so it's just easiest to do this with a macro
macro_rules! parse_generic {
    ($name:ident, $int_type:ty, $safe_digits:expr) => {
        pub fn $name<const SAT: bool>(s: &[u8]) -> Result<$int_type, ()> {
            if s.is_empty() {
                return Err(());
            }
            let mut result: $int_type = 0;
            for digit in s.iter().take($safe_digits) {
                match digit {
                    b'0'..=b'9' => result = (result * 10) + (digit & 0x0F) as $int_type,
                    _ => return Err(()),
                }
            }
            for digit in s.iter().skip($safe_digits) {
                match digit {
                    b'0'..=b'9' => {
                        if SAT {
                            result = result
                                .saturating_mul(10)
                                .saturating_add((digit & 0x0F) as $int_type)
                        } else {
                            result = result
                                .checked_mul(10)
                                .ok_or(())?
                                .checked_add((digit & 0x0F) as $int_type)
                                .ok_or(())?
                        }
                    }
                    _ => return Err(()),
                }
            }
            Ok(result)
        }
    };
}
parse_generic!(parse_u8, u8, 2);
parse_generic!(parse_u16, u16, 4);
parse_generic!(parse_u32, u32, 9);
parse_generic!(parse_u64, u64, 19);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_u64() {
        assert_eq!(parse_u64::<false>(b""), Err(()));
        assert_eq!(parse_u64::<false>(b"+0"), Err(()));
        assert_eq!(parse_u64::<false>(b"+"), Err(()));
        assert_eq!(parse_u64::<false>(b"garble0"), Err(()));
        assert_eq!(parse_u64::<false>(b"0garble"), Err(()));
        assert_eq!(parse_u64::<false>(b" 0"), Err(()));
        assert_eq!(parse_u64::<false>(b"0 "), Err(()));
        assert_eq!(parse_u64::<false>(b"-0"), Err(()));
        assert_eq!(parse_u64::<false>(b"-1"), Err(()));
        assert_eq!(parse_u64::<false>(b"0"), Ok(0u64));
        assert_eq!(parse_u64::<false>(b"000000"), Ok(0u64));
        assert_eq!(parse_u64::<false>(b"00000000000000000000"), Ok(0u64));
        assert_eq!(parse_u64::<false>(b"1"), Ok(1u64));
        assert_eq!(parse_u64::<false>(b"000001"), Ok(1u64));
        assert_eq!(parse_u64::<false>(b"00000000000000000001"), Ok(1u64));
        assert_eq!(
            parse_u64::<false>(b"9999999999999999"),
            Ok(9999999999999999u64)
        );
        assert_eq!(parse_u64::<false>(b"99999999999999999999"), Err(()));
        assert_eq!(parse_u64::<true>(b"99999999999999999999"), Ok(u64::MAX));
        assert_eq!(parse_u8::<false>(b"99"), Ok(99u8));
        assert_eq!(parse_u8::<false>(b"255"), Ok(255u8));
        assert_eq!(parse_u8::<false>(b"256"), Err(()));
        assert_eq!(parse_u8::<true>(b"256"), Ok(u8::MAX));
    }
}
