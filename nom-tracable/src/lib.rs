//! `nom-tracable` is an extension of [nom](https://docs.rs/nom) to trace parser.
//!
//! ## Examples
//!
//! The following example show a quick example.
//!
//! ```
//! use nom::branch::*;
//! use nom::character::complete::*;
//! use nom::IResult;
//! use nom_locate::LocatedSpanEx;
//! use nom_tracable::{tracable_parser, TracableInfo};
//!
//! // Input type must implement trait Tracable
//! // nom_locate::LocatedSpanEx<T, TracableInfo> implements it.
//! type Span<'a> = LocatedSpanEx<&'a str, TracableInfo>;
//!
//! // Apply tracable_parser by custom attribute
//! #[tracable_parser]
//! pub fn term(s: Span) -> IResult<Span, String> {
//!     let (s, x) = char('1')(s)?;
//!     Ok((s, x.to_string()))
//! }
//!
//! #[test]
//! fn test() {
//!     // Configure trace setting
//!     let info = TracableInfo::new().forward(true).backward(true);
//!     let ret = term(LocatedSpanEx::new_extra("1", info));
//!     assert_eq!("\"1\"", format!("{:?}", ret.unwrap().1));
//! }
//! ```

use nom::IResult;
pub use nom_tracable_macros::tracable_parser;

pub trait Tracable: HasTracableInfo {
    fn inc_depth(self) -> Self;
    fn dec_depth(self) -> Self;
    fn format(&self) -> String;
    fn header(&self) -> String;
}

pub trait HasTracableInfo {
    fn get_tracable_info(&self) -> TracableInfo;
    fn set_tracable_info(self, info: TracableInfo) -> Self;
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TracableInfo {
    #[cfg(feature = "trace")]
    depth: usize,
    #[cfg(feature = "trace")]
    forward: bool,
    #[cfg(feature = "trace")]
    backward: bool,
    #[cfg(feature = "trace")]
    count_width: usize,
    #[cfg(feature = "trace")]
    parser_width: usize,
}

impl TracableInfo {
    pub fn new() -> Self {
        TracableInfo {
            #[cfg(feature = "trace")]
            count_width: 10,
            #[cfg(feature = "trace")]
            parser_width: 96,
            ..Default::default()
        }
    }

    #[cfg(feature = "trace")]
    pub fn depth(mut self, x: usize) -> Self {
        self.depth = x;
        self
    }

    #[cfg(not(feature = "trace"))]
    pub fn depth(self, _x: usize) -> Self {
        self
    }

    #[cfg(feature = "trace")]
    pub fn forward(mut self, x: bool) -> Self {
        self.forward = x;
        self
    }

    #[cfg(not(feature = "trace"))]
    pub fn forward(self, _x: bool) -> Self {
        self
    }

    #[cfg(feature = "trace")]
    pub fn backward(mut self, x: bool) -> Self {
        self.backward = x;
        self
    }

    #[cfg(not(feature = "trace"))]
    pub fn backward(self, _x: bool) -> Self {
        self
    }

    #[cfg(feature = "trace")]
    pub fn count_width(mut self, x: usize) -> Self {
        self.count_width = x;
        self
    }

    #[cfg(not(feature = "trace"))]
    pub fn count_width(self, _x: usize) -> Self {
        self
    }

    #[cfg(feature = "trace")]
    pub fn parser_width(mut self, x: usize) -> Self {
        self.parser_width = x;
        self
    }

    #[cfg(not(feature = "trace"))]
    pub fn parser_width(self, _x: usize) -> Self {
        self
    }
}

impl HasTracableInfo for TracableInfo {
    fn get_tracable_info(&self) -> TracableInfo {
        *self
    }

    fn set_tracable_info(self, info: TracableInfo) -> Self {
        info
    }
}

#[cfg(feature = "trace")]
impl<T: std::fmt::Display, U: HasTracableInfo> HasTracableInfo for nom_locate::LocatedSpanEx<T, U> {
    fn get_tracable_info(&self) -> TracableInfo {
        self.extra.get_tracable_info()
    }

    fn set_tracable_info(mut self, info: TracableInfo) -> Self {
        self.extra = self.extra.set_tracable_info(info);
        self
    }
}

#[cfg(feature = "trace")]
impl<T: std::fmt::Display, U: HasTracableInfo> Tracable for nom_locate::LocatedSpanEx<T, U> {
    fn inc_depth(self) -> Self {
        let info = self.get_tracable_info();
        let info = info.depth(info.depth + 1);
        self.set_tracable_info(info)
    }

    fn dec_depth(self) -> Self {
        let info = self.get_tracable_info();
        let info = info.depth(info.depth - 1);
        self.set_tracable_info(info)
    }

    fn format(&self) -> String {
        format!("{:<8} : {}", self.offset, self.fragment)
    }

    fn header(&self) -> String {
        format!("{:<8} : {}", "offset", "fragment")
    }
}

#[derive(Debug, Default)]
pub struct TracableStorage {
    forward_count: usize,
    backward_count: usize,
}

impl TracableStorage {
    fn new() -> Self {
        TracableStorage::default()
    }

    pub fn init_count(&mut self) {
        self.forward_count = 0;
        self.backward_count = 0;
    }

    pub fn get_forward_count(&self) -> usize {
        self.forward_count
    }

    pub fn get_backward_count(&self) -> usize {
        self.backward_count
    }

    pub fn inc_forward_count(&mut self) {
        self.forward_count += 1
    }

    pub fn inc_backward_count(&mut self) {
        self.backward_count += 1
    }
}

thread_local!(
    pub static TRACABLE_STORAGE: core::cell::RefCell<crate::TracableStorage> = {
        core::cell::RefCell::new(crate::TracableStorage::new())
    }
);

#[allow(unused_variables)]
pub fn forward_trace<T: Tracable>(input: T, name: &str) -> (TracableInfo, T) {
    #[cfg(feature = "trace")]
    {
        let info = input.get_tracable_info();
        let depth = info.depth;
        if (depth == 0) & (info.forward | info.backward) {
            crate::TRACABLE_STORAGE.with(|storage| {
                storage.borrow_mut().init_count();
            });
            println!(
                "\n{:<count_width$} {:<count_width$} : {:<parser_width$} : {}",
                "forward",
                "backward",
                "parser",
                input.header(),
                count_width = info.count_width,
                parser_width = info.parser_width - 11, /* Control character width */
            );
        }

        if info.forward {
            let forward_count = crate::TRACABLE_STORAGE.with(|storage| {
                storage.borrow_mut().inc_forward_count();
                storage.borrow().get_forward_count()
            });

            println!(
                "{:<count_width$} {} : {:<parser_width$} : {}",
                forward_count,
                " ".repeat(info.count_width),
                format!(
                    "{}{}-> {}{}",
                    "\u{001b}[1;37m",
                    " ".repeat(depth),
                    name,
                    "\u{001b}[0m"
                ),
                input.format(),
                count_width = info.count_width,
                parser_width = info.parser_width,
            );
        }

        let input = input.inc_depth();
        (info, input)
    }
    #[cfg(not(feature = "trace"))]
    (TracableInfo::default(), input)
}

#[allow(unused_variables)]
pub fn backward_trace<T: Tracable, U>(
    input: IResult<T, U>,
    name: &str,
    info: TracableInfo,
) -> IResult<T, U> {
    #[cfg(feature = "trace")]
    {
        let depth = info.depth;

        if info.backward {
            let backward_count = crate::TRACABLE_STORAGE.with(|storage| {
                storage.borrow_mut().inc_backward_count();
                storage.borrow().get_backward_count()
            });
            match input {
                Ok((s, x)) => {
                    println!(
                        "{} {:<count_width$} : {:<parser_width$} : {}",
                        " ".repeat(info.count_width),
                        backward_count,
                        format!(
                            "{}{}<- {}{}",
                            "\u{001b}[1;32m",
                            " ".repeat(depth),
                            name,
                            "\u{001b}[0m"
                        ),
                        s.format(),
                        count_width = info.count_width,
                        parser_width = info.parser_width,
                    );
                    Ok((s.dec_depth(), x))
                }
                Err(x) => {
                    println!(
                        "{} {:<count_width$} : {:<parser_width$}",
                        " ".repeat(info.count_width),
                        backward_count,
                        format!(
                            "{}{}<- {}{}",
                            "\u{001b}[1;31m",
                            " ".repeat(depth),
                            name,
                            "\u{001b}[0m"
                        ),
                        count_width = info.count_width,
                        parser_width = info.parser_width,
                    );
                    Err(x)
                }
            }
        } else {
            input
        }
    }
    #[cfg(not(feature = "trace"))]
    input
}