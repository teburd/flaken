# MonteFlake

[![Build Status](https://travis-ci.org/bfrog/monteflake.svg?branch=master)](https://travis-ci.org/bfrog/monteflake)


A 64bit snowflake like id generator, encoder, and decoder which allows setting the
bitwidths of each portion of the 64 bit encoded id.

``` rust
let flake = MonteFlake::new(0);
let id = flake.next();
```
