#![cfg_attr(not(test), no_std)]
#![allow(clippy::needless_doctest_main)]
#![allow(clippy::doc_nested_refdefs)]
#![cfg_attr(not(doctest), doc = include_str!("../README.md"))]

// mod icon;
// pub mod icon;

pub mod smartstate;
pub mod style;
// mod temp;
pub mod framebuf;
pub mod helpers;
pub mod interaction;
pub mod response;
pub mod ui;
pub mod widgets;

pub mod prelude {
    pub use embedded_iconoir::prelude::*;
}

pub use interaction::Interaction;
pub use response::{GuiError, GuiResult, InternalResponse, Response};
pub use ui::Ui;
pub use ui::Widget;

pub use embedded_iconoir::icons;

pub enum RefOption<'a, T> {
    Some(&'a mut T),
    None,
}

impl<T: Copy> RefOption<'_, T> {
    pub fn copy(&self) -> Option<T> {
        match self {
            RefOption::Some(t) => Some(**t),
            RefOption::None => None,
        }
    }
}

impl<'a, T> RefOption<'a, T> {
    pub fn new(t: &'a mut T) -> Self {
        RefOption::Some(t)
    }
}
