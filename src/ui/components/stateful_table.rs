use ratatui::widgets::TableState;

/// A stateful table that wraps a vector of items and manages table state
pub struct StatefulTable<T> {
    /// Table items
    pub items: Vec<T>,
    /// Current table state
    pub state: TableState,
    /// Total number of items
    pub total: usize,
}

impl<T> StatefulTable<T> {
    /// Create a new stateful table
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            state: TableState::default(),
            total: 0,
        }
    }

    /// Create a new stateful table with items
    pub fn with_items(items: Vec<T>) -> Self {
        let total = items.len();
        Self {
            items,
            state: TableState::default(),
            total,
        }
    }

    /// Set the table items
    pub fn set_items(&mut self, items: Vec<T>) {
        self.total = items.len();
        self.items = items;
        
        // Ensure selection is still valid
        if let Some(selected) = self.state.selected() {
            if selected >= self.total && self.total > 0 {
                self.state.select(Some(self.total - 1));
            }
        }
    }

    /// Select the next item
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.total.saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    /// Select the previous item
    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.total.saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    
    /// Select a specific item by index
    pub fn select(&mut self, index: usize) {
        if index < self.total {
            self.state.select(Some(index));
        }
    }
    
    /// Unselect the current item
    pub fn unselect(&mut self) {
        self.state.select(None);
    }
    
    /// Get the currently selected item
    pub fn selected_item(&self) -> Option<&T> {
        self.state.selected().and_then(|i| self.items.get(i))
    }
    
    /// Get the currently selected index
    pub fn selected_index(&self) -> Option<usize> {
        self.state.selected()
    }
    
    /// Scroll to the top of the table
    pub fn scroll_to_top(&mut self) {
        if self.total > 0 {
            self.state.select(Some(0));
        } else {
            self.state.select(None);
        }
    }
    
    /// Scroll to the bottom of the table
    pub fn scroll_to_bottom(&mut self) {
        if self.total > 0 {
            self.state.select(Some(self.total - 1));
        } else {
            self.state.select(None);
        }
    }
    
    /// Reset the table state
    pub fn reset(&mut self) {
        self.items.clear();
        self.state.select(None);
        self.total = 0;
    }
}