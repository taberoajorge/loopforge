use std::sync::mpsc::{Receiver, TryRecvError, channel};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogStreamSnapshot {
    pub request_key: String,
    pub request_count: usize,
    pub is_loading: bool,
    pub total_lines: usize,
    pub scroll_top_line: usize,
    pub visible_lines: Vec<String>,
}
#[derive(Debug)]
struct LogChunk {
    request_ticket: usize,
    request_key: String,
    lines: Vec<String>,
    is_final: bool,
}
#[derive(Debug)]
pub struct LogStreamState {
    request_key: String,
    request_count: usize,
    is_loading: bool,
    lines: Vec<String>,
    scroll_top_line: usize,
    request_ticket: usize,
    pending_receiver: Option<Receiver<LogChunk>>,
}
impl LogStreamState {
    pub fn new() -> Self {
        Self {
            request_key: String::new(),
            request_count: 0,
            is_loading: false,
            lines: Vec::new(),
            scroll_top_line: 0,
            request_ticket: 0,
            pending_receiver: None,
        }
    }
    pub fn load_fixture_async(&mut self, request_key: String, fixture_text: String, chunk_size: usize) {
        self.request_count += 1;
        self.request_ticket += 1;
        self.request_key = request_key.clone();
        self.is_loading = true;
        self.lines.clear();
        self.scroll_top_line = 0;
        let chunk_size_value = chunk_size.max(1);
        let request_ticket = self.request_ticket;
        let (sender, receiver) = channel();
        std::thread::spawn(move || {
            let parsed_lines = if fixture_text.is_empty() {
                vec!["No log output".to_string()]
            } else {
                fixture_text.lines().map(str::to_string).collect::<Vec<String>>()
            };
            for line_chunk in parsed_lines.chunks(chunk_size_value) {
                let _ = sender.send(LogChunk {
                    request_ticket,
                    request_key: request_key.clone(),
                    lines: line_chunk.to_vec(),
                    is_final: false,
                });
            }
            let _ = sender.send(LogChunk {
                request_ticket,
                request_key,
                lines: Vec::new(),
                is_final: true,
            });
        });
        self.pending_receiver = Some(receiver);
    }
    pub fn poll_background_load(&mut self) -> bool {
        let Some(receiver) = self.pending_receiver.take() else {
            return false;
        };
        match receiver.try_recv() {
            Ok(chunk) => {
                if chunk.request_ticket == self.request_ticket {
                    self.request_key = chunk.request_key;
                    if !chunk.lines.is_empty() {
                        self.lines.extend(chunk.lines);
                    }
                    if chunk.is_final {
                        self.is_loading = false;
                        self.clamp_scroll_top_line();
                    }
                }
                if self.is_loading {
                    self.pending_receiver = Some(receiver);
                }
                true
            }
            Err(TryRecvError::Empty) => {
                self.pending_receiver = Some(receiver);
                false
            }
            Err(TryRecvError::Disconnected) => {
                self.is_loading = false;
                false
            }
        }
    }
    pub fn set_scroll_top_line(&mut self, scroll_top_line: usize) {
        self.scroll_top_line = scroll_top_line;
        self.clamp_scroll_top_line();
    }
    pub fn is_loading(&self) -> bool {
        self.is_loading
    }
    pub fn snapshot(&self, viewport_rows: usize) -> LogStreamSnapshot {
        let rows = viewport_rows.max(1);
        let start = self.scroll_top_line.min(self.lines.len());
        let end = (start + rows).min(self.lines.len());
        LogStreamSnapshot {
            request_key: self.request_key.clone(),
            request_count: self.request_count,
            is_loading: self.is_loading,
            total_lines: self.lines.len(),
            scroll_top_line: self.scroll_top_line,
            visible_lines: self.lines[start..end].to_vec(),
        }
    }
    fn clamp_scroll_top_line(&mut self) {
        if self.lines.is_empty() {
            self.scroll_top_line = 0;
            return;
        }
        self.scroll_top_line = self.scroll_top_line.min(self.lines.len() - 1);
    }
}
