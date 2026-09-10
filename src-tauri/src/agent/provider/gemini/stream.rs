//! Separates Google thought summaries; signatures never enter this text parser.
#[derive(Default)]
pub struct ThoughtText {
    pending: String,
    thinking: bool,
    started: bool,
}

impl ThoughtText {
    pub fn push(&mut self, text: &str, thought: bool) -> (String, String) {
        if !self.started && thought {
            self.thinking = true;
        }
        if !text.is_empty() {
            self.started = true;
        }
        // Plain answer text is not markup. In particular, literal <thought> code is preserved.
        if !thought && !self.thinking && self.pending.is_empty() {
            return (String::new(), text.to_string());
        }
        self.pending.push_str(text);
        self.drain(false)
    }

    pub fn finish(&mut self) -> (String, String) {
        self.drain(true)
    }

    fn drain(&mut self, eof: bool) -> (String, String) {
        let mut reasoning = String::new();
        let mut content = String::new();
        while !self.pending.is_empty() {
            if self.pending.starts_with("<thought>") {
                self.pending.drain(..9);
                self.thinking = true;
            } else if self.pending.starts_with("</thought>") {
                self.pending.drain(..10);
                self.thinking = false;
                // The remainder of this chunk is an answer, even if it quotes thought tags.
                content.push_str(&self.pending);
                self.pending.clear();
                break;
            } else if !eof
                && ["<thought>", "</thought>"]
                    .iter()
                    .any(|tag| tag.starts_with(&self.pending))
            {
                break;
            } else {
                let end = self
                    .pending
                    .find('<')
                    .filter(|i| *i > 0)
                    .unwrap_or_else(|| {
                        if self.pending.starts_with('<') {
                            1
                        } else {
                            self.pending.len()
                        }
                    });
                if self.thinking {
                    reasoning.push_str(&self.pending[..end]);
                } else {
                    content.push_str(&self.pending[..end]);
                }
                self.pending.drain(..end);
            }
        }
        (reasoning, content)
    }
}
