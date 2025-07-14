use crate::{
    buffer::Buffer
};

#[test] fn works_with_ascii_text(){
    let buffer = Buffer::new("idk\nsome\nshit\n", None, false);
    assert_eq!(3, buffer.line_width_chars(0, false));
    assert_eq!(4, buffer.line_width_chars(0, true));

    assert_eq!(4, buffer.line_width_chars(1, false));
    assert_eq!(5, buffer.line_width_chars(1, true));

    assert_eq!(4, buffer.line_width_chars(2, false));
    assert_eq!(5, buffer.line_width_chars(2, true));

    assert_eq!(0, buffer.line_width_chars(3, false));
    assert_eq!(0, buffer.line_width_chars(3, true));
}

#[ignore] #[test] fn works_with_crlf_newline(){
    todo!()
}

//works with utf8 text
#[ignore] #[test] fn works_with_utf_8(){
    todo!()
}
