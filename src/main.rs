// prevent linter warnings for these scenarios  //this should prob be set up in its own clippy.toml config file in the crate root
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::len_without_is_empty)]
#![allow(clippy::assign_op_pattern)]    //allow x = x + y, instead of x += y
#![allow(clippy::if_same_then_else)]
#![allow(clippy::match_same_arms)]  //idk,double check if we want this one...
#![allow(clippy::bool_to_int_with_if)]  //idk, double check if we want this one...
//#![warn(unused_results)]
#![allow(clippy::cast_possible_truncation)]



use std::error::Error;
use std::panic;
use ratatui::{backend::CrosstermBackend, Terminal};



fn main() -> Result<(), Box<dyn Error>>{
    //panic hook to restore terminal to valid state https://ratatui.rs/recipes/apps/panic-hooks/
    panic::set_hook(Box::new(|info| {
        // You can also log the stack trace or other relevant info here
        // Optionally, perform cleanup operations here
        let mut terminal = Terminal::new(
            CrosstermBackend::new(std::io::stdout())
        ).unwrap();
        edit::cli::restore_terminal(&mut terminal).unwrap();
        println!("Application panicked: {info:?}");
    }));

    edit::cli::parse_args()
}
