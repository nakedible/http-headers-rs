pub mod util;

pub mod age;
pub mod cache_control;
pub mod expires;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        //assert_eq!(Ok((&b""[..], 353u64)), max_age(br#"max-age="3\5\3""#));
    }
}
