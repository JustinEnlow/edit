// prevent linter warnings for these scenarios  //this should prob be set up in its own clippy.toml config file in the crate root
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::len_without_is_empty)]
#![allow(clippy::assign_op_pattern)]    //allow x = x + y, instead of x += y
#![allow(clippy::if_same_then_else)]
//#![warn(unused_results)]

use crate::application::Application;
use crate::config::CURSOR_STYLE;
use std::error::Error;
use std::panic;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture}, execute, terminal, ExecutableCommand
};
use ratatui::{backend::CrosstermBackend, Terminal};

mod application;
mod keybind;
mod ui;
mod config;

// TODO: define panic hook to restore terminal to valid state   //https://ratatui.rs/recipes/apps/panic-hooks/

fn main() -> Result<(), Box<dyn Error>>{
    // Set a custom panic hook
    panic::set_hook(Box::new(|info| {
        // You can also log the stack trace or other relevant info here
        // Optionally, perform cleanup operations here
        let mut terminal = Terminal::new(
            CrosstermBackend::new(std::io::stdout())
        ).unwrap();
        restore_terminal(&mut terminal).unwrap();
        println!("Application panicked: {:?}", info);
    }));

    let file_path = get_file_path()?;

    let mut terminal = setup_terminal()?;
    
    match Application::new(&file_path, &terminal){
        Ok(mut app) => {
            if let Err(e) = app.run(&mut terminal){
                restore_terminal(&mut terminal)?;
                eprintln!("Encountered an error while running the application: {e}");
                return Err(e);
            }
        }
        Err(e) => {
            restore_terminal(&mut terminal)?;
            eprintln!("Encountered an error while setting up the application: {e}");
            return Err(e);
        }
    }

    restore_terminal(&mut terminal)?;

    Ok(())
}

fn get_file_path() -> Result<String, Box<dyn Error>>{
    let args: Vec<String> = std::env::args().collect();
    match args.len(){
        2 => Ok(args[1].clone()),
        _ => Err("Usage: <program> <file_path>".into()),
    }
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>, Box<dyn Error>>{
    let mut stdout = std::io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(crossterm::terminal::EnterAlternateScreen)?;
    stdout.execute(CURSOR_STYLE)?;
    //
    //stdout.execute(EnableMouseCapture)?;    //without this, mouse scroll seems to call whatever method is assigned at keypress up/down, and multiple times...
    //
    
    let supports_keyboard_enhancement = terminal::supports_keyboard_enhancement().unwrap_or(false);

    // only allow terminals with enhanced kb protocol support?
    //if !supports_keyboard_enhancement{
    //    panic!("this terminal does not support enhanced keyboard protocols")
    //}
    //
    
    if supports_keyboard_enhancement {
        use event::{KeyboardEnhancementFlags, PushKeyboardEnhancementFlags};
        execute!(
            stdout, 
            PushKeyboardEnhancementFlags(
                KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                //| KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
                //| KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS
                //| KeyboardEnhancementFlags::REPORT_EVENT_TYPES
            )
        )?;
    }

    let terminal = Terminal::new(
        CrosstermBackend::new(stdout)
    )?;

    Ok(terminal)
}

pub fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>
) -> Result<(), Box<dyn Error>>{
    let supports_keyboard_enhancement = terminal::supports_keyboard_enhancement().unwrap_or(false);

    if supports_keyboard_enhancement{
        terminal.backend_mut().execute(event::PopKeyboardEnhancementFlags)?;
    }
    terminal::disable_raw_mode()?;
    terminal.backend_mut().execute(crossterm::terminal::LeaveAlternateScreen)?;
    terminal.backend_mut().execute(crossterm::cursor::SetCursorStyle::DefaultUserShape)?;
    //
    //terminal.backend_mut().execute(DisableMouseCapture)?;   //restore default terminal mouse behavior
    //
    terminal.show_cursor()?;
    
    Ok(())
}
