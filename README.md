# MonteFlake

A 64bit snowflake like id generator, encoder, and decoder which allows setting the
bitwidths of each portion of the 64 bit encoded id.

``` rust
let flake = MonteFlake::new(0);
let id = flake.next();
```
