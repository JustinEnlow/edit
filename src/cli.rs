use std::error::Error;
use std::io::{self, Read};

use crate::application::Application;
use crate::config::CURSOR_STYLE;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture}, execute, terminal, ExecutableCommand
};
use ratatui::{backend::CrosstermBackend, Terminal};



// -t flag may require session_id in future client/server impl    //edit -t [session_id] [client_id]
// -p could be for passing commands to server(although, this isn't necessary with filesystem ipc. user could just write to server in file instead)
//TODO: accept a --readonly flag, to disallow editing a buffer
//files with permissions set to read only should automatically set Application's read_only value to true. writing to a read only file should never be permitted. user can change file permissions using another utility if they truly wish to modify the file
//ex: edit file_name.rs                     //automatically set the named buffer, the contents of which are the contents of file_name.rs, to read only, if file_name.rs' permissions are read only. otherwise, read only is false
//ex: edit --readonly file_name.rs          //explicitly set the named buffer, the contents of which are the contents of file_name.rs, to read only
//ex: edit --readonly -p < file_name.rs     //explicitly set the scratch buffer, the contents of which are the contents of file_name.rs, to read only
//ex: date | edit --readonly -p             //explicitly set the scratch buffer, the contents of which are the output from the date program, to read only
//TODO: file perms being read_only and passing the read_only flag may need to be handled differently, because file perms can change in the time the buffer is open
//on a read_only perms file, maybe let the buffer be edited, but emit a read_only warning on save attempt?...
const USAGE: &'static str = "
Usage: edit [Options] [<file_path>]

Options:
    -t      use stdin to populate a temporary, un-named buffer
";



pub fn parse_args() -> Result<(), Box<dyn Error>>{
    let args: Vec<String> = std::env::args().collect();
    match args.len(){
        2 => {
            let arg = args[1].clone();
            let mut terminal = setup_terminal()?;
            let (buffer_text, file_path, read_only) = match arg.as_str(){
                "-t" => {
                    //init app with buffer from stdin
                    let mut buffer_text = String::new();
                    io::stdin().read_to_string(&mut buffer_text).expect("Failed to read from stdin");
                    //TODO: strip ansi escape codes from buffer_text (some utilities will write text containing ansi escape codes to their stdout, which messes up edit's display. these need to be removed...)
                    //this may only matter for TUI client implementation... //wouldn't be needed if terminals didn't operate using ansi escape codes
                    //println!("{}", input);

                    (buffer_text, None, false/*true only if readonly flag passed*/)
                }
                _ => {
                    //init app with buffer from file path
                    let maybe_valid_file_path = arg;
                    //let path = std::path::PathBuf::from(maybe_valid_file_path).canonicalize()?; //may need to restore terminal here, if path invalid
                    let path = match std::path::PathBuf::from(maybe_valid_file_path).canonicalize(){
                        Ok(path) => path,
                        Err(/*e*/_) => {
                            restore_terminal(&mut terminal)?;
                            //return Err(Box::new(e));
                            println!("{}", USAGE);
                            return Err("invalid argument supplied".into());
                        }
                    };
                    let buffer_text = std::fs::read_to_string(&path)?;
                    //TODO: ensure buffer_text doesn't contain any \t(and maybe others) chars, because it messes up edit's display
                    //these should be converted to TAB_WIDTH number of spaces

                    //TODO: check if we have write permissions for file or not(read_only)

                    (buffer_text, Some(path), false/*true if file is read_only, or if read_only flag passed*/)
                }
            };

            match Application::new(&buffer_text, file_path, read_only, &terminal){
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

            restore_terminal(&mut terminal)
        }
        _ => {
            println!("{}", USAGE);
            Err("invalid argument supplied".into())
        }
    }
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>, Box<dyn Error>>{
    let mut stdout = std::io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(crossterm::terminal::EnterAlternateScreen)?;
    stdout.execute(CURSOR_STYLE)?;
    //
    stdout.execute(EnableMouseCapture)?;    //without this, mouse scroll seems to call whatever method is assigned at keypress up/down, and multiple times...
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
    terminal.backend_mut().execute(DisableMouseCapture)?;   //restore default terminal mouse behavior
    //
    terminal.show_cursor()?;
    
    Ok(())
}
