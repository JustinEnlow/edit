use crate::{
    action::{Action, EditorAction}, 
    application::Application, 
    config::{Command, DisplayMode, OptionType}, 
    selection::CursorSemantics, 
    selections
};



#[derive(PartialEq, Debug, Clone)] enum ExpansionType{Option, Value, Register, Shell}
#[derive(PartialEq, Debug, Clone)] enum WordType{
    Unquoted,                   //word
    Quoted,                     //'a word', "a word", %{a word}
    Expansion(ExpansionType)    //%value{value_name}   //valid types are "shell", "register", "option", "value"
}
#[derive(PartialEq, Debug, Clone)] pub struct Word{
    word_type: WordType,
    content: String
}
//at the extreme, i think every action could end up being a command
//in that sense, the editor is just a command parser, with command specific response behavior
//NOTE: expansions should be performed at the time of execution. fn execute_command()
pub fn parse_command(command_string: String) -> Result<Vec<Vec<Word>>, String>{
    if command_string.is_empty(){return Err(String::from("cannot parse empty string"));}
    let mut commands = Vec::new();
    let mut command = Vec::new();
    let mut word = String::new();
    let mut expansion_type_string = String::new();

    let mut inside_of_quotations = false;
    let mut quote_char: Vec<char> = Vec::new();   //may need to become a Vec<char>, so that we can have nestable brace quotes no_op %sh{{ sleep 10 } > /dev/null 2>&1 < /dev/null &}     //push to vec on '{', pop from vec on '}'. push word to command if vec empty
    let mut inside_of_comment = false;
    //let mut escape_next = false;
    let mut follows_percent = false;
    #[cfg(test)] println!("command string:\n{}\n", command_string);
    #[cfg(test)] println!("command string length: {}\n", command_string.len());
    //TODO: for grapheme in command_string.graphemes(true){
    for (_i, char) in command_string.chars().enumerate(){
        //TODO: maybe we should push '\' to word, and pop from word if the following char is something we should escape
        //that way we don't have to double escape unquoted strings containg '\'
        #[cfg(test)] println!("char: {char}, index: {_i}");
        match char{
            ' ' | '\t' => {
                if inside_of_comment{
                    #[cfg(test)] println!("char ignored as comment");
                }
                else if inside_of_quotations{   //this may become inside_of_single_quote || inside_of_double_quote || inside_of_percent_quote
                    #[cfg(test)] println!("char pushed to word: {char}");
                    word.push(char);
                }
                //else if escape_next{
                //    #[cfg(test)] println!("char pushed to word: {char}");
                //    word.push(char);
                //    escape_next = false;
                //}
                else{
                    if !word.is_empty(){
                        #[cfg(test)] println!("word pushed to command: {:?}", word);
                        command.push(Word{word_type: WordType::Unquoted, content: word});
                        //reset
                        word = String::new();
                        inside_of_quotations = false;
                        quote_char = Vec::new();
                        follows_percent = false;
                    }
                }
            }
            '\n' => {
                if inside_of_comment{
                    #[cfg(test)] println!("end of comment");
                    inside_of_comment = false;
                }
                else if inside_of_quotations{
                    #[cfg(test)] println!("char pushed to word: {char}");
                    word.push(char);
                }
                //else if escape_next{
                //    #[cfg(test)] println!("char pushed to word: {char}");
                //    word.push(char);
                //    escape_next = false;
                //}
                else{
                    if !word.is_empty(){
                        #[cfg(test)] println!("word pushed to command: {:?}", word);
                        command.push(Word{word_type: WordType::Unquoted, content: word});
                        //reset
                        word = String::new();
                        inside_of_quotations = false;
                        quote_char = Vec::new();
                        follows_percent = false;
                    }
                    if !command.is_empty(){
                        #[cfg(test)] println!("command pushed to commands: {:?}", command);
                        commands.push(command);
                        //reset
                        command = Vec::new();
                        inside_of_quotations = false;
                        quote_char = Vec::new();
                        follows_percent = false;
                    }
                }
            }
            ';' => {
                if inside_of_comment{
                    #[cfg(test)] println!("char ignored as comment");
                }
                else if inside_of_quotations{
                    #[cfg(test)] println!("char pushed to word: {char}");
                    word.push(char);
                }
                //else if escape_next{
                //    #[cfg(test)] println!("char pushed to word: {char}");
                //    word.push(char);
                //    escape_next = false;
                //}
                else{
                    if !word.is_empty(){
                        #[cfg(test)] println!("word pushed to command: {:?}", word);
                        command.push(Word{word_type: WordType::Unquoted, content: word});
                        //reset
                        word = String::new();
                        inside_of_quotations = false;
                        quote_char = Vec::new();
                        follows_percent = false;
                    }
                    if !command.is_empty(){
                        #[cfg(test)] println!("command pushed to commands: {:?}", command);
                        commands.push(command);
                        //reset
                        command = Vec::new();
                        inside_of_quotations = false;
                        quote_char = Vec::new();
                        follows_percent = false;
                    }
                }
            }
            '#' => {
                if inside_of_comment{
                    #[cfg(test)] println!("char ignored as comment");
                }
                else if inside_of_quotations{
                    #[cfg(test)] println!("char pushed to word: {char}");
                    word.push(char);
                }
                //else if escape_next{
                //    #[cfg(test)] println!("char pushed to word: {char}");
                //    word.push(char);
                //    escape_next = false;
                //}
                else{
                    #[cfg(test)] println!("comment started");
                    inside_of_comment = true;
                }
            }
//TODO: reimpl how escapes work... 
//            '\\' => {
//                if inside_of_comment{
//                    #[cfg(test)] println!("char ignored as comment");
//                }
//                else if inside_of_quotations{
//                    #[cfg(test)] println!("char pushed to word: {char}");
//                    word.push(char);
//                }
//                else if escape_next{
//                    #[cfg(test)] println!("char pushed to word: {char}");
//                    word.push(char);
//                    escape_next = false;
//                }
//                else{
//                    #[cfg(test)] println!("escaping next char");
//                    escape_next = true;
//                }
//            }

//TODO: support expansion inside double quotes: echo "the date is %sh{date}"
            '\'' | '"' => {
                if inside_of_comment{
                    #[cfg(test)] println!("char ignored as comment");
                }
                else if inside_of_quotations{
                    if quote_char.last() == Some(&char){    //if same as opening quote char
                        let _ = quote_char.pop();   //remove opening quote char from stack
                        if Option::is_none(&quote_char.last()){ //if quote char stack is empty  //should always be the case for '\'' and '"'
                            let _removed_char = word.remove(0); //remove leading '\'' or '"' from word
                            #[cfg(test)] println!("leading {} removed", _removed_char);

                            #[cfg(test)] println!("word pushed to command: {:?}", word);
                            command.push(Word{word_type: WordType::Quoted, content: word});
                            //reset necessary variables
                            word = String::new();
                            inside_of_quotations = false;
                            assert!(quote_char.is_empty()); //could prob remove if Option::is_none, and assert after quote_char.pop() above...
                        }
                    }else{  //for all other quote chars than same as opening, push to word
                        #[cfg(test)] println!("char pushed to word: {char}");
                        word.push(char);
                    }
                }
                //else if escape_next{
                //    #[cfg(test)] println!("char pushed to word: {char}");
                //    word.push(char);
                //    escape_next = false;
                //}
                else{
                    #[cfg(test)] println!("char pushed to word: {char}");
                    word.push(char);
                    inside_of_quotations = true;
                    quote_char.push(char);
                }
            }
            '%' => {
                if inside_of_comment{
                    #[cfg(test)] println!("char ignored as comment");
                }
                else if inside_of_quotations{
                    #[cfg(test)] println!("char pushed to word: {char}");
                    word.push(char);
                }
                //else if escape_next{
                //    #[cfg(test)] println!("char pushed to word: {char}");
                //    word.push(char);
                //    escape_next = false;
                //}
                else{
                    if word.is_empty(){follows_percent = true;}
                    #[cfg(test)] println!("char pushed to word: {char}");
                    word.push(char);
                }
            }
            '{' | '[' | '(' => {
                if inside_of_comment{
                    #[cfg(test)] println!("char ignored as comment");
                }
                else if inside_of_quotations{
                    #[cfg(test)] println!("char pushed to word: {char}");
                    word.push(char);
                    if quote_char.last() == Some(&char){
                        quote_char.push(char);
                    }
                }
                //else if escape_next{
                //    #[cfg(test)] println!("char pushed to word: {char}");
                //    word.push(char);
                //    escape_next = false;
                //}
                else if follows_percent{
                    expansion_type_string = word.clone();   //copy preceding chars in word as expansion_type
                    expansion_type_string.remove(0);    //remove leading '%'
                    #[cfg(test)] println!("expansion_type_string: {}", expansion_type_string);

                    #[cfg(test)] println!("char pushed to word: {char}");
                    word.push(char);
                    inside_of_quotations = true;
                    quote_char.push(char);
                    follows_percent = false;
                }
                else{
                    #[cfg(test)] println!("char pushed to word: {char}");
                    word.push(char);
                }
            }
            '}' | ']' | ')' => {    //TODO: add '<' and '>' to supported percent quote characters
                fn inverse_brace(char: char) -> Option<char>{
                    if char == '}'{Some('{')}
                    else if char == ']'{Some('[')}
                    else if char == ')'{Some('(')}
                    else{None}  //or maybe unreachable!
                }
                if inside_of_comment{
                    #[cfg(test)] println!("char ignored as comment");
                }
                else if inside_of_quotations{
                    if quote_char.last() == Some(&inverse_brace(char).unwrap()){    //if char matches opening quote char    //ok to unwrap here because inputs are verified by parent match expression
                        let _ = quote_char.pop();   //remove latest opening quote char from stack
                        if Option::is_none(&quote_char.last()){
                            if expansion_type_string.is_empty(){
                                let _removed_char = word.remove(0); //remove leading '%' from word
                                #[cfg(test)] println!("leading {} removed", _removed_char);
                                let _removed_char = word.remove(0); //remove trailing '{', '[', '(', or '<' from word
                                #[cfg(test)] println!("trailing {} removed", _removed_char);

                                #[cfg(test)] println!("word pushed to command: {:?}", word);
                                command.push(Word{word_type: WordType::Quoted, content: word});
                            }else{
                                let _removed_char = word.remove(0); //remove leading '%' from word
                                #[cfg(test)] println!("leading {} removed", _removed_char);
                                for _ in 0..expansion_type_string.len(){
                                    let _removed_char = word.remove(0); //remove expansion type chars from word
                                    #[cfg(test)] println!("expansion string {} removed", _removed_char);
                                }
                                let _removed_char = word.remove(0); //remove trailing '{', '[', '(', or '<' from word
                                #[cfg(test)] println!("trailing {} removed", _removed_char);

                                let expansion_type = match expansion_type_string.as_str(){
                                    "opt" => ExpansionType::Option,
                                    "reg" => ExpansionType::Register,
                                    "sh" => ExpansionType::Shell,
                                    "val" => ExpansionType::Value,
                                    _ => return Err(String::from("unsupported expansion type"))
                                };
                                #[cfg(test)] println!("word pushed to command: {:?}", word);
                                command.push(Word{word_type: WordType::Expansion(expansion_type), content: word});
                            }
                            word = String::new();
                            inside_of_quotations = false;
                            assert!(quote_char.is_empty());
                        }else{
                            #[cfg(test)] println!("char pushed to word: {char}");
                            word.push(char);
                        }
                    }
                    else{
                        #[cfg(test)] println!("char pushed to word: {char}");
                        word.push(char);
                    }
                }
                //else if escape_next{
                //    #[cfg(test)] println!("char pushed to word: {char}");
                //    word.push(char);
                //    escape_next = false;
                //}
                else{
                    #[cfg(test)] println!("char pushed to word: {char}");
                    word.push(char);
                }
            }
            //| => {}
            _ => {
                if inside_of_comment{
                    #[cfg(test)] println!("char ignored as comment");
                }
                else if inside_of_quotations{
                    #[cfg(test)] println!("char pushed to word: {char}");
                    word.push(char);
                }
                //else if escape_next{
                //    #[cfg(test)] println!("char pushed to word: {char}");
                //    word.push(char);
                //    escape_next = false;
                //}
                else{
                    #[cfg(test)] println!("char pushed to word: {char}");
                    word.push(char);
                }
            }
        }
    }
    if !word.is_empty(){
        #[cfg(test)] println!("word pushed to command: {:?}", word);
        command.push(Word{word_type: WordType::Unquoted, content: word});
    }
    if !command.is_empty(){
        #[cfg(test)] println!("command pushed to commands: {:?}", command);
        commands.push(command);
    }
    if commands.is_empty(){return Err(String::from("failed to parse string as commands"));}
    #[cfg(test)] println!("commands: {:?}", commands);
    #[cfg(test)] println!("");
    Ok(commands)    
}
#[test]fn empty_command_string_should_error(){
    assert_eq!(Err(String::from("cannot parse empty string")), parse_command(String::from("")));
}
#[test] fn single_unquoted_word(){
    //idk
    assert_eq!(
        Ok(
            vec![//commands
                vec![//command
                    //String::from("idk") //word
                    Word{
                        word_type: WordType::Unquoted,
                        content: String::from("idk")
                    }
                ]
            ]
        ), 
        parse_command(String::from("idk"))
    );
}
#[test] fn multiple_unquoted_words_separated_by_spaces(){
    //command --flag flag_item positional
    assert_eq!(
        Ok(
            vec![//commands
                vec![//command
                    //String::from("command"),    //word
                    Word{word_type: WordType::Unquoted, content: String::from("command")},
                    //String::from("--flag"),     //word
                    Word{word_type: WordType::Unquoted, content: String::from("--flag")},
                    //String::from("flag_item"),  //word
                    Word{word_type: WordType::Unquoted, content: String::from("flag_item")},
                    //String::from("positional")  //word
                    Word{word_type: WordType::Unquoted, content: String::from("positional")},
                ]
            ]
        ), 
        parse_command(String::from("command --flag flag_item positional"))
    );
}
#[test] fn multiple_unquoted_words_separated_by_tabs(){
    assert_eq!(
        Ok(
            vec![//commands
                vec![//command
                    //String::from("this"),       //word
                    Word{word_type: WordType::Unquoted, content: String::from("this")},
                    //String::from("command"),    //word
                    Word{word_type: WordType::Unquoted, content: String::from("command")},
                    //String::from("contains"),   //word
                    Word{word_type: WordType::Unquoted, content: String::from("contains")},
                    //String::from("tabs"),       //word
                    Word{word_type: WordType::Unquoted, content: String::from("tabs")}
                ]
            ]
        ),
        parse_command(String::from("this\tcommand\tcontains\ttabs"))
    )
}
#[test] fn newline_splits_multiple_commands(){
    //ln
    //sb
    assert_eq!(
        Ok(
            vec![//commands
                vec![//command
                    //String::from("ln")//word
                    Word{word_type: WordType::Unquoted, content: String::from("ln")}
                ], 
                vec![//command
                    //String::from("sb")//word
                    Word{word_type: WordType::Unquoted, content: String::from("sb")}
                ]
            ]
        ), 
        parse_command(String::from("ln\nsb"))
    );
}
#[test] fn semicolon_splits_multiple_commands(){
    //ln;sb
    assert_eq!(
        Ok(
            vec![//commands
                vec![//command
                    //String::from("ln")//word
                    Word{word_type: WordType::Unquoted, content: String::from("ln")}
                ], 
                vec![//command
                    //String::from("sb")//word
                    Word{word_type: WordType::Unquoted, content: String::from("sb")}
                ]
            ]
        ), 
        parse_command(String::from("ln;sb"))
    );
}
#[test] fn with_comment(){
    //# this is a comment
    //and this is a command
    assert_eq!(
        Ok(
            vec![//commands
                vec![//command
                    //String::from("and"),    //word
                    Word{word_type: WordType::Unquoted, content: String::from("and")},
                    //String::from("this"),   //word
                    Word{word_type: WordType::Unquoted, content: String::from("this")},
                    //String::from("is"),     //word
                    Word{word_type: WordType::Unquoted, content: String::from("is")},
                    //String::from("a"),      //word
                    Word{word_type: WordType::Unquoted, content: String::from("a")},
                    //String::from("command") //word
                    Word{word_type: WordType::Unquoted, content: String::from("command")}
                ]
            ]
        ), 
        parse_command(String::from("# this is a comment\nand this is a command"))
    );
}
//#[test] fn with_escaped_characters(){
//    //idk \"some shit\"
//    assert_eq!(
//        Ok(
//            vec![//commands
//                vec![//command
//                    String::from("idk"),    //word
//                    String::from("\"some"), //word
//                    String::from("shit\"")  //word
//                ]
//            ]
//        ),
//        parse_command(String::from("idk \\\"some shit\\\""))
//    );
//}
#[test] fn single_quoted_string(){
    //'this is a quoted string'
    assert_eq!(
        Ok(
            vec![
                vec![
                    //String::from("'this is a quoted string'")
                    Word{word_type: WordType::Quoted, content: String::from("this is a quoted string")}
                ]
            ]
        ),
        parse_command(String::from("'this is a quoted string'"))
    );
}
#[test] fn unbalanced_single_quoted_returns_unquoted(){
    assert_eq!(
        Ok(
            vec![
                vec![
                    Word{word_type: WordType::Unquoted, content: String::from("echo")},
                    Word{word_type: WordType::Unquoted, content: String::from("'idk")}
                ]
            ]
        ),
        parse_command(String::from("echo 'idk"))
    )
}
#[test] fn double_quoted_string(){
    //"this is a quoted string"
    assert_eq!(
        Ok(
            vec![
                vec![
                    //String::from("\"this is a quoted string\"")
                    Word{word_type: WordType::Quoted, content: String::from("this is a quoted string")}
                ]
            ]
        ),
        parse_command(String::from("\"this is a quoted string\""))
    );
}
#[test] fn unbalanced_double_quoted_returns_unquoted(){
    assert_eq!(
        Ok(
            vec![
                vec![
                    Word{word_type: WordType::Unquoted, content: String::from("echo")},
                    Word{word_type: WordType::Unquoted, content: String::from("\"idk")}
                ]
            ]
        ),
        parse_command(String::from("echo \"idk"))
    )
}
#[test] fn with_space_inside_quotation(){
    //split ' '
    assert_eq!(
        Ok(
            vec![//commands
                vec![//command
                    //String::from("split"),  //word
                    Word{word_type: WordType::Unquoted, content: String::from("split")},
                    //String::from("' '")     //word
                    Word{word_type: WordType::Quoted, content: String::from(" ")}
                ]
            ]
        ), 
        parse_command(String::from("split ' '"))
    );
}
#[test] fn with_percent_string_no_type(){
    //echo %{this is quoted}
    assert_eq!(
        Ok(
            vec![
                vec![
                    //String::from("echo"),
                    Word{word_type: WordType::Unquoted, content: String::from("echo")},
                    //String::from("%{this is quoted}")
                    Word{word_type: WordType::Quoted, content: String::from("this is quoted")},
                ]
            ]
        ),
        parse_command(String::from("echo %{this is quoted}"))
    );
}
#[test] fn unbalanced_percent_string_returns_unquoted(){
    assert_eq!(
        Ok(
            vec![
                vec![
                    Word{word_type: WordType::Unquoted, content: String::from("echo")},
                    Word{word_type: WordType::Unquoted, content: String::from("%{idk")}
                ]
            ]
        ),
        parse_command(String::from("echo %{idk"))
    )
}
#[test] fn with_percent_string_typed(){
    //echo %sh{this is quoted}
    assert_eq!(
        Ok(
            vec![
                vec![
                    //String::from("echo"),
                    Word{word_type: WordType::Unquoted, content: String::from("echo")},
                    //String::from("%sh{this is quoted}")
                    Word{word_type: WordType::Expansion(ExpansionType::Shell), content: String::from("this is quoted")}
                ]
            ]
        ),
        parse_command(String::from("echo %sh{this is quoted}"))
    );
}
#[test] fn percent_string_opt_typed(){
    //echo %opt{cursor_semantics}
    assert_eq!(
        Ok(
            vec![
                vec![
                    Word{word_type: WordType::Unquoted, content: String::from("echo")},
                    Word{word_type: WordType::Expansion(ExpansionType::Option), content: String::from("cursor_semantics")}
                ]
            ]
        ),
        parse_command(String::from("echo %opt{cursor_semantics}"))
    );
}
#[test] fn unbalanced_expansion_returns_unquoted(){
    assert_eq!(
        Ok(
            vec![
                vec![
                    Word{word_type: WordType::Unquoted, content: String::from("echo")},
                    Word{word_type: WordType::Unquoted, content: String::from("%val{idk")}
                ]
            ]
        ),
        parse_command(String::from("echo %val{idk"))
    )
}


//TODO: consider how to handle a failed command in a list of commands. should we just error on first failed command?...
//TODO: execute_command should return a new instance of Application instead of modifying existing
//that way, we could apply changes to the new instance, and if an error occurs, default back to the old instance with no changes
//also, create a successful_commands counter. on each successful command, increment by 1;
//if unsuccessful command, return "Error: {error} on command {counter + 1}"
fn execute_parsed_commands(app: &mut Application, commands: Vec<Vec<Word>>) -> Result<(), String>{//Result<Application, ApplicationError>{
    fn expand(app: &Application, word_content: &str, expansion_type: &ExpansionType) -> Result<String, String>{
        fn expand_option(app: &Application, option: String) -> Result<String, String>{
            match option.as_ref(){
                "cursor_semantics" => Ok(format!("{:?}", app.config.semantics)),
                "use_full_file_path" => Ok(app.config.use_full_file_path.to_string()),
                "use_hard_tab" => Ok(app.config.use_hard_tab.to_string()),
                "tab_width" => Ok(app.config.tab_width.to_string()),
                "view_scroll_amount" => Ok(app.config.view_scroll_amount.to_string()),
                "show_cursor_column" => Ok(app.config.show_cursor_column.to_string()),
                "show_cursor_line" => Ok(app.config.show_cursor_line.to_string()),
                "show_line_numbers" => Ok(app.ui.document_viewport.line_number_widget.show.to_string()),
                "show_status_bar" => Ok(app.ui.status_bar.show.to_string()),
                _ => {
                    match app.config.user_options.get(&option){
                        Some(option_type) => {
                            Ok(
                                match option_type{
                                    OptionType::Bool(bool) => bool.to_string(),
                                    OptionType::U8(u8) => u8.to_string(),
                                    OptionType::String(string) => string.clone()
                                }
                            )
                        }
                        None => Err(format!("{} option does not exist", option))
                    }
                }
            }
        }
        //fn expand_register() -> Result<String, ()>{Err(())}
        fn expand_shell(command_string: String, app: &Application) -> Result<String, String>{
            //check content for $values, and set as environment variables
            let mut environment_variables = std::collections::HashMap::new();
            //environment_variables.insert("MY_VAR", "environment variable content");
            if command_string.contains("$EDIT_OPT_SHOW_LINE_NUMBERS"){  //env vars can also be lower case...
                environment_variables.insert("EDIT_OPT_SHOW_LINE_NUMBERS", app.ui.document_viewport.line_number_widget.show.to_string());
            }
            if command_string.contains("$EDIT_OPT_SHOW_STATUS_BAR"){
                environment_variables.insert("EDIT_OPT_SHOW_STATUS_BAR", app.ui.status_bar.show.to_string());
            }
            
            let output = std::process::Command::new("sh"/*"bash"*/) //TODO: should this be calling the first arg in command string instead?...
                .arg("-c")
                .arg(command_string)
                //.env("MY_VAR", "environment variable content")
                .envs(&environment_variables)
                //.stdout(std::process::Stdio::piped()) //i think this is the default with .output()
                //.stderr(std::process::Stdio::piped()) //i think this is the default with .output()
                .output()
                .expect("failed to execute process");

            if output.status.success(){
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                if stdout.is_empty(){
                    Ok(String::from("shell command succeeded with empty output string"))
                }else{
                    Ok(stdout)
                }
            }else{
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                if stderr.is_empty(){
                    Err(String::from("shell command failed with empty error string"))
                }else{
                    Err(stderr)
                }
            }
        }
        //fn expand_value() -> Result<String, ()>{Err(())}

        match expansion_type{
            ExpansionType::Option => {
                expand_option(app, word_content.to_string())
            }
            ExpansionType::Register => {
                Err("register expansion unimplemented".to_string())
            }
            ExpansionType::Shell => {
                expand_shell(word_content.to_string(), app)
            }
            ExpansionType::Value => {
                Err("value expansion unimplemented".to_string())
            }
        }
    }

    //thinking this should be useful to help with handling possible expansion on each command_words.next()
    fn resolve_potential_expansion(app: &Application, word: Word) -> Result<String, String>{
        match &word.word_type{
            WordType::Expansion(expansion_type) => {
                match expand(app, &word.content, expansion_type){
                    Ok(output) => Ok(output),
                    Err(error) => Err(error)
                }
            }
            _ => Ok(word.content)
        }
    }

    for command in commands{
        let mut command_words = command.into_iter();
        let first = match command_words.next(){
            None => return Err(String::from("no command to execute")),
            Some(word) => {
                match resolve_potential_expansion(app, word){
                    Err(error) => return Err(error),
                    Ok(first) => first
                }
            }
        };
        match first.as_str(){
            "evaluate_commands" => {
                //evaluate_commands <commands>
                let commands = match command_words.next(){
                    None => return Err(String::from("too few args: evaluate_commands <commands>")),
                    Some(word) => {
                        match resolve_potential_expansion(app, word){
                            Err(error) => return Err(error),
                            Ok(commands) => commands
                        }
                    }
                };
                match command_words.next(){
                    Some(_) => return Err(String::from("too many args: evaluate_commands <commands>")),
                    None => {
                        match parse_command(commands){
                            Err(error) => return Err(error),
                            Ok(parsed_commands) => {
                                match execute_parsed_commands(app, parsed_commands){
                                    Err(error) => return Err(error),
                                    Ok(()) => {}
                                }
                            }
                        }
                    }
                }
            }
            "echo" => {
                //echo [diagnostic_mode] <message>
                let mut display_mode = DisplayMode::Info;
                let message = match command_words.next(){
                    None => return Err(String::from("too few arguments: echo [diagnostic_mode] <message>")),
                    Some(word) => {
                        match resolve_potential_expansion(app, word){
                            Err(error) => return Err(error),
                            Ok(word_content) => {
                                let mut process_next_word = true;
                                match word_content.as_str(){
                                    "--error" => display_mode = DisplayMode::Error,
                                    "--warning" => display_mode = DisplayMode::Warning,
                                    "--notify" => display_mode = DisplayMode::Notify,
                                    "--info" => {/* already set to info mode */}
                                    _ => {process_next_word = false;}
                                }
                                if process_next_word{
                                    match command_words.next(){
                                        None => return Err(String::from("too few arguments: echo [diagnostic_mode] <message>")),
                                        Some(word) => {
                                            match resolve_potential_expansion(app, word){
                                                Err(error) => return Err(error),
                                                Ok(message) => message
                                            }
                                        }
                                    }
                                }else{word_content}
                            }
                        }
                    }
                };
                match command_words.next(){
                    Some(_) => return Err(String::from("too many arguments: echo [diagnostic_mode] <message>")),
                    None => {
                        handle_message(app, display_mode.clone(), &message);
                    }
                }
            }
            
            //TODO: replace with user command: add_command term 'no_op %sh{nohup alacritty >/dev/null 2>&1 &}' 'opens a new alacritty window'
            //"term" | "t" => app.action(Action::EditorAction(EditorAction::OpenNewTerminalWindow)),

            //TODO: replace with user command: add_command toggle_line_numbers %sh{#some logic} 'toggles the display of line numbers'
            //can currently just: set_option show_line_numbers true|false
            //"toggle_line_numbers" | "ln" => app.action(Action::EditorAction(EditorAction::ToggleLineNumbers)),  //these will prob end up using set-option command...

            //TODO: replace with user command: add_command toggle_status_bar %sh{#some logic} 'toggles the display of the status bar'
            //can currently just: set_option show_status_bar true|false
            //"toggle_status_bar" | "sb" => app.action(Action::EditorAction(EditorAction::ToggleStatusBar)),      //these will prob end up using set-option command...
            
            "quit" | "q" => app.action(Action::EditorAction(EditorAction::Quit)),
            "quit!" | "q!" => app.action(Action::EditorAction(EditorAction::QuitIgnoringChanges)),
            //write buffer contents to file //should this optionally take a filepath to save to? then we don't need to implement save as    //would have to split util bar text on ' ' into separate args
            "write" | "w" => app.action(Action::EditorAction(EditorAction::Save)),
            "search" => {
                //search <regex>
                let regex = match command_words.next(){
                    None => return Err(String::from("too few args: search <regex>")),
                    Some(word) => {
                        match resolve_potential_expansion(app, word){
                            Err(error) => return Err(error),
                            Ok(regex) => regex
                        }
                    }
                };
                match command_words.next(){
                    Some(_) => return Err(String::from("too many args: search <regex>")),
                    None => {
                        //match crate::utilities::incremental_search_in_selection::selections_impl(&app.selections, &regex, &app.buffer, app.config.semantics.clone()){
                        match selections::incremental_search_in_selection(&app.selections, &regex, &app.buffer, app.config.semantics.clone()){
                            Err(_) => return Err(String::from("no matching regex")),
                            Ok(new_selections) => {
                                app.selections = new_selections;
                                app.checked_scroll_and_update(
                                    &app.selections.primary.clone(), 
                                    Application::update_ui_data_document, 
                                    Application::update_ui_data_selections
                                );
                            }
                        }
                    }
                }
            }
            "split" => {    //we may need to take certain regexes in quotes. i would assume the same applies to search
                //split <regex>
                let regex = match command_words.next(){
                    None => return Err(String::from("too few args: split <regex>")),
                    Some(word) => {
                        match resolve_potential_expansion(app, word){
                            Err(error) => return Err(error),
                            Ok(regex) => regex
                        }
                    }
                };
                match command_words.next(){
                    Some(_) => return Err(String::from("too many args: split <regex>")),
                    None => {
                        //match crate::utilities::incremental_split_in_selection::selections_impl(&app.selections, &regex, &app.buffer, app.config.semantics.clone()){
                        match selections::incremental_split_in_selection(&app.selections, &regex, &app.buffer, app.config.semantics.clone()){
                            Err(_) => return Err(String::from("no matching regex")),
                            Ok(new_selections) => {
                                app.selections = new_selections;
                                app.checked_scroll_and_update(
                                    &app.selections.primary.clone(), 
                                    Application::update_ui_data_document, 
                                    Application::update_ui_data_selections
                                );
                            }
                        }
                    }
                }
            }
                
            //user defined commands may need to be quoted "if spaces are used"...
            //"\"idk some shit\"" => handle_message(app, DisplayMode::Error, "idk some shit"),  //commands with whitespace can be handled this way
                
            "add_command" => {  //TODO: figure out how to handle command aliases...
                //add_command <command_name> <command> [optional_doc_string]
                let command_name = match command_words.next(){
                    None => return Err(String::from("too few args: add_command <command_name> <command> [optional_documentation]")),
                    Some(word) => {
                        match resolve_potential_expansion(app, word){
                            Err(error) => return Err(error),
                            Ok(command_name) => command_name
                        }
                    }
                };
                let command = match command_words.next(){
                    None => return Err(String::from("too few args: add_command <command_name> <command> [optional_documentation]")),
                    Some(word) => {
                        match resolve_potential_expansion(app, word){
                            Err(error) => return Err(error),
                            Ok(command) => command
                        }
                    }
                };
                let optional_documentation = match command_words.next(){
                    None => None,
                    Some(word) => {
                        match resolve_potential_expansion(app, word){
                            Err(error) => return Err(error),
                            Ok(documentation) => Some(documentation)
                        }
                    }
                };
                match command_words.next(){
                    Some(_) => return Err(String::from("too many args: add_command <command_name> <command> [optional_documentation]")),
                    None => {
                        match command_name.as_str(){
                            "evaluate_commands" |
                            "echo" |
                            //"term" | "t" |
                            //"toggle_line_numbers" | "ln" |
                            //"toggle_status_bar" | "sb" |
                            "quit" | "q" |
                            "quit!" | "q!" |
                            "write" | "w" |
                            "search" |
                            "split" |
                            "add_command" |
                            "remove_command" |
                            "add_option" |
                            "remove_option" |
                            "set_option" |
                            "no_op" => return Err(format!("{:?} already defined in built in commands", &command_name)),
                            _ => {
                                match app.config.user_commands.contains_key(&command_name){
                                    true => return Err(format!("{:?} already defined in user commands", &command_name)),
                                    false => {
                                        app.config.user_commands.insert(
                                            command_name.clone(), 
                                            Command{
                                                aliases: Vec::new(),
                                                documentation: optional_documentation,
                                                command_body: match parse_command(command){
                                                    Err(error) => return Err(error),
                                                    Ok(command) => command
                                                }
                                            }
                                        );
                                        handle_message(app, DisplayMode::Notify, &format!("{} added to commands", &command_name));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            "remove_command" => {
                //remove_command <command_name>
                let command_name = match command_words.next(){
                    None => return Err(String::from("too few arguments: remove_command <command_name>")),
                    Some(word) => {
                        match resolve_potential_expansion(app, word){
                            Err(error) => return Err(error),
                            Ok(command_name) => command_name
                        }
                    }
                };
                match command_words.next(){
                    Some(_) => return Err(String::from("too many args: remove_command <command_name>")),
                    None => {
                        match app.config.user_commands.remove(&command_name){
                            None => return Err(format!("{} does not exist in user commands", &command_name)),
                            Some(_) => handle_message(app, DisplayMode::Notify, &format!("{} removed from user commands", &command_name)),
                        }
                    }
                }
            }
                
            //add_keybind <mode> <keybind> <command>
            //"add_keybind" => {
            //    let mode = Mode::Insert;    //get mode from positional args
            //    let keycode = crossterm::event::KeyCode::Char('n'); //get mode from positional args
            //    let modifiers = crossterm::event::KeyModifiers::CONTROL;    //get mode from positional args
            //    let key_event = crossterm::event::KeyEvent::new(keycode, modifiers);
            //    let _command = "idk some shit".to_string();  //get mode from positional args
            //    if app.config.keybinds.contains_key(&(mode, key_event)){
            //        return Err(String::from("this keybind has already been mapped"))
            //    }else{
            //        //app.config.keybinds.insert((mode, key_event), Action::EditorAction(EditorAction::EvalCommand(command)));
            //        handle_message(app, DisplayMode::Info, "keybind added");
            //    }
            //}
            //remove_keybind <keybind>
                
            "add_option" => {   //TODO: ensure user does not add option with same name as built in options
                //add_option <name> <option_type> [initial_value]
                let name = match command_words.next(){
                    None => return Err(String::from("too few arguments: add_option <name> <option_type> [initial_value]")),
                    Some(word) => {
                        match resolve_potential_expansion(app, word){
                            Err(error) => return Err(error),
                            Ok(name) => name
                        }
                    }
                };
                let option_type = match command_words.next(){
                    None => return Err(String::from("too few arguments: add_option <name> <option_type> [initial_value]")),
                    Some(word) => {
                        match resolve_potential_expansion(app, word){
                            Err(error) => return Err(error),
                            Ok(option_type) => option_type,
                        }
                    }
                };
                let maybe_initial_value = match command_words.next(){
                    None => None,
                    Some(word) => {
                        match resolve_potential_expansion(app, word){
                            Err(error) => return Err(error),
                            Ok(initial_value) => Some(initial_value)
                        }
                    }
                };
                match command_words.next(){
                    Some(_) => return Err(String::from("too many args: add_option <name> <option_type> [initial_value]")),
                    None => {
                        match name.as_ref(){
                            "cursor_semantics" |
                            "use_full_file_path" |
                            "use_hard_tab" |
                            "tab_width" |
                            "view_scroll_amount" |
                            "show_cursor_column" |
                            "show_cursor_line" |
                            "show_line_numbers" |
                            "show_status_bar" => return Err(format!("{} is already a built in option", &name)),
                            _ => {
                                match app.config.user_options.contains_key(&name){
                                    true => return Err(format!("{} user option already exists", &name)),
                                    false => {
                                        app.config.user_options.insert(
                                            name.clone(), 
                                            match option_type.as_str(){
                                                "bool" => {
                                                    OptionType::Bool(
                                                        match maybe_initial_value{
                                                            None => false,
                                                            Some(initial_value) => {
                                                                match initial_value.parse::<bool>(){
                                                                    Err(error) => return Err(format!("{}", error)),
                                                                    Ok(parsed_initial_value) => parsed_initial_value
                                                                }
                                                            }
                                                        }
                                                    )
                                                }
                                                "u8" => {
                                                    OptionType::U8(
                                                        match maybe_initial_value{
                                                            None => 0,
                                                            Some(initial_value) => {
                                                                match initial_value.parse::<u8>(){
                                                                    Err(error) => return Err(format!("{}", error)),
                                                                    Ok(parsed_initial_value) => parsed_initial_value
                                                                }
                                                            }
                                                        }
                                                    )
                                                }
                                                "string" => {
                                                    OptionType::String(
                                                        match maybe_initial_value{
                                                            None => String::new(),
                                                            Some(initial_value) => initial_value
                                                        }
                                                    )
                                                }
                                                _ => return Err(String::from("invalid option type"))
                                            }
                                        );
                                        handle_message(app, DisplayMode::Notify, &format!("{:?} added to user_options", name));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            "remove_option" => {
                //remove_option <name>
                let name = match command_words.next(){
                    None => return Err(String::from("too few args: remove_option <name>")),
                    Some(word) => {
                        match resolve_potential_expansion(app, word){
                            Err(error) => return Err(error),
                            Ok(name) => name
                        }
                    }
                };
                match command_words.next(){
                    Some(_) => return Err(String::from("too many args: remove_option <name>")),
                    None => {
                        match app.config.user_options.contains_key(&name){
                            false => return Err(format!("{} is not a valid user option", &name)),
                            true => {
                                app.config.user_options.remove(&name);
                                handle_message(app, DisplayMode::Notify, &format!("{} removed from user options", &name));
                            }
                        }
                    }
                }
            }
            "set_option" => {
                //set_option <name> <value>
                let name = match command_words.next(){
                    None => return Err(String::from("too few args: set_option <name> <value>")),
                    Some(word) => {
                        match resolve_potential_expansion(app, word){
                            Err(error) => return Err(error),
                            Ok(name) => name
                        }
                    }
                };
                let value = match command_words.next(){
                    None => return Err(String::from("too few args: set_option <name> <value>")),
                    Some(word) => {
                        match resolve_potential_expansion(app, word){
                            Err(error) => return Err(error),
                            Ok(value) => value
                        }
                    }
                };
                match command_words.next(){
                    Some(_) => return Err(String::from("too many args: set_option <name> <value>")),
                    None => {
                        match name.as_ref(){
                            //NOTE: may not allow setting cursor semantics for TUI, because terminal cannot currently handle multicursor bar cursor display...
                            "cursor_semantics" => { //TODO: maybe return error results in same state if already set to provided value. maybe do that for all options...
                                match value.as_str(){
                                    "Bar" | "bar" => {
                                        if app.config.semantics == CursorSemantics::Bar{handle_message(app, SAME_STATE_DISPLAY_MODE, SAME_STATE);}
                                        else{
                                            app.config.semantics = CursorSemantics::Bar;
                                            //TODO: change selections from Block to Bar
                                            handle_message(app, DisplayMode::Notify, &format!("cursor_semantics set to {}", value));
                                        }
                                    }
                                    "Block" | "block" => {
                                        if app.config.semantics == CursorSemantics::Block{handle_message(app, SAME_STATE_DISPLAY_MODE, SAME_STATE);}
                                        else{
                                            app.config.semantics = CursorSemantics::Block;
                                            //TODO: change selections from Bar to Block
                                            handle_message(app, DisplayMode::Notify, &format!("cursor_semantics set to {}", value));
                                        }
                                    }
                                    _ => return Err(format!("{} is not a valid value for cursor_semantics", value))
                                }
                            }
                            "use_full_file_path" => {
                                match value.parse::<bool>(){
                                    Err(error) => return Err(format!("{}", error)),
                                    Ok(parsed_value) => {
                                        app.config.use_full_file_path = parsed_value;
                                        if app.config.use_full_file_path{
                                            app.ui.status_bar.file_name_widget.text = app.buffer.file_path().unwrap_or_default();
                                        }else{
                                            app.ui.status_bar.file_name_widget.text = app.buffer.file_name().unwrap_or_default();
                                        }
                                        handle_message(app, DisplayMode::Notify, &format!("{} set to {}", name, parsed_value));
                                    }
                                }
                            }
                            "use_hard_tab" => {
                                match value.parse::<bool>(){
                                    Err(error) => return Err(format!("{}", error)),
                                    Ok(parsed_value) => {
                                        app.config.use_hard_tab = parsed_value;
                                        handle_message(app, DisplayMode::Notify, &format!("{} set to {}", name, parsed_value));
                                    }
                                }
                            }
                            "tab_width" => {
                                match value.parse::<usize>(){
                                    Err(error) => return Err(format!("{}", error)),
                                    Ok(parsed_value) => {
                                        app.config.tab_width = parsed_value;
                                        handle_message(app, DisplayMode::Notify, &format!("{} set to {}", name, parsed_value));
                                    }
                                }
                            }
                            "view_scroll_amount" => {
                                match value.parse::<usize>(){
                                    Err(error) => return Err(format!("{}", error)),
                                    Ok(parsed_value) => {
                                        app.config.view_scroll_amount = parsed_value;
                                        handle_message(app, DisplayMode::Notify, &format!("{} set to {}", name, parsed_value));
                                    }
                                }
                            }
                            "show_cursor_column" => {
                                match value.parse::<bool>(){
                                    Err(error) => return Err(format!("{}", error)),
                                    Ok(parsed_value) => {
                                        app.config.show_cursor_column = parsed_value;
                                        handle_message(app, DisplayMode::Notify, &format!("{} set to {}", name, parsed_value));
                                    }
                                }
                            }
                            "show_cursor_line" => {
                                match value.parse::<bool>(){
                                    Err(error) => return Err(format!("{}", error)),
                                    Ok(parsed_value) => {
                                        app.config.show_cursor_line = parsed_value;
                                        handle_message(app, DisplayMode::Notify, &format!("{} set to {}", name, parsed_value));
                                    }
                                }
                            }
                            "show_line_numbers" => {
                                match value.parse::<bool>(){
                                    Err(error) => return Err(format!("{}", error)),
                                    Ok(parsed_value) => {
                                        //TODO?: if app.mode() == Mode::Command{app.pop_to_insert()/*although, this fn is scoped within action()...*/}
                                        app.ui.document_viewport.line_number_widget.show = parsed_value;
                                        //
                                        app.update_layouts();
                                        app.update_ui_data_document();
                                        //
                                        handle_message(app, DisplayMode::Notify, &format!("{} set to {}", name, parsed_value));
                                    }
                                }
                            }
                            "show_status_bar" => {
                                match value.parse::<bool>(){
                                    Err(error) => return Err(format!("{}", error)),
                                    Ok(parsed_value) => {
                                        //TODO?: if app.mode() == Mode::Command{app.pop_to_insert()/*although, this fn is scoped within action()...*/}
                                        app.ui.status_bar.show = parsed_value;
                                        //
                                        app.update_layouts();
                                        app.update_ui_data_document();
                                        //
                                        handle_message(app, DisplayMode::Notify, &format!("{} set to {}", name, parsed_value));
                                    }
                                }
                            }
                            _ => {
                                match app.config.user_options.get(&name){
                                    None => return Err(format!("user options does not contain {}", &name)),
                                    Some(option_type) => {
                                        match option_type{
                                            OptionType::Bool(_) => {
                                                let maybe_parsed_value: Result<bool, std::str::ParseBoolError> = value.parse();
                                                match maybe_parsed_value{
                                                    Ok(parsed_value) => {
                                                        app.config.user_options.insert(name.clone(), OptionType::Bool(parsed_value));
                                                        handle_message(app, DisplayMode::Notify, &format!("{} set to {}", name, parsed_value));
                                                    }
                                                    Err(error) => return Err(format!("{}", error))
                                                }
                                            }
                                            OptionType::U8(_) => {
                                                let maybe_parsed_value: Result<u8, std::num::ParseIntError> = value.parse();//word.content.parse();
                                                match maybe_parsed_value{
                                                    Ok(parsed_value) => {
                                                        app.config.user_options.insert(name.clone(), OptionType::U8(parsed_value));
                                                        handle_message(app, DisplayMode::Notify, &format!("{} set to {}", name, parsed_value));
                                                    }
                                                    Err(error) => return Err(format!("{}", error))
                                                }
                                            }
                                            OptionType::String(_) => {
                                                app.config.user_options.insert(name.clone(), OptionType::String(value.clone()));
                                                handle_message(app, DisplayMode::Notify, &format!("{} set to {}", name, value));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
                
            "no_op" => {    //this would be used to start some external program or similar. no editor explicit behavior
                //no_op <command>
                let _command = match command_words.next(){
                    None => return Err(String::from("too few args: no_op <command>")),
                    Some(word) => {
                        match resolve_potential_expansion(app, word){
                            Err(error) => return Err(error),
                            Ok(command) => command
                        }
                    }
                };
                match command_words.next(){
                    Some(_) => return Err(String::from("too many args: no_op <command>")),
                    None => {
                        //should we really be displaying anything here?...
                        handle_message(app, DisplayMode::Info, "no op");
                    }
                }
            }
            //add_hook <group_name> <event> <filtering_regex> <response_command>    //maybe set a hook name instead of group?...    //if no group/name provided, only trigger once, then remove
            //remove hook <group_name>
            //TODO: add-selection
            //TODO: set-selection
            //add-highlighter <group_id> [buffer_offset|widget_coords|screen_coords] <value>
                //value = buffer range | widget line/column/cell | screen line/column/cell
                //buffer_offset highlighter could map directly to the buffer, which would convert to widget_coords for render...
            //remove-highlighter <group_id>
            _ => {
                match app.config.user_commands.get(&first){
                    None => return Err(format!("{:?} is not a valid command", first)),
                    Some(command) => {
                        match execute_parsed_commands(app, command.command_body.clone()){
                            Err(error) => return Err(error),
                            Ok(()) => {}
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/*
this could be used for aliasing commands, instead of storing "aliases: Vec<String>"
let mut key_to_id: HashMap<String, usize> = HashMap::new();
let mut id_to_value: HashMap<usize, Command> = HashMap::new();

//command: add_command term 'no_op %sh{alacritty}' 'opens a new alacritty window'
key_to_id.insert(String::from("term"), 0);  //we could generate the id value

id_to_value.insert(
    0, 
    Command{
        documentation: Some(String::from("opens a new alacritty window"), 
        command_body: Vec<Vec<Word{word_type: WordType::Expansion, content: String::from("alacritty")}>>)
    }
);

//command: alias term t
key_to_id.insert(String::from("t"), 0); //this is the alias


how would removing aliased commands work?...
    remove_command <command_name>
    get key_to_id for <command_name>
    remove all keys with value id?
what if we just wanted to remove the alias?...
    maybe store key (command: String, is_alias: bool)       //key_to_id: HashMap<(String, bool), usize>     //could newtype String to CommandName, and bool to IsAlias
    then, removing a command when is_alias == false, remove all keys with that shared id
    and, removing a command when is_alias == true, just remove that one

let idk = user_commands.get(user_command_ids.get("user_command_name"));
let idk = built_in_commands.get(built_in_command_ids.get("built_in_command_name"));
*/
