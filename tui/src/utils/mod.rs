mod vertical_cursor;

pub struct VerticalCursor {
    position: usize,
}

impl VerticalCursor {
    pub fn new() -> Self {
        Self { position: 0 }
    }

    pub fn up(&mut self) -> Result<(), CursorMoveError> {
        if self.position == 0 {
            return Err(CursorMoveError);
        }

        self.position = self.position - 1;
        return Ok(());
    }

    pub fn down(&mut self, limit: usize) -> Result<(), CursorMoveError> {
        if self.position + 1 > limit {
            return Err(CursorMoveError);
        }

        self.position = self.position + 1;
        Ok(())
    }

    pub fn position(&self) -> usize {
        self.position
    }
}

pub struct CursorMoveError;

pub enum Focus {
    Root,
    Child,
}
