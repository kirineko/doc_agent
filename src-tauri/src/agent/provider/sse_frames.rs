//! Byte-oriented SSE framing shared by Chat Completions providers.
use super::sse::SseError;

#[derive(Default)]
pub struct Frames {
    pending: Vec<u8>,
}

impl Frames {
    pub fn push(&mut self, bytes: &[u8]) -> Result<Vec<String>, SseError> {
        self.pending.extend_from_slice(bytes);
        let mut frames = Vec::new();
        loop {
            let split = self.pending.iter().enumerate().find_map(|(i, b)| {
                if *b != b'\n' {
                    return None;
                }
                if self.pending.get(i + 1) == Some(&b'\n') {
                    return Some((i, i + 2));
                }
                if self.pending.get(i + 1..i + 3) == Some(b"\r\n") {
                    return Some((i, i + 3));
                }
                None
            });
            let Some((end, consumed)) = split else { break };
            let raw = std::str::from_utf8(&self.pending[..end])
                .map_err(|_| SseError::Json("SSE frame is not valid UTF-8".into()))?;
            let data: Vec<_> = raw
                .lines()
                .filter_map(|line| {
                    line.strip_prefix("data:")
                        .map(|s| s.strip_prefix(' ').unwrap_or(s))
                })
                .collect();
            if !data.is_empty() {
                frames.push(data.join("\n"));
            }
            self.pending.drain(..consumed);
        }
        Ok(frames)
    }

    pub fn finish(&self) -> Result<(), SseError> {
        if self.pending.iter().any(|b| !b.is_ascii_whitespace()) {
            return Err(SseError::Json("SSE ended inside a frame".into()));
        }
        Ok(())
    }
}
