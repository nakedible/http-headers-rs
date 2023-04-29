pub fn parse_int_saturating(s: &[u8]) -> Result<u64, ()> {
    if s.is_empty() {
        return Err(());
    }
    let mut result: u64 = 0;
    for digit in s.iter().take(19) {
        match digit {
            b'0'..=b'9' => result = (result * 10) + (digit & 0x0F) as u64,
            _ => return Err(()),
        }
    }
    for digit in s.iter().skip(19) {
        match digit {
            b'0'..=b'9' => {
                result = result
                    .saturating_mul(10)
                    .saturating_add((digit & 0x0F) as u64)
            }
            _ => return Err(()),
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_int_saturating() {
        assert_eq!(parse_int_saturating(b""), Err(()));
        assert_eq!(parse_int_saturating(b"+0"), Err(()));
        assert_eq!(parse_int_saturating(b"+"), Err(()));
        assert_eq!(parse_int_saturating(b" 0"), Err(()));
        assert_eq!(parse_int_saturating(b"0 "), Err(()));
        assert_eq!(parse_int_saturating(b"0"), Ok(0u64));
        assert_eq!(parse_int_saturating(b"000000"), Ok(0u64));
        assert_eq!(parse_int_saturating(b"000000000000000000000000000000000000000000"), Ok(0u64));
        assert_eq!(parse_int_saturating(b"1"), Ok(1u64));
        assert_eq!(parse_int_saturating(b"000001"), Ok(1u64));
        assert_eq!(parse_int_saturating(b"00000000000000000000000000000000000000001"), Ok(1u64));
        assert_eq!(parse_int_saturating(b"9999999999999999"), Ok(9999999999999999u64));
        assert_eq!(parse_int_saturating(b"99999999999999999999999999999999999999999"), Ok(u64::MAX));
    }
}
