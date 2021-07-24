#![allow(
    clippy::unreadable_literal,
    clippy::upper_case_acronyms,
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    overflowing_literals,
    unused_variables,
    unused_assignments
)]
use chrono::format::{DelayedFormat, StrftimeItems};
use chrono::Local;

pub fn datetime_str() -> String {
    let now = Local::now();
    let fmt = "%Y-%m-%d %H:%M:%S";
    let dft: DelayedFormat<StrftimeItems> = now.format(fmt);
    let str_datetime: String = dft.to_string(); // 2021-01-04 20:02:09
    str_datetime
}
