use ratatui::Frame;
use ratatui::layout::Rect;

mod machines;
pub use machines::MachinesPage;

pub trait Page {
    fn can_go_down(&self) -> bool;

    fn up(&mut self) -> bool;
    fn down(&mut self);
    fn left(&mut self) {}
    fn right(&mut self) {}
    fn display(&self, frame: &mut Frame, chunk: Rect);
}
