use std::panic;
use std::error::Error;
use std::io::{self, Read};

use edit::application::Application;
//use edit::config::CURSOR_STYLE;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture}, execute, terminal, ExecutableCommand
};
use ratatui::{backend::CrosstermBackend, Terminal};
use edit::tutorial::TUTORIAL;
use std::path::PathBuf;
use std::io::Stdout;


//TODO: support navigating to specific locations by appending certain characters at end of provided file name
    //edit file_name.rs:10:15
    //edit file_name.rs:/regex/
    //edit -t file_name.rs:0        //this won't work because -t requires '< file_name.rs'
    //edit -t < file_name.rs:0      //this won't work because ...
    //edit --tutor:/regex/          //should this be supported?...
//passing flags instead would support more cases
    //edit --line 10 --column 15 file_name.rs
    //edit --search <regex> file_name.rs
    //edit --line 0 -t file_name.rs
    //edit --line 0 -t < file_name.rs
    //edit --search <regex> --tutor

//edit file_name.rs                 //open named buffer, the contents of which are the contents of file_name.rs
//edit file_name.rs -l 420 -c 69    //open named buffer, the constents of which are the contents of file_name.rs, with the cursor at :420:69
//edit -r file_name.rs              //open named buffer, the contents of which are the contents of file_name.rs, as read only
//edit -t < file_name.rs            //open unnamed buffer, the contents of which are the contents of file_name.rs
//date | edit -t                    //open unnamed buffer, the contents of which are the output from the date program
//edit -r -t < file_name.rs         //open unnamed buffer, the contents of which are the contents of file_name.rs, as read only
//date | edit -r -t                 //open unnamed buffer, the contents of which are the output from the date program, as read only
//edit -r -t -l 0 -c 0 < file_name.rs
//date | edit -r -t -l 0 -c 0
//edit --tutor                      //open unnamed buffer, the contents of which are edit's tutorial

//TODO: file perms being read_only and passing the read_only flag may need to be handled separately, 
//  because file perms can change in the time the buffer is open
//writing to a file with read only perms should never be permitted. user can change file permissions using another utility if they truly wish to modify the file
//on a read_only perms file, maybe let the buffer be edited, but emit a read_only warning on save attempt?...

const USAGE: &str = "
Usage: edit [Options] [<file_path>]

Options:
    -h, --help                    (Exclusive arg) prints help
    -v, --version                 (Exclusive arg) prints version
    -t, --temp_file               use stdin to populate a temporary, un-named buffer
    -r, --read_only               sets the buffer to read only
    -l, --line <line_number>      places the primary cursor at line(not implemented)
    -c, --column <column_number>  places the primary cursor at column(not implemented)
        --tutor                   tutorial
";



fn pre_terminal_setup_error(message: &str) -> Result<(), String>{
    println!("{USAGE}");
    Err(message.into())
}
fn pre_terminal_setup_ok(message: &str) -> Result<(), String>{
    println!("{message}");
    Ok(())
}
fn post_terminal_setup_error(message: &str, show_usage: bool, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), String>{
    if let Err(e) = restore_terminal(terminal){
        return Err(format!("{e}"));
    }
    if show_usage{println!("{USAGE}");}
    Err(message.into())
}
fn main() -> Result<(), String>{
    //panic hook to restore terminal to valid state https://ratatui.rs/recipes/apps/panic-hooks/
    panic::set_hook(Box::new(|info| {
        // You can also log the stack trace or other relevant info here
        // Optionally, perform cleanup operations here
        let mut terminal = Terminal::new(
            CrosstermBackend::new(std::io::stdout())
        ).unwrap();
        restore_terminal(&mut terminal).unwrap();
        println!("Application panicked: {info:?}");
    }));

    let mut temp_buffer = false;
    let mut read_only = false;
    let mut file_path: Option<PathBuf> = None;
    let mut _line_number = 1;//0;
    let mut _column_number = 1;//0;
    let mut open_tutorial = false;

    let mut args = std::env::args();
    let _ = args.next();    //discard program name, which is always the first arg
    while let Some(arg) = args.next(){
        match arg.as_str(){
            "-h" | "--help" => {return pre_terminal_setup_ok(USAGE);}
            "-v" | "--version" => {return pre_terminal_setup_ok(env!("CARGO_PKG_VERSION"));}
            "-t" | "--temp_buffer" => {temp_buffer = true;}
            "-r" | "--read_only" => {read_only = true;}
            "-l" | "--line" => {
                if let Some(line) = args.next(){
                    if let Ok(line) = line.parse(){_line_number = line;}
                    else{return pre_terminal_setup_error("line number must be an unsigned integer");}
                }else{return pre_terminal_setup_error("line number required");}
            }
            "-c" | "--column" => {
                if let Some(column) = args.next(){
                    if let Ok(column) = column.parse(){_column_number = column;}
                    else{return pre_terminal_setup_error("column number must be an unsigned integer");}
                }else{return pre_terminal_setup_error("column number required");}
            }
            "--tutor" => {open_tutorial = true;}
            //anything else will always be interpreted as a file path...
            path => {
                if let Ok(_file_path) = std::path::PathBuf::from(path).canonicalize(){
                    //if _file_path.is_dir(){
                    //    return pre_terminal_setup_error("path must be to a file, not a directory");
                    //}
                    file_path = Some(_file_path);
                }else{return pre_terminal_setup_error("invalid file path");}
            }
        }
    }
    if temp_buffer && file_path.is_some(){return pre_terminal_setup_error("temp buffer content must be piped over stdin");}

    let mut terminal = match setup_terminal(){
        Ok(term) => term,
        Err(e) => return Err(format!("{e}"))    //TODO: return pre/post terminal setup error
    };
        
    if open_tutorial{   //init app with buffer from tutorial file
        run_app(TUTORIAL, None, read_only, &mut terminal)
        //run_app(&crate::tutorial::tutorial_text(), file_path, read_only, &mut terminal)
    }else if temp_buffer{   //init app with buffer from stdin
        let mut buffer_text = String::new();
        //if let Err(e) = io::stdin().read_to_string(&mut buffer_text){return Err(format!("{e}"));}
        if let Err(e) = io::stdin().read_to_string(&mut buffer_text){
            return post_terminal_setup_error(&format!("{e}"), true, &mut terminal);
        }
        
        //TODO: strip ansi escape codes from buffer_text (some utilities will write text containing ansi escape codes to their stdout, which messes up edit's display. these need to be removed...)
        //this may only matter for TUI client implementation... //wouldn't be needed if terminals didn't operate using ansi escape codes
        run_app(&buffer_text, None, read_only, &mut terminal)
    }else{  //init app with buffer from provided file
        let verified_file_path = match &file_path{
            Some(file_path) => file_path,
            None => {return post_terminal_setup_error("invalid or no arguments provided", true, &mut terminal);}
        };
        //let buffer_text = match std::fs::read_to_string(verified_file_path){
        //    Ok(text) => text,
        //    //Err(e) => return Err(format!("{e}"))
        //    Err(e) => return post_terminal_setup_error(&format!("{e}"), true, &mut terminal),
        //};
        let buffer_text = if verified_file_path.is_file(){
            match std::fs::read_to_string(verified_file_path){
                Ok(text) => text,
                Err(e) => return post_terminal_setup_error(&format!("{e}"), true, &mut terminal),
            }
        }else{
            let mut buf = String::new();
            let dir_content = match std::fs::read_dir(verified_file_path){
                Err(e) => return post_terminal_setup_error(&format!("{e}"), true, &mut terminal),
                Ok(dir_content) => dir_content
            };
            //TODO?: should we change out .to_string_lossy, and guarantee valid UTF-8?...
            //TODO?: sort entries for deterministic output
            //TODO?: include full entry paths using entry.path() instead of entry.file_name()
            for entry in dir_content{
                let entry = match entry{
                    Err(e) => return post_terminal_setup_error(&format!("{e}"), true, &mut terminal),
                    Ok(entry) => entry
                };
                buf.insert_str(buf.len(), entry.file_name().to_string_lossy().as_ref());
                if entry.path().is_dir(){
                    buf.push('/');  //or platform specific separator...
                }
                buf.push('\n');
            }
            buf
        };
        //TODO: ensure buffer_text doesn't contain any \t(and maybe others) chars, because it messes up edit's display
        //these should be converted to TAB_WIDTH number of spaces
        run_app(&buffer_text, file_path, read_only, &mut terminal)
    }
}

fn run_app(buffer_text: &str, file_path: Option<PathBuf>, read_only: bool, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), String>{
    //let config = edit::config::Config{
    //    user_options: std::collections::HashMap::new(),
    //    user_commands: std::collections::HashMap::new(),
    //    semantics: edit::selection::CursorSemantics::Block,
    //    use_full_file_path: false,
    //    use_hard_tab: false,
    //    tab_width: 4,
    //    view_scroll_amount: 1,
    //    show_cursor_column: false,
    //    show_cursor_line: true,
    //    keybinds: edit::keybind::default_keybinds()
    //};
    let config = edit::config::Config::default();
    //TODO: pass a default config to start app
    //TODO: then update config from rc file. if an option is undefined in rc, it will already have a default value
    match Application::new(config, buffer_text, file_path, read_only, terminal){
        Ok(mut app) => {
            //TODO: could pass column_number and line_number here, after verifying they are valid positions...
            if let Err(e) = app.run(terminal){
                return post_terminal_setup_error(&e, false, terminal);
            }
        }
        Err(e) => {
            return post_terminal_setup_error(&e, false, terminal);
        }
    }
    
    match restore_terminal(terminal){
        Ok(()) => Ok(()),
        Err(e) => Err(format!("{e}"))
    }
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>, Box<dyn Error>>{
    let mut stdout = std::io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(crossterm::terminal::EnterAlternateScreen)?;
    //stdout.execute(CURSOR_STYLE)?;
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
