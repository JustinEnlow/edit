use std::{
    panic,
    io::{self, Error, Read, Stdout, IsTerminal},
    sync::mpsc,
    thread,
    path::PathBuf,
};
use crossterm::{
    execute, 
    event::{self, DisableMouseCapture, EnableMouseCapture}, 
    terminal, 
    ExecutableCommand
};
use ratatui::{
    backend::CrosstermBackend, 
    Terminal
};
use edit::{
    application::{Application, Event},
    //config::CURSOR_STYLE
};



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

//TODO: figure out how to create a man page, and put this explanatory content(and others) there...

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
// edit can now also open directories as a named buffer, the contents of which is a list of the directory's children, with subdirectories
// being appended by '/'. if use_full_file_path config setting is true, the full path will be displayed, otherwise, just the file/subdirectory
// name will be displayed.
// a directory listing can be opened as a temp buffer using: ls <directory> | edit -t
// however, this will not have the appended '/', because we are just directly piping ls's output into the temp buffer, and cannot reason about the content
//TODO: make sure if directory, staus bar also displays an appended '/'

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
                                        echo idk | edit -t
                                        edit -t < some_file
    -r, --read_only               sets the buffer to read only
    -l, --line <line_number>      places the primary cursor at line(not implemented)
    -c, --column <column_number>  places the primary cursor at column(not implemented)
        --tutor                   opens buffer with tutorial text
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
struct ParsedArgs<'a>{
    temp_buffer: bool,
    read_only: bool,
    file_path: Option<PathBuf>,
    _line_number: u16,
    _column_number: u16,    //TODO: ensure cannot be used along with line_number or column_number       start_selection: StartSelection{Position{line: u16, column: u16}, Regex(String)}    default to StartSelection::Position{line: 1, column: 1}
    _regex: &'a str,
    open_tutorial: bool
}
enum ArgParseResult<'a>{
    Ok(ParsedArgs<'a>),
    PreTerminalSetupOk(&'a str),
    PreTerminalSetupError(&'a str),
}
//TODO?: maybe add overrides for path of start file, path of mount, and a flag for automount via 9pfuse?... 
//or auto_mount: Option<Path>, None if no automount desired
fn parse_args<'a>() -> ArgParseResult<'a>{
    let mut temp_buffer = false;
    let mut read_only = false;
    let mut file_path: Option<PathBuf> = None;
    let mut _line_number = 1;//0;
    let mut _column_number = 1;//0;
    let mut _regex = "";
    let mut open_tutorial = false;

    let mut args = std::env::args();
    let _ = args.next();    //discard program name, which is always the first arg
    while let Some(arg) = args.next(){
        match arg.as_str(){
            "-h" | "--help" => {return ArgParseResult::PreTerminalSetupOk(USAGE);}
            "-v" | "--version" => {return ArgParseResult::PreTerminalSetupOk(env!("CARGO_PKG_VERSION"));}
            "-t" | "--temp_buffer" => {temp_buffer = true;}
            "-r" | "--read_only" => {read_only = true;}
            "-l" | "--line" => {
                if let Some(line) = args.next(){
                    if let Ok(line) = line.parse(){_line_number = line;}
                    else{return ArgParseResult::PreTerminalSetupError("associated argument must be an unsigned integer for flag: -l, --line");}
                }else{return ArgParseResult::PreTerminalSetupError("missing associated argument for flag: -l, --line");}
            }
            "-c" | "--column" => {
                if let Some(column) = args.next(){
                    if let Ok(column) = column.parse(){_column_number = column;}
                    else{return ArgParseResult::PreTerminalSetupError("associated argument must be an unsigned integer for flag: -c, --column");}
                }else{return ArgParseResult::PreTerminalSetupError("missing associated argument for flag: -c, --column");}
            }
            "--tutor" => {open_tutorial = true;}
            //anything else will always be interpreted as a file path...
            path => {
                if let Ok(_file_path) = std::path::PathBuf::from(path).canonicalize(){
                    //if _file_path.is_dir(){
                    //    return pre_terminal_setup_error("path must be to a file, not a directory");
                    //}
                    file_path = Some(_file_path);
                }else{return ArgParseResult::PreTerminalSetupError("invalid file path");}
            }
        }
    }
    if temp_buffer && file_path.is_some(){return ArgParseResult::PreTerminalSetupError("temp buffer content must be piped over stdin");}
    ArgParseResult::Ok(
        ParsedArgs{
            temp_buffer,
            read_only,
            file_path,
            _line_number,
            _column_number,
            _regex,
            open_tutorial
        }
    )
}
fn main() -> Result<(), String>{
    set_panic_hook();

    let args = match parse_args(){
        ArgParseResult::PreTerminalSetupError(error) => return pre_terminal_setup_error(&error),
        ArgParseResult::PreTerminalSetupOk(message) => return pre_terminal_setup_ok(&message),
        ArgParseResult::Ok(args) => args
    };

    let mut terminal = match setup_terminal(){
        Ok(term) => term,
        Err(e) => return pre_terminal_setup_error(&format!("{e}"))
    };

    //let config = edit::config::Config{
    //    semantics: edit::selection::CursorSemantics::Block,
    //    use_full_file_path: false,
    //    use_hard_tab: false,
    //    tab_width: 4,
    //    view_scroll_amount: 1,
    //    show_cursor_column: false,
    //    show_cursor_line: true,
    //    keybinds: edit::keybind::default_keybinds()
    //};
    let config = edit::config::Config::default();   //TODO?: load user config from config file  //or just pass config items as flag args, then update at runtime via 9p interface

    //init app with buffer from tutorial file
    let (buffer_text, file_path, read_only) = if args.open_tutorial{
        (
            match std::fs::read_to_string("/home/j/software/edit_suite/edit/EDIT_TRAMPOLINE"){
                Err(_) => return post_terminal_setup_error("EDIT_TRAMPOLINE file does not exist at expected path", false, &mut terminal),
                Ok(tutorial_text) => tutorial_text
            }
            //alternatively:
            //edit::tutorial::TUTORIAL.to_string()  //static tutorial text as crate defined str instead of separate file on disk
            //edit::tutorial::tutorial_text()       //dynamically constructed version of tutorial text (can print with user defined keybinds, etc)
            ,
            None,
            args.read_only
        )
    }
    //init app with buffer from stdin
    else if args.temp_buffer{
        let mut buffer_text = String::new();
        //fixes bug when "edit -t" called with no stdin supplied...
        if io::stdin().is_terminal(){
            return post_terminal_setup_error("no stdin supplied for temporary buffer", true, &mut terminal);
        }
        //
        if let Err(e) = io::stdin().read_to_string(&mut buffer_text){
            return post_terminal_setup_error(&format!("{e}"), true, &mut terminal);
        }
        
        //TODO: strip ansi escape codes from buffer_text (some utilities will write text containing ansi escape codes to their stdout, which messes up edit's display. these need to be removed...)
        //this may only matter for TUI client implementation... //wouldn't be needed if terminals didn't operate using ansi escape codes
        (buffer_text, None, args.read_only)
    }
    //init app with buffer from provided file
    else{
        //TODO: if provided file is readonly, set read_only to true
        let parsed_file_path = match &args.file_path{
            Some(file_path) => file_path,
            None => {return post_terminal_setup_error("invalid or no arguments provided", true, &mut terminal);}
        };
        //let buffer_text = match std::fs::read_to_string(verified_file_path){
        //    Ok(text) => text,
        //    Err(e) => return post_terminal_setup_error(&format!("{e}"), true, &mut terminal),
        //};
        let buffer_text = if parsed_file_path.is_file(){
            match std::fs::read_to_string(parsed_file_path){
                Ok(text) => text,
                Err(e) => return post_terminal_setup_error(&format!("{e}"), true, &mut terminal),
            }
        }else{
            let mut buf = String::new();
            let dir_content = match std::fs::read_dir(parsed_file_path){
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
                //TODO?: maybe insert full entry path if config.use_full_file_path == true
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
        //actually, we need to handle '\t' properly...
        (buffer_text, args.file_path, args.read_only)
    };

    //TODO: could pass column_number, line_number and regex here, returning Err() if invalid positions...
    //enum OpenPosition{Point{line: u16, column: u16}, Regex{regex: String}},   open_position: Option<OpenPosition>
    let mut app = match Application::new(config, &buffer_text, file_path, read_only, &terminal){
        Err(e) => return post_terminal_setup_error(&e, false, &mut terminal),
        Ok(app) => app
    };

    // TODO: have input and 9p on separate threads, editor on main. send events via mpsc channel.   asynchronous inputs -> synchronous events
    // also, possibly add "tick" thread for timed events such as cursor blink...
    // should these eventing threads be moved into Application instead?
    // maybe in new and hold handles in app struct, or in app.run before the loop
    // then app can check handle status in the run loop, and handle panic/errors?...
    let (event_tx, event_rx) = mpsc::channel::<Event>();
    
    //input thread
    let input_event_tx = event_tx.clone();
    let input_thread_handle = thread::spawn(|| {
        handle_terminal_events(input_event_tx);
    });

    //9p thread
//    let ninep_event_tx = event_tx.clone();
//    let listener = match setup_listener(){
//        Err(e) => return post_terminal_setup_error(&format!("{e}"), false, &mut terminal),
//        Ok(listener) => listener
//    };
//    let ninep_thread_handle = thread::spawn(||{
//        handle_ninep_events(listener, ninep_event_tx);
//    });
    
    if let Err(e) = app.run(&mut terminal, event_rx){
        return post_terminal_setup_error(&e, false, &mut terminal);
    }
    
    match restore_terminal(&mut terminal){
        Err(e) => Err(format!("{e}")),
        Ok(()) => Ok(()),
    }
}

use std::os::unix::net::UnixListener;
use std::path::Path;
fn setup_listener() -> Result<UnixListener, String>{
    //let socket_path = "/tmp/ns.j.:1/edit9p.sock";
    let socket_path = "/tmp/ns.j.:0/edit9p.sock";
    //let socket_path = "/tmp/edit9p.sock";
    if Path::new(socket_path).exists(){
        if let Err(e) = std::fs::remove_file(socket_path){
            return Err(format!("edit::main.rs::setup_listener::remove_file: {e}"));
        }
    }
    match UnixListener::bind(socket_path){
        Err(e) => Err(format!("edit::main.rs::setup_listener::bind {e}")),
        Ok(listener) => Ok(listener)
    }
}

//TODO: blocking read in the terminal_events thread meant we had a signifigant lag when main thread exits.
//solution was to event::poll before event::read.
//reasonable poll duration is 10-16 ms
// we still have a signifigant quit lag sometimes... hitting any other key seems to force it to work somehow...
fn handle_terminal_events(event_tx: mpsc::Sender<Event>){
    loop{
        match crossterm::event::poll(std::time::Duration::from_millis(16)){
            Err(_) => continue,
            Ok(_) => {
                let event = crossterm::event::read().expect("edit/src/main:/fn handle_terminal_events/: could not read crossterm event");
                match event{
                    event::Event::Key(key_event) => event_tx.send(Event::KeyboardInput(key_event)).unwrap(),
                    event::Event::Mouse(mouse_event) => {event_tx.send(Event::MouseInput(mouse_event)).unwrap()}
                    event::Event::Resize(width, height) => {event_tx.send(Event::Window(edit::application::WindowEvent::Resize{width, height})).unwrap()}
                    event::Event::FocusLost => {event_tx.send(Event::Window(edit::application::WindowEvent::FocusLost)).unwrap()}
                    event::Event::FocusGained => {event_tx.send(Event::Window(edit::application::WindowEvent::FocusGained)).unwrap()}
                    event::Event::Paste(_) => {}
                }
            }
        }
    }
}
//|-------------serve9p----------------|-intermed-|-------editor--------|-intermed-|---------serve9p---------|
//transport -> Tmessage -> handle -> fs_tx -> event_tx -> editor -> event_rx -> fs_rx -> Rmessage -> transport
fn handle_ninep_events(listener: UnixListener, event_tx: mpsc::Sender<Event>){
    use serve9p::file_system::FsRequest;

    let (fs_tx, fs_rx) = mpsc::channel::<FsRequest>();
    loop{
        //serve9p handler
        let (stream, _) = listener.accept().unwrap();//map_err(|e| e.to_string())?;
        let connection_tx = fs_tx.clone();

        //thread per connection (or async in future?...)
        thread::spawn(move || {
            let mut connection = serve9p::server::Connection::new();
            let mut transport = stream;
            //log: println!("client connected\n");
            //if let Err(e) = connection.run(&mut transport, &connection_tx){
            //    eprintln!("connection error: {}", e);
            //}
            use serve9p::transport::Transport;
            use serve9p::codec::Tmessage;
            loop{
                match transport.read_into(&mut connection.message_buffer){
                    Err(e) => return Err(e.to_string()),
                    Ok(bytes_read) => {
                        if bytes_read == 0{return Ok(());}  //client disconnected
                    }
                };
                match connection.try_frame_message(){
                    None => {}  //wait for full 9p message
                    Some(framed_message) => {
                        let t_msg = Tmessage::decode(&framed_message)?;
                        let r_msg = connection.handle(t_msg, &connection_tx);
                        let reply = r_msg.clone().encode();
                        //log response
                        println!("Server responded: \n{:?}", reply);
                        println!("{:#?}\n", r_msg);
                        match transport.write_from(&reply){
                            Err(e) => return Err(e.to_string()),
                            Ok(_bytes_written) => {}
                        };
                    }
                }
            }
        });

        //intermediate handler
        //receive fs_request from serve9p
        match fs_rx.recv().unwrap(){
            FsRequest::Attach{uname, aname, reply} => {
                //create intermediate channel
                let (intermediate_tx, intermediate_rx) = mpsc::channel();
                //wrap in event
                let fs_request_event = Event::NineP(FsRequest::Attach{uname, aname, reply: intermediate_tx});
                //send to editor
                event_tx.send(fs_request_event).unwrap();
                //get result from editor
                let result = intermediate_rx.recv().unwrap();
                //send result back to serve9p for writing to transport
                reply.send(result).unwrap();
            }
            FsRequest::Walk{user, qid_path, wnames, reply} => {}
            FsRequest::Open{user, qid_path, mode, reply} => {}
            FsRequest::Read{qid_path, offset, count, reply} => {}
            FsRequest::Write{qid_path, offset, data, reply} => {}
            FsRequest::Clunk{qid_path, reply} => {}
            FsRequest::Stat{qid_path, reply} => {}
        };
    }
}

fn set_panic_hook(){
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
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, Error>{
    let mut stdout = io::stdout();
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

pub fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), Error>{
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
