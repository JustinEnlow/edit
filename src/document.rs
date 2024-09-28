use crate::view::View;
use crate::selection::{Selection, Selections};
use std::fs::{self, File};
use std::error::Error;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use ropey::Rope;
use crate::text_util;

// tab keypress inserts the number of spaces specified in TAB_WIDTH into the focused document
pub const TAB_WIDTH: usize = 4; //should this be language dependant? on-the-fly configurable?
// whether to use full file path or just file name
pub const USE_FULL_FILE_PATH: bool = false;



pub struct Document{
    text: Rope,
    file_path: Option<PathBuf>,
    modified: bool,
    selections: Selections,
    client_view: View,
    //history: History,
    last_saved_text: Rope,
}
impl Document{
    pub fn open(path: PathBuf) -> Result<Self, Box<dyn Error>>{
        let text = Rope::from_reader(BufReader::new(File::open(&path)?))?;
    
        Ok(Self{
            text: text.clone(),
            file_path: Some(path.clone()),
            modified: false,
            selections: Selections::new(vec![Selection::new(0, 0)], 0, &text.clone()),
            client_view: View::default(),
            last_saved_text: text.clone(),
        })
    }
    pub fn new() -> Self{
        Self{
            text: Rope::new(),
            file_path: None,
            modified: false,
            selections: Selections::new(vec![Selection::new(0, 0)], 0, &Rope::new()),
            client_view: View::default(),
            last_saved_text: Rope::new(),
        }
    }
    pub fn with_text(&self, text: Rope) -> Self{
        Self{
            text: text.clone(),
            file_path: self.file_path.clone(),
            modified: self.modified,
            selections: self.selections.clone(),
            client_view: self.client_view.clone(),
            last_saved_text: text.clone(),
        }
    }
    pub fn with_selections(&self, selections: Selections) -> Self{
        Self{
            text: self.text.clone(),
            file_path: self.file_path.clone(),
            modified: self.modified,
            selections,
            client_view: self.client_view.clone(),
            last_saved_text: self.last_saved_text.clone(),
        }
    }
    pub fn file_name(&self) -> Option<String>{
        match &self.file_path{
            Some(path) => {
                if USE_FULL_FILE_PATH{
                    Some(path.to_string_lossy().to_string())
                }else{
                    Some(path.file_name().unwrap().to_string_lossy().to_string())
                }
            }
            None => None
        }
    }
    pub fn len(&self) -> usize{
        self.text.len_lines()
    }
    pub fn selections(&self) -> &Selections{
        &self.selections
    }
    pub fn selections_mut(&mut self) -> &mut Selections{
        &mut self.selections
    }
    pub fn text(&self) -> &Rope{
        &self.text
    }
    pub fn view(&self) -> &View{
        &self.client_view
    }
    pub fn view_mut(&mut self) -> &mut View{
        &mut self.client_view
    }
    pub fn save(&mut self) -> Result<(), Box<dyn Error>>{
        if let Some(path) = &self.file_path{ // does nothing if path is None
            self.text.write_to(BufWriter::new(fs::File::create(path)?))?;
            
            self.modified = false;
            self.last_saved_text = self.text.clone();
        }
        
        Ok(())
    }
    pub fn is_modified(&self) -> bool{
        self.modified
    }

// AUTO-INDENT
//#[test]
//fn auto_indent_works(){
//    assert!(false);
//}
    /// Creates a new line in the document.
    /// #### Invariants:
    /// - TODO
    /// # Example
    /// ```
    /// # use ropey::Rope;
    /// # use edit::document::Document;
    /// 
    /// let mut doc = Document::new().with_text(Rope::from("idk"));
    /// doc.enter();
    /// assert!(doc.text().clone() == Rope::from("\nidk"));
    /// ```
    pub fn enter(&mut self){
        //for cursor in self.cursors.iter_mut(){
        //    Document::enter_at_cursor(cursor, &mut self.lines, &mut self.modified);
        //}

        //TODO: push current state to history?
        self.insert_char('\n');
    }
            // auto indent doesn't work correctly if previous line has only whitespace characters
            // also doesn't auto indent for first line of function bodies, because function declaration
            // is at lower indentation level
            //fn enter_at_cursor(cursor: &mut Cursor, lines: &mut Vec<String>, modified: &mut bool){
            //    *modified = true;
            //    
            //    match lines.get_mut(cursor.head.y){
            //        Some(line) => {
            //            let start_of_line = get_first_non_whitespace_character_index(line);
            //            let mut modified_current_line: String = String::new();
            //            let mut new_line: String = String::new();
            //            for (index, grapheme) in line[..].graphemes(true).enumerate(){
            //                if index < cursor.head.x{
            //                    modified_current_line.push_str(grapheme);
            //                }
            //                else{
            //                    new_line.push_str(grapheme);
            //                }
            //            }
            //            *line = modified_current_line;
            //            lines.insert(cursor.head.y.saturating_add(1), new_line);
            //            Document::move_cursor_right(cursor, &lines);
            //            // auto indent
            //            if start_of_line != 0{
            //                for _ in 0..start_of_line{
            //                    Document::insert_char_at_cursor(' ', cursor, lines, modified);
            //                }
            //            }
            //        }
            //        None => panic!("No line at cursor position. This should be impossible")
            //    }
            //}

    //TODO: edits should be handled in reverse_selection_order. this ensures that edits at selection position
    //in rope do not effect subsequent selection positions
    // e.g. "ab[]cd[]efg" insert char x
        //if not reversed, would result in "abx[]c[]xdefg" because second selection position remains at position 4 in rope
        // if reversed, would result in "abx[]cdx[]efg" because previous selection positions arent effected by later insertions
    //we also need to ensure selections are sorted by position/index on the rope. and overlapping selections
    //are combined into a single selection

//INSERT SELECTION
//#[test]
//fn single_cursor_insert_single_line_selection_works(){
//    assert!(false);
//}
//#[test]
//fn single_cursor_insert_multi_line_selection_works(){
//    assert!(false);
//}
    pub fn paste(&mut self, _clipboard: &str){}

    /// Inserts specified char, replacing selected text if selection extended.
    /// #### Invariants:
    /// - TODO
    /// # Example
    /// ```
    /// # use ropey::Rope;
    /// # use edit::document::Document;
    /// # use edit::selection::{Selection, Selections};
    /// 
    /// // normal use. selection not extended
    /// let mut doc = Document::new().with_text(Rope::from("idk\nsome\nshit\n"));
    /// doc.insert_char('x');
    /// assert!(doc.text().clone() == Rope::from("xidk\nsome\nshit\n"));
    /// 
    /// // with selection extended
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// let mut doc = Document::new().with_text(text.clone()).with_selections(Selections::new(vec![Selection::new(0, 1)], 0, &text));
    /// doc.insert_char('x');
    /// assert!(doc.text().clone() == Rope::from("xdk\nsome\nshit\n"));
    /// ```
    pub fn insert_char(&mut self, c: char){
        //TODO: if use auto-pairs and inserted char has a mapped auto-pair character, insert that char as well
        for cursor in self.selections.iter_mut().rev(){
            (*cursor, self.text) = Document::insert_char_at_cursor(
                cursor.clone(), 
                &self.text, 
                c
            );
        }

        self.modified = !(self.text == self.last_saved_text);
    }
    fn insert_char_at_cursor(mut selection: Selection, text: &Rope, char: char) -> (Selection, Rope){
        let mut new_text = text.clone();
        if selection.is_extended(){
            (new_text, selection) = Document::delete_at_cursor(selection.clone(), text);
        }
        new_text.insert_char(selection.head(), char);
        selection.move_right(&new_text);

        (selection, new_text)
    }

    /// Inserts [TAB_WIDTH] spaces.
    /// #### Invariants:
    /// - with selection extended, replaces selection
    /// # Example
    /// ```
    /// # use ropey::Rope;
    /// # use edit::document::{Document, TAB_WIDTH};
    /// 
    /// let mut doc = Document::new().with_text(Rope::from("idk\nsome\nshit\n"));
    /// let mut spaces = String::new();
    /// for x in 0..TAB_WIDTH{
    ///     spaces.push(' ');
    /// }
    /// let expected_text = Rope::from(format!("{}idk\nsome\nshit\n", spaces));
    /// doc.tab();
    /// assert!(doc.text().clone() == expected_text);
    /// ```
    pub fn tab(&mut self){
        for selection in self.selections.iter_mut().rev(){
            let tab_distance = text_util::distance_to_next_multiple_of_tab_width(selection.clone(), &self.text);
            let modified_tab_width = if tab_distance > 0 && tab_distance < TAB_WIDTH{
                tab_distance
            }else{
                TAB_WIDTH
            };
            for _ in 0..modified_tab_width{
                (*selection, self.text) = Document::insert_char_at_cursor(
                    selection.clone(), 
                    &self.text, 
                    ' '
                );
            }
        }

        self.modified = !(self.text == self.last_saved_text);
    }

    /// Deletes selection, or if no selection, the next character.
    /// #### Invariants:
    /// - stays within doc bounds
    /// # Example
    /// ```
    /// # use ropey::Rope;
    /// # use edit::document::Document;
    /// # use edit::selection::{Selection, Selections};
    /// 
    /// // will not delete past end of doc
    /// let text = Rope::from("idk");
    /// let mut doc = Document::new().with_text(text.clone()).with_selections(Selections::new(vec![Selection::new(3, 3)], 0, &text));
    /// doc.delete();
    /// assert!(doc.text().clone() == Rope::from("idk"));
    /// 
    /// // no selection
    /// let mut doc = Document::new().with_text(Rope::from("idk\nsome\nshit\n"));
    /// doc.delete();
    /// assert!(doc.text().clone() == Rope::from("dk\nsome\nshit\n"));
    /// 
    /// // with selection head > anchor
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// let mut doc = Document::new().with_text(text.clone()).with_selections(Selections::new(vec![Selection::new(0, 2)], 0, &text));
    /// doc.delete();
    /// assert!(doc.text().clone() == Rope::from("k\nsome\nshit\n"));
    /// 
    /// // with selection head < anchor
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// let mut doc = Document::new().with_text(text.clone()).with_selections(Selections::new(vec![Selection::new(1, 2)], 0, &text));
    /// doc.delete();
    /// assert!(doc.text().clone() == Rope::from("ik\nsome\nshit\n"));
    /// ```
    pub fn delete(&mut self){
        for selection in self.selections.iter_mut().rev(){
            (self.text, *selection) = Document::delete_at_cursor(selection.clone(), &self.text);
        }

        self.modified = !(self.text == self.last_saved_text);
    }
    fn delete_at_cursor(mut selection: Selection, text: &Rope) -> (Rope, Selection){
        let mut new_text = text.clone();

        if selection.head() < text.len_chars(){ //can this be guaranteed by the Selection type? make invalid state impossible?
            if selection.is_extended(){
                if selection.head() < selection.anchor(){
                    new_text.remove(selection.head()..selection.anchor());
                    selection.move_to(selection.head(), text);
                }
                else if selection.head() > selection.anchor(){
                    new_text.remove(selection.anchor()..selection.head());
                    selection.move_to(selection.anchor(), text);
                }
            }else{
                new_text.remove(selection.head()..selection.head()+1);
            }
            //TODO: add ability to delete tabs(repeated spaces) from ahead
        }

        (new_text, selection)
    }

//BACKSPACE
//#[test]
//fn single_cursor_backspace_removes_previous_tab(){
//    let mut doc = Document::default();
//    let mut line = String::new();
//    for _ in 0..TAB_WIDTH{
//        line.push(' ');
//    }
//    line.push_str("something");
//    doc.lines = vec![line];
//
//    let cursor = doc.cursors.get_mut(0).unwrap();
//    let position = Position::new(TAB_WIDTH, 0);
//    *cursor = Document::set_cursor_position(cursor, position, &doc.lines).unwrap();
//    Document::backspace_at_cursor(cursor, &mut doc.lines, &mut doc.modified);
//    println!("{:?}", doc.lines);
//    assert!(doc.lines == vec!["something".to_string()]);
//    println!("{:?}", cursor.head);
//    assert!(cursor.head.x() == 0);
//    assert!(cursor.head.y() == 0);
//    assert!(cursor.anchor.x() == 0);
//    assert!(cursor.anchor.y() == 0);
//}
    /// Deletes the previous character, or deletes selection if extended.
    /// #### Invariants:
    /// - will not delete past start of doc
    /// - at start of line, appends current line to end of previous line
    /// - removes previous soft tab, if TAB_WIDTH spaces are before cursor
    /// - deletes selection if selection extended
    /// # Example
    /// ```
    /// # use ropey::Rope;
    /// # use edit::document::Document;
    /// # use edit::selection::{Selection, Selections};
    /// 
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// 
    /// // without selection deletes previous char
    /// let mut doc = Document::new().with_text(text.clone()).with_selections(Selections::new(vec![Selection::new(0, 1)], 0, &text));
    /// doc.backspace();
    /// assert!(doc.text().clone() == Rope::from("dk\nsome\nshit\n"));
    /// 
    /// // backspace at start of line appends current line to end of previous line
    /// let mut doc = Document::new().with_text(text.clone()).with_selections(Selections::new(vec![Selection::new(4, 4)], 0, &text));
    /// doc.backspace();
    /// assert!(doc.text().clone() == Rope::from("idksome\nshit\n"));
    /// 
    /// // with selection
    /// let mut doc = Document::new().with_text(text.clone()).with_selections(Selections::new(vec![Selection::new(0, 2)], 0, &text));
    /// doc.backspace();
    /// assert!(doc.text().clone() == Rope::from("k\nsome\nshit\n"));
    /// ```
    pub fn backspace(&mut self){
        for selection in self.selections.iter_mut().rev(){
            let cursor_line_position = selection.head() - self.text.line_to_char(self.text.char_to_line(selection.head()));

            let is_deletable_soft_tab = cursor_line_position >= TAB_WIDTH
            // handles case where user adds a space after a tab, and wants to delete only the space
            && cursor_line_position % TAB_WIDTH == 0
            // if previous 4 chars are spaces, delete 4. otherwise, use default behavior
            && text_util::slice_is_all_spaces(
                self.text.line(
                    self.text.char_to_line(selection.head())
                ).as_str().unwrap(),
                cursor_line_position - TAB_WIDTH,
                cursor_line_position
            );
            
            if selection.is_extended(){
                (self.text, *selection) = Document::delete_at_cursor(selection.clone(), &self.text);
            }else{
                if is_deletable_soft_tab{
                    for _ in 0..TAB_WIDTH{
                        selection.move_left(&self.text);
                        (self.text, *selection) = Document::delete_at_cursor(
                            selection.clone(), 
                            &self.text
                        );
                    }
                }
                else if selection.head() > 0{
                    selection.move_left(&self.text);
                    (self.text, *selection) = Document::delete_at_cursor(
                        selection.clone(), 
                        &self.text
                    );
                }
            }
        }

        self.modified = !(self.text == self.last_saved_text);
    }
}
