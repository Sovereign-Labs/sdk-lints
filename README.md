# SDK Lints

This crate defines lints using the [`dylint` crate](https://github.com/trailofbits/dylint?tab=readme-ov-file) for use with the Sovereign SDK. 

Dylint is a fork of clippy which can load new lints at runtime. Unfortunately, it's not very well documented. I'd recomend starting with the [`clippy` development docs](https://doc.rust-lang.org/nightly/clippy/development/index.html) first, since code can be translated almost 1:1 to dylint afterward.

# Provided Lints

## Drop Linear Type

### What it does

This crate checks for drops of types that implement the `nearly_linear::DropWarning` trait.

### Why is this bad?

Types that implement this trait need some manual cleanup in some cases. Dropping can be an indication that the cleanup was forgotten.
