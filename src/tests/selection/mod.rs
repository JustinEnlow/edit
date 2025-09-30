mod new_from_range;
mod to_string;
mod debug_over_buffer_content;
mod spans_multiple_lines;
mod merge_overlapping;
mod shift_and_extend;
mod selection_to_selection2d;


//this should hopefully help with generalizing tests to avoid testing specific implementation details...
//TODO: test using ".." and "..=" range syntax
//TODO: test using buffer debug string with '|', '<', '>', ':' selection indicators
