use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::style::{Style, Color};
use ratatui::layout::{Alignment, Direction, Layout, Constraint};



#[derive(Default)]
pub struct LineNumberWidget{
    pub rect: Rect,
    pub line_numbers_in_view: String,
}
impl LineNumberWidget{
    pub fn widget(&self) -> Paragraph<'static>{
        Paragraph::new(self.line_numbers_in_view.clone())
            .style(Style::default().fg(Color::Rgb(100, 100, 100)))
            .alignment(Alignment::Right)
    }
}

#[derive(Default, Clone)]
pub struct DocumentWidget{
    pub rect: Rect,
    pub doc_length: usize,  //used in DocumentViewport
    pub text_in_view: String,
}
impl DocumentWidget{
    pub fn widget(&self) -> Paragraph<'static>{
        //let lines: Vec<String> = self.text_in_view.clone().lines().map(|line| line.to_string()).collect();
        Paragraph::new(self.text_in_view.clone())
    }
}

/// Container type for widgets in the document viewport.
pub struct DocumentViewport{
    pub display_line_numbers: bool,
    pub document_widget: DocumentWidget,
    pub line_number_widget: LineNumberWidget,
}
impl Default for DocumentViewport{
    fn default() -> Self{
        Self{
            display_line_numbers: true,
            document_widget: DocumentWidget::default(),
            line_number_widget: LineNumberWidget::default(),
        }
    }
}
impl DocumentViewport{
    pub fn toggle_line_numbers(&mut self){
        self.display_line_numbers = !self.display_line_numbers;
    }
    pub fn layout(&self, rect: Rect) -> std::rc::Rc<[Rect]>{
        // layout of document + line num rect
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                vec![
                    // line number left padding
                    //Constraint::Length(if self.display_line_numbers{1}else{0}),
                    // line number rect width
                    Constraint::Length(
                        if self.display_line_numbers{
                            count_digits(self.document_widget.doc_length)
                        }else{0}
                    ),
                    // line number right padding
                    Constraint::Length(if self.display_line_numbers{1}else{0}),
                    // document rect width
                    Constraint::Min(5)
                ]
            )
            .split(rect)
    }
}

fn count_digits(mut n: usize) -> u16{
    if n == 0{
        return 1;
    }

    let mut count = 0;
    while n > 0{
        count += 1;
        n /= 10;
    }

    count
}
