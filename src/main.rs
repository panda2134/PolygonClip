extern crate core;

use crate::ui::{run_loop};

mod ui;
mod clipping;
mod edge;
mod vec;
mod polygon;

fn main() {
    run_loop()
}
