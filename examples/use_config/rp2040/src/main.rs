#![no_main]
#![no_std]

mod keymap;
mod vial;

use crate::keymap::KEYMAP;
use rmk::macros::rmk_keyboard;
use vial::{VIAL_KEYBOARD_DEF, VIAL_KEYBOARD_ID};

// Create and run your keyboard with a single macro: `rmk_keyboard`, that's it!
#[rmk_keyboard]
mod keyboard {}
