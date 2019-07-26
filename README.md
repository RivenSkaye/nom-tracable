# nom-tracable
Extension of [nom](https://github.com/Geal/nom) to trace parser.

[![Build Status](https://dev.azure.com/dalance/nom-tracable/_apis/build/status/dalance.nom-tracable?branchName=master)](https://dev.azure.com/dalance/nom-tracable/_build/latest?definitionId=1&branchName=master)
[![Crates.io](https://img.shields.io/crates/v/nom-tracable.svg)](https://crates.io/crates/nom-tracable)
[![Docs.rs](https://docs.rs/nom-tracable/badge.svg)](https://docs.rs/nom-tracable)

## Requirement

nom must be 5.0.0 or later.
nom-tracable can be applied to function-style parser only.

The input type of nom parser must implement `Tracable` trait.
Therefore `&str` and `&[u8]` can't be used.
You can define a wrapper type of `&str` or `&[u8]` and implement `Tracable`.

Alternatively you can use `nom_locate::LocatedSpanEx<T, TracableInfo>`.
This implements `Tracable` in this crate.

## Usage

```Cargo.toml
[dependencies]
nom-tracable = "0.1.0"
```

nom-tracable provide two features.
`forward_trace` dump parser name before execution of the parser.
`backward_trace` dump parser name after execution of the parser.
Both of them can be used simultaneously.

These features are not default.
If none of them is specified, there is no additional cost.

## Example

You can try an example by the following command.

```
$ cargo run --manifest-path=nom-tracable/Cargo.toml --example example --features "forward_trace backward_trace"
```

The output of the example is below:

![nom-tracable](https://user-images.githubusercontent.com/4331004/61949595-5252ae80-afe6-11e9-93dc-d5c5fa3a2d0e.png)

```rust
use nom::branch::*;
use nom::character::complete::*;
use nom::IResult;
use nom_locate::LocatedSpanEx;
use nom_tracable::{tracable_parser, Tracable, TracableInfo};

// Input type must implement trait Tracable
// nom_locate::LocatedSpanEx<T, TracableInfo> implements it.
type Span<'a> = LocatedSpanEx<&'a str, TracableInfo>;

// Apply tracable_parser by custom attribute
#[tracable_parser]
pub fn expr(s: Span) -> IResult<Span, String> {
    alt((expr_plus, expr_minus, term))(s)
}

#[tracable_parser]
pub fn expr_plus(s: Span) -> IResult<Span, String> {
    let (s, x) = term(s)?;
    let (s, y) = char('+')(s)?;
    let (s, z) = expr(s)?;
    let ret = format!("{}{}{}", x, y, z);
    Ok((s, ret))
}

#[tracable_parser]
pub fn expr_minus(s: Span) -> IResult<Span, String> {
    let (s, x) = term(s)?;
    let (s, y) = char('-')(s)?;
    let (s, z) = expr(s)?;
    let ret = format!("{}{}{}", x, y, z);
    Ok((s, ret))
}

#[tracable_parser]
pub fn term(s: Span) -> IResult<Span, String> {
    let (s, x) = char('1')(s)?;
    Ok((s, x.to_string()))
}

fn main() {
    let ret = expr(LocatedSpanEx::new_extra(
        "1-1+1+1-1+1+1-1+1",
        TracableInfo::default(),
    ));
    println!("{:?}", ret.unwrap().1);
}
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
