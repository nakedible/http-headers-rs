use criterion::{black_box, criterion_group, criterion_main, Criterion};

use headers::{CacheControl, Header, HeaderMap, HeaderMapExt, HeaderName, HeaderValue};

use http_header_types::cache_control;

pub fn naive(s: &[u8]) -> u64 {
    std::str::from_utf8(s).unwrap().parse().unwrap()
}

pub fn naive_bytes_and(s: &[u8]) -> u64 {
    s.iter().fold(0, |a, c| a * 10 + (c & 0x0f) as u64)
}

pub fn naive_bytes_andx(s: &[u8]) -> u64 {
    s.iter()
        .fold(Some(0), |a, c| match (a, c) {
            (Some(a), b'0'..=b'9') => Some(a * 10 + (c & 0x0f) as u64),
            _ => None,
        })
        .unwrap()
}

pub fn naive_bytes(s: &[u8]) -> Result<u64, ()> {
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

fn atoi_test(s: &[u8]) -> Result<u64, ()> {
    atoi::atoi(s).ok_or(())
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("str::parse", |b| b.iter(|| naive(black_box(b"0002342342"))));
    // c.bench_function("nux", |b| {
    //     b.iter(|| naive_bytes_and(black_box(b"0002342342342342")))
    // });
    // c.bench_function("sux", |b| {
    //     b.iter(|| naive_bytes_andx(black_box(b"0002342342342342")))
    // });
    // c.bench_function("hex", |b| {
    //     b.iter(|| cache_control::parse_u64_log(black_box(b"0002342342342342")))
    // });
    c.bench_function("hix", |b| {
        b.iter(|| naive_bytes(black_box(b"0002342342")).unwrap())
    });
    c.bench_function("str::parsex", |b| {
        b.iter(|| naive(black_box(b"0002342342")))
    });
    c.bench_function("atoi", |b| b.iter(|| atoi_test(black_box(b"0002342342"))));
    // let mut headers = HeaderMap::new();
    // headers.insert(
    //     "Cache-Control",
    //     HeaderValue::from_static("no-cache, max-age=100"),
    // );
    // c.bench_function("cache-control-headers", |b| {
    //     b.iter(|| black_box(&headers).typed_get::<CacheControl>())
    // });
    // c.bench_function("cache-control-nom", |b| {
    //     b.iter(|| {
    //         let _ = cache_control(black_box(&headers).get("Cache-Control").unwrap().as_bytes());
    //     })
    // });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
