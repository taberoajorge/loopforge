use super::log_stream::{LogStreamSnapshot, LogStreamState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutputPaneSnapshot {
    pub title: String,
    pub stream: LogStreamSnapshot,
}
#[derive(Debug)]
pub struct OutputPaneState {
    title: String,
    stream: LogStreamState,
}
impl OutputPaneState {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            stream: LogStreamState::new(),
        }
    }
    pub fn load_fixture_async(
        &mut self,
        title: String,
        request_key: String,
        fixture_text: String,
        chunk_size: usize,
    ) {
        self.title = title;
        self.stream
            .load_fixture_async(request_key, fixture_text, chunk_size);
    }
    pub fn poll_background_load(&mut self) -> bool {
        self.stream.poll_background_load()
    }
    pub fn set_scroll_top_line(&mut self, scroll_top_line: usize) {
        self.stream.set_scroll_top_line(scroll_top_line);
    }
    pub fn is_loading(&self) -> bool {
        self.stream.is_loading()
    }
    pub fn snapshot(&self, viewport_rows: usize) -> OutputPaneSnapshot {
        OutputPaneSnapshot {
            title: self.title.clone(),
            stream: self.stream.snapshot(viewport_rows),
        }
    }
}
