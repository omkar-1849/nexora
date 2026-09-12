use crate::filesystem::VirtualPath;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Grid,
    List,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavDirection {
    Forward,
    Backward,
    None,
}

pub struct ExplorerState {
    pub current_path: VirtualPath,
    pub selected: Option<String>,
    pub show_details: bool,
    pub history: Vec<VirtualPath>,
    pub history_index: usize,
    pub search_query: String,
    pub is_editing_path: bool,
    pub path_edit_buffer: String,
    pub view_mode: ViewMode,
    pub nav_direction: NavDirection,
    pub focus_search_requested: bool,
    pub status_message: Option<String>,
}

impl ExplorerState {
    pub fn new() -> Self {
        let initial_path = VirtualPath::parse("/workspace").unwrap_or_else(|_| VirtualPath::root());
        Self {
            current_path: initial_path.clone(),
            selected: None,
            show_details: false,
            history: vec![initial_path],
            history_index: 0,
            search_query: String::new(),
            is_editing_path: false,
            path_edit_buffer: String::new(),
            view_mode: ViewMode::Grid,
            nav_direction: NavDirection::None,
            focus_search_requested: false,
            status_message: None,
        }
    }

    pub fn navigate_to(&mut self, path: VirtualPath) {
        if self.current_path == path {
            return;
        }

        // Truncate forward history if we are in the middle of history stack
        if self.history_index + 1 < self.history.len() {
            self.history.truncate(self.history_index + 1);
        }

        self.history.push(path.clone());
        self.history_index = self.history.len() - 1;

        self.current_path = path;
        self.selected = None;
        self.search_query.clear();
        self.is_editing_path = false;
        self.nav_direction = NavDirection::Forward;
    }

    pub fn can_go_back(&self) -> bool {
        self.history_index > 0
    }

    pub fn can_go_forward(&self) -> bool {
        self.history_index + 1 < self.history.len()
    }

    pub fn go_back(&mut self) {
        if self.can_go_back() {
            self.history_index -= 1;
            self.current_path = self.history[self.history_index].clone();
            self.selected = None;
            self.search_query.clear();
            self.is_editing_path = false;
            self.nav_direction = NavDirection::Backward;
        }
    }

    pub fn go_forward(&mut self) {
        if self.can_go_forward() {
            self.history_index += 1;
            self.current_path = self.history[self.history_index].clone();
            self.selected = None;
            self.search_query.clear();
            self.is_editing_path = false;
            self.nav_direction = NavDirection::Forward;
        }
    }

    pub fn go_up(&mut self) {
        if let Some(parent) = self.current_path.parent() {
            self.navigate_to(parent);
            self.nav_direction = NavDirection::Backward;
        }
    }

    pub fn select(&mut self, name: String) {
        self.selected = Some(name);
        self.show_details = true;
    }

    pub fn clear_selection(&mut self) {
        self.selected = None;
        self.show_details = false;
    }

    pub fn toggle_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Grid => ViewMode::List,
            ViewMode::List => ViewMode::Grid,
        };
    }
}

impl Default for ExplorerState {
    fn default() -> Self {
        Self::new()
    }
}