use std::panic;
use ratatui::{backend::CrosstermBackend, Terminal};



fn main() -> Result<(), String>{
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
