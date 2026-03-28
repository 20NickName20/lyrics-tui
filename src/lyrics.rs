use std::{ops::Range, time::Duration};

#[derive(Debug)]
pub struct LyricLine {
    time: Duration,
    text: Range<usize>
}

#[derive(Debug)]
pub struct Lyrics {
    source: String,
    lines: Vec<LyricLine>,
}

fn parse_timestamp(ts: &str) -> Option<Duration> {
    // format: mm:ss.xx
    let mut parts = ts.split(':');
    let minutes = parts.next()?.parse::<u64>().ok()?;
    let sec_part = parts.next()?;

    let mut sec_parts = sec_part.split('.');
    let seconds = sec_parts.next()?.parse::<u64>().ok()?;
    let millis = sec_parts
        .next()
        .and_then(|ms| ms.parse::<u64>().ok())
        .unwrap_or(0);

    Some(Duration::from_millis(
        minutes * 60_000 + seconds * 1000 + millis * 10,
    ))
}

impl Lyrics {
    pub fn from(source: String) -> Lyrics {
        let mut lines = Vec::new();
        let mut current_line_start = 0;
        for raw_line in source.split_inclusive('\n') {
            let line_end = current_line_start + raw_line.len();
            let mut timestamps = Vec::new();
            let mut search_offset = 0;
            while let Some(start_bracket) = raw_line[search_offset..].find('[') {
                let ts_start = search_offset + start_bracket;
                if let Some(end_bracket) = raw_line[ts_start..].find(']') {
                    let ts_end = ts_start + end_bracket;
                    let ts = &raw_line[(ts_start + 1)..ts_end];
                    if let Some(dur) = parse_timestamp(ts) {
                        timestamps.push(dur);
                    }
                    search_offset = ts_end + 1;
                } else {
                    break;
                }
            }

            for c in raw_line[search_offset..].chars() {
                if c == '\n' {
                    break;
                }
                if c.is_whitespace() {
                    search_offset += 1;
                } else {
                    break;
                }
            }
            let text_end = current_line_start + raw_line.trim_end().len();
            let text_start = (current_line_start + search_offset).min(text_end);

            for ts in timestamps {
                lines.push(LyricLine {
                    time: ts,
                    text: text_start..text_end
                });
            }

            current_line_start = line_end;
        }

        lines.sort_by_key(|l| l.time);

        Lyrics { source, lines }
    }

    pub fn get_current_line_number(&self, time: Duration) -> usize {
        match self.lines.binary_search_by_key(&time, |lyric_line| lyric_line.time) {
            Ok(idx) => idx + 1,
            Err(idx) => idx
        }
    }

    pub fn get_line(&self, number: usize) -> &str {
        let Some(idx) = number.checked_sub(1) else {
            return "";
        };
        self.lines.get(idx)
            .map(|lyric_line| {
                &self.source[lyric_line.text.clone()]
            })
            .unwrap_or("")
    }
}
