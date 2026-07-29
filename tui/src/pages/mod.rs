use qitech_framework_common::RuntimeRequestKind;
use ratatui::Frame;
use ratatui::layout::Rect;

mod machines;
pub use machines::MachinesPage;

pub trait Page {
    fn up(&mut self) -> bool;
    fn down(&mut self);
    fn left(&mut self) {}
    fn right(&mut self) {}
    fn can_edit(&self) -> bool {
        false
    }
    fn edit_to_request(&mut self, data: String) -> RuntimeRequestKind;
    fn display(&self, frame: &mut Frame, chunk: Rect);
}
