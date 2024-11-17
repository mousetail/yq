use std::fmt::{Display, Write};

pub struct Slug<'a>(pub &'a str);

macro_rules! write_previous {
    ($file:ident, $string:ident, $start:ident, $end: expr, $last_is_space: expr) => {
        if $last_is_space {
            $file.write_char('-')?;
        }
        if $end != $start {
            $file.write_str(&$string[$start..$end])?;
        }
    };
}

impl Display for Slug<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let trimmed_value = self.0.trim();

        let mut start = 0;
        let mut iter = trimmed_value.char_indices();
        let mut last_is_space = false;
        while let Some((index, chr)) = iter.next() {
            if chr.is_ascii_uppercase() {
                write_previous!(f, trimmed_value, start, index, last_is_space);
                f.write_char(chr.to_ascii_lowercase())?;
                start = index + chr.len_utf8();
                last_is_space = false;
            }
            if !chr.is_ascii_alphabetic() {
                write_previous!(f, trimmed_value, start, index, last_is_space);
                start = index + chr.len_utf8();
                last_is_space = true;
            }
        }
        if start != trimmed_value.len() {
            write_previous!(f, trimmed_value, start, trimmed_value.len(), last_is_space);
        }
        Ok(())
    }
}
