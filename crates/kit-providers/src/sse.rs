//! Server-sent events, parsed from raw bytes.
//!
//! Chunks can split anywhere, including inside a UTF-8 character or a CRLF,
//! so lines are cut from bytes and decoded only once they are whole.

/// One dispatched event.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SseEvent {
    /// The `event:` field, if any.
    pub event: Option<String>,
    /// The `data:` lines, joined with `\n`.
    pub data: String,
}

/// Incremental parser. Feed it chunks with [`push`](Self::push).
#[derive(Debug, Default)]
pub struct SseParser {
    line: Vec<u8>,
    event: Option<String>,
    data: Vec<String>,
}

impl SseParser {
    /// Add bytes; returns every event they completed.
    pub fn push(&mut self, bytes: &[u8]) -> Vec<SseEvent> {
        let mut out = Vec::new();
        for &b in bytes {
            if b == b'\n' {
                if self.line.last() == Some(&b'\r') {
                    self.line.pop();
                }
                let line = String::from_utf8_lossy(&self.line).into_owned();
                self.line.clear();
                if let Some(ev) = self.take_line(&line) {
                    out.push(ev);
                }
            } else {
                self.line.push(b);
            }
        }
        out
    }

    /// The stream ended: dispatch a last event that had no blank line after it.
    pub fn finish(&mut self) -> Vec<SseEvent> {
        let mut out = Vec::new();
        if !self.line.is_empty() {
            let line = String::from_utf8_lossy(&self.line).into_owned();
            self.line.clear();
            if let Some(ev) = self.take_line(&line) {
                out.push(ev);
            }
        }
        if let Some(ev) = self.dispatch() {
            out.push(ev);
        }
        out
    }

    fn take_line(&mut self, line: &str) -> Option<SseEvent> {
        if line.is_empty() {
            return self.dispatch();
        }
        if line.starts_with(':') {
            return None;
        }
        let (field, value) = match line.split_once(':') {
            Some((f, v)) => (f, v.strip_prefix(' ').unwrap_or(v)),
            None => (line, ""),
        };
        match field {
            "event" => self.event = Some(value.to_owned()),
            "data" => self.data.push(value.to_owned()),
            _ => {}
        }
        None
    }

    fn dispatch(&mut self) -> Option<SseEvent> {
        if self.data.is_empty() && self.event.is_none() {
            return None;
        }
        let ev = SseEvent {
            event: self.event.take(),
            data: self.data.join("\n"),
        };
        self.data.clear();
        Some(ev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all(chunks: &[&[u8]]) -> Vec<SseEvent> {
        let mut p = SseParser::default();
        let mut out = Vec::new();
        for c in chunks {
            out.extend(p.push(c));
        }
        out.extend(p.finish());
        out
    }

    #[test]
    fn named_events_and_multi_line_data() {
        let evs = all(&[b"event: ping\ndata: {}\n\ndata: a\ndata: b\n\n"]);
        assert_eq!(evs.len(), 2);
        assert_eq!(evs[0].event.as_deref(), Some("ping"));
        assert_eq!(evs[1].data, "a\nb");
    }

    #[test]
    fn splits_anywhere_including_crlf_and_utf8() {
        let whole = "data: caf\u{e9} \u{2192}\r\n\r\n".as_bytes();
        for cut in 1..whole.len() {
            let evs = all(&[&whole[..cut], &whole[cut..]]);
            assert_eq!(evs.len(), 1, "cut at {cut}");
            assert_eq!(evs[0].data, "caf\u{e9} \u{2192}", "cut at {cut}");
        }
    }

    #[test]
    fn comments_are_ignored_and_a_trailing_event_is_kept() {
        let evs = all(&[b": keep-alive\n\ndata: [DONE]"]);
        assert_eq!(
            evs,
            vec![SseEvent {
                event: None,
                data: "[DONE]".into()
            }]
        );
    }
}
