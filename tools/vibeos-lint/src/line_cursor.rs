//! Constant-space line numbers for ordered matches in transformed source text.
pub(crate) struct LineCursor<'a> {
    bytes: &'a [u8],
    offset: usize,
    line: usize,
}

impl<'a> LineCursor<'a> {
    pub(crate) fn new(text: &'a str) -> Self {
        Self {
            bytes: text.as_bytes(),
            offset: 0,
            line: 1,
        }
    }

    pub(crate) fn line(&mut self, offset: usize) -> usize {
        // A caller can restart a scan without constructing a second cursor.
        if offset < self.offset {
            self.offset = 0;
            self.line = 1;
        }
        self.line += self.bytes[self.offset..offset]
            .iter()
            .filter(|&&b| b == b'\n')
            .count();
        self.offset = offset;
        self.line
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_prefix_counts_for_ordered_repeated_and_restarted_offsets() {
        for text in ["", "\n", "α\r\n猫\n\nlast\0", "a\rb"] {
            let offsets: Vec<_> = (0..=text.len())
                .filter(|&at| text.is_char_boundary(at))
                .collect();
            let mut cursor = LineCursor::new(text);
            for &at in offsets
                .iter()
                .chain(offsets.iter())
                .chain(offsets.iter().rev())
            {
                assert_eq!(
                    cursor.line(at),
                    text[..at].bytes().filter(|&b| b == b'\n').count() + 1
                );
            }
        }
    }
}
