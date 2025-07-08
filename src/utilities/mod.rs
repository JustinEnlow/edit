//selection
pub mod move_to_line_number;
pub mod move_cursor_down;
pub mod move_cursor_left;
pub mod move_cursor_right;
pub mod move_cursor_up;
pub mod move_cursor_word_boundary_backward;
pub mod move_cursor_word_boundary_forward;
pub mod move_cursor_home;
pub mod move_cursor_line_end;   //may rename to move_cursor_line_text_end
pub mod move_cursor_line_start;
pub mod move_cursor_line_text_start;
pub mod move_cursor_page_down;
pub mod move_cursor_page_up;
pub mod move_cursor_buffer_end;
pub mod move_cursor_buffer_start;
pub mod extend_selection_down;
pub mod extend_selection_left;
pub mod extend_selection_right;
pub mod extend_selection_up;
pub mod extend_selection_word_boundary_backward;
pub mod extend_selection_word_boundary_forward;
pub mod extend_selection_home;
pub mod extend_selection_line_end;  //may rename to extend_selection_line_text_end
pub mod extend_selection_line_start;
pub mod extend_selection_line_text_start;
pub mod select_all;
pub mod select_line;
pub mod flip_direction;
pub mod collapse_selections_to_anchor;
pub mod collapse_selections_to_cursor;

//selections
pub mod add_selection_above;
pub mod add_selection_below;
pub mod clear_non_primary_selections;
pub mod remove_primary_selection;
pub mod increment_primary_selection;
pub mod decrement_primary_selection;
pub mod surround;
pub mod nearest_surrounding_pair;   //may rename to nearest bracket pair, and have separate nearest quote pair fn

pub mod incremental_search_in_selection;
pub mod incremental_split_in_selection;

//edit
pub mod insert_string;
pub mod delete;
pub mod backspace;
pub mod cut;
pub mod paste;
pub mod undo;
pub mod redo;
pub mod add_surrounding_pair;
    //swap selected text with line above
    //swap selected text with line below
    //align selected text vertically
    //rotate text between selections


//view
pub mod center_view_vertically_around_cursor;
pub mod scroll_view_up;
pub mod scroll_view_down;
pub mod scroll_view_left;
pub mod scroll_view_right;
    //center_view_horizontally_around_cursor
    //align_view_with_cursor_at_top
    //align_view_with_cursor_at_bottom

//other
pub mod copy;
pub mod save;
