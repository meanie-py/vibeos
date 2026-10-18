//! Exact byte offsets, compact for ordinary files and wide for inputs over 4 GiB.
pub(crate) enum LineIndex {
    Compact(Box<[u32]>),
    Wide(Box<[usize]>),
}

impl LineIndex {
    pub(crate) fn new(text: &str) -> Self {
        let starts = std::iter::once(0).chain(text.match_indices('\n').map(|(at, _)| at + 1));
        if u32::try_from(text.len()).is_ok() {
            Self::Compact(starts.map(|at| at as u32).collect())
        } else {
            Self::Wide(starts.collect())
        }
    }
    pub(crate) fn get(&self, row: usize) -> Option<usize> {
        match self {
            Self::Compact(lines) => lines.get(row).map(|&at| at as usize),
            Self::Wide(lines) => lines.get(row).copied(),
        }
    }
    pub(crate) fn line(&self, byte: usize) -> usize {
        match self {
            Self::Compact(lines) => lines.partition_point(|&at| at as usize <= byte),
            Self::Wide(lines) => lines.partition_point(|&at| at <= byte),
        }
    }
    pub(crate) fn bytes(&self) -> usize {
        match self {
            Self::Compact(lines) => std::mem::size_of_val(&**lines),
            Self::Wide(lines) => std::mem::size_of_val(&**lines),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compact_and_wide_offsets_match_prefix_counts() {
        for text in ["", "a", "\n", "a\r\nb\r\r\nc\r", "é\n猫\n"] {
            let starts: Vec<_> = std::iter::once(0)
                .chain(text.match_indices('\n').map(|(at, _)| at + 1))
                .collect();
            for index in [LineIndex::new(text), LineIndex::Wide(starts.clone().into())] {
                for byte in 0..=text.len() + 1 {
                    assert_eq!(index.line(byte), starts.partition_point(|&at| at <= byte));
                }
                assert_eq!(index.line(usize::MAX), starts.len());
                for row in 0..=starts.len() {
                    assert_eq!(index.get(row), starts.get(row).copied());
                }
            }
        }
    }
}
