use std::vec;

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;

use crate::types::AppAction;
use crate::types::AppContext;
use crate::types::KeyResult;
use crate::widgets::MachinesPage;
use crate::widgets::StatusDisplay;
use crate::widgets::TabView;
use crate::widgets::tab_view::TabEntry;
use crate::widgets::transactions::TransactionsPage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Status,
    Content,
}

pub struct UIRoot {
    status: StatusDisplay,
    pages: TabView<AppContext>,
    focus: Focus,
}

impl UIRoot {
    pub fn new() -> Self {
        Self {
            focus: Focus::Status,
            status: StatusDisplay,
            pages: TabView::new(
                false,
                vec![
                    TabEntry {
                        title: "Machines",
                        item: Box::new(MachinesPage::new()),
                    },
                    TabEntry {
                        title: "Transactions",
                        item: Box::new(TransactionsPage::new()),
                    },
                    // TabEntry {
                    //     title: "EtherCAT",
                    //     item: Box::new(MachinesView::new()),
                    // },
                ],
            ),
        }
    }

    pub fn on_key(&mut self, event: KeyEvent, ctx: AppContext) -> Result<AppAction, KeyEvent> {
        match self.focus {
            Focus::Status => match event.code {
                KeyCode::Down => {
                    self.focus = Focus::Content;
                    Ok(AppAction::NoAction)
                }
                _ => Err(event),
            },

            Focus::Content => match self.pages.on_key(event.code, ctx) {
                KeyResult::Handled(v) => Ok(v),
                KeyResult::Bubble(_) => match event.code {
                    KeyCode::Up => {
                        self.focus = Focus::Status;
                        Ok(AppAction::NoAction)
                    }
                    _ => Err(event),
                },
            },
        }
    }

    pub fn render(&mut self, frame: &mut Frame, ctx: AppContext) {
        const TITLE: &str = " QiTech Control (Terminal Edition) ";

        let outer = Block::default().borders(Borders::ALL).title(TITLE);
        frame.render_widget(&outer, frame.area());

        let inner = outer.inner(frame.area());
        let chunks = Layout::vertical([Constraint::Length(4), Constraint::Min(0)]).split(inner);

        // --- render components ---
        self.status
            .render(frame, chunks[0], self.focus == Focus::Status, ctx);

        self.pages
            .render(frame, chunks[1], self.focus == Focus::Content, ctx);
    }
}
