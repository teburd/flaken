# Flaken 

[![Build Status](https://travis-ci.org/bfrog/flaken.svg?branch=master)](https://travis-ci.org/bfrog/flaken)
[![Coverage Status](https://coveralls.io/repos/github/bfrog/flaken/badge.svg?branch=master)](https://coveralls.io/github/bfrog/flaken?branch=master)


A 64bit snowflake like id generator, encoder, and decoder which allows setting the
bitwidths of each portion of the 64 bit encoded id.

``` rust
let mut flake = Flaken::default().node(10).epoch(0).bitwidths(48, 12);
let id = flake.next();
let (timestamp, node, seq) = flake.decode(id);
assert!(timestamp > 0);
assert_eq!(node, 10);
assert_eq!(seq, 0);
assert_eq!(flake.encode(timestamp, node, seq), id);
```
