//#![warn(unused_results)]

use crate::application::Application;
use std::error::Error;

mod application;
mod ui;

// TODO: define panic hook to restore terminal to valid state   //https://ratatui.rs/recipes/apps/panic-hooks/

fn main() -> Result<(), Box<dyn Error>>{
    let file_path = if std::env::args().count() > 1 && std::env::args().count() < 3{
        std::env::args().nth(1).unwrap()
    }else if std::env::args().count() > 2{
        panic!("Too many arguments passed in.");
    }else{
        panic!("No file path provided.");
    };
    
    let mut app = Application::new()?;
    if let Err(e) = app.run(file_path){
        app.restore_terminal()?;
        println!("Encountered an error while running nlo code editor. error: {e}");
        return Err(e);
    }

    app.restore_terminal()?;

    Ok(())
}
