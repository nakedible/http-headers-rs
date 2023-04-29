# http-headers-typed-rs

RFC correct and performant typed HTTP header parsing and generation.

## Design principles

- All parsing should be done strictly according to most recent RFC ABNF, with no omissions, except in cases where real world use case disagrees with specification
- Every header should round-trip as semantically equivalent for valid values, preserving unknown entries
- Support both client and server, including proxy use cases

## Why not `hyperion/headers`?

The idea is sound, but we have differing design principles. It seems `hyperion/headers` is built by people of more practical nature.
