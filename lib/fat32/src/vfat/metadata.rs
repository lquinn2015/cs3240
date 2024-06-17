use core::fmt;

use alloc::string::String;

use crate::traits;

/// A date as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Date(u16);

/// Time as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Time(u16);

/// File attributes as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Attributes(u8);

macro_rules! ShiftFn {
    ($name:ident, $amt:literal) => {
        fn $name(&self) -> bool {
            ((self.0 >> $amt) & 1) != 0
        }
    };
    ($name:ident, $amt:literal, $mask:literal, $oty:ty) => {
        fn $name(&self) -> $oty {
            ((self.0 >> $amt) & $mask) as $oty
        }
    };

    ($name:ident, $amt:literal, $mask:literal, $oty:ty, + $offset:literal) => {
        fn $name(&self) -> $oty {
            $offset + ((self.0 >> $amt) & $mask) as $oty
        }
    };
    ($name:ident, $amt:literal, $mask:literal, $oty:ty, * $offset:literal) => {
        fn $name(&self) -> $oty {
            ($offset * ((self.0 >> $amt) & $mask)) as $oty
        }
    };
}

impl Attributes {
    ShiftFn! {read_only, 1}
    ShiftFn! {hidden, 2}
    ShiftFn! {system, 3}
    ShiftFn! {volumn_id, 4}
    ShiftFn! {directory, 5}
    ShiftFn! {archive, 6}
    fn lfn(&self) -> bool {
        (self.0 & 0xF) == 0xF
    }
}

impl Date {
    const MONTHS: [&'static str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    ShiftFn!(year, 9, 0x7f, usize, +1980);
    ShiftFn!(month, 5, 0xf, u8);
    ShiftFn!(day, 0, 0x1f, u8);
}

impl Time {
    ShiftFn!(hour, 11, 0x1f, u8);
    ShiftFn!(minutes, 5, 0x1f, u8);
    ShiftFn!(seconds, 0, 0x1f, u8, *2);
}

/// A structure containing a date and time.
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct Timestamp {
    pub date: Date,
    pub time: Time,
}

/// Metadata for a directory entry.
#[derive(Default, Debug, Clone)]
pub struct Metadata {
    // FIXME: Fill me in.
    attrs: Attributes,
    created: Timestamp,
    modified: Timestamp,
    accessed: Date,
}

impl traits::Timestamp for Timestamp {
    fn year(&self) -> usize {
        self.date.year()
    }
    fn month(&self) -> u8 {
        self.date.month()
    }
    fn day(&self) -> u8 {
        self.date.day()
    }
    fn hour(&self) -> u8 {
        self.time.hour()
    }
    fn minute(&self) -> u8 {
        self.time.minutes()
    }
    fn second(&self) -> u8 {
        self.time.seconds()
    }
}

// FIXME: Implement `traits::Metadata` for `Metadata`.
impl traits::Metadata for Metadata {
    type Timestamp = self::Timestamp;
}

// FIXME: Implement `fmt::Display` (to your liking) for `Metadata`.
