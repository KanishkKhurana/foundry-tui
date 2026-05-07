pub(crate) fn text_char_len(text: &str) -> usize {
    text.chars().count()
}

pub(crate) fn max_horizontal_offset(text: &str, visible_chars: usize) -> usize {
    text_char_len(text).saturating_sub(visible_chars)
}

pub(crate) fn horizontal_slice(text: &str, offset: usize, visible_chars: usize) -> String {
    if visible_chars == 0 {
        return String::new();
    }

    text.chars().skip(offset).take(visible_chars).collect()
}

pub(crate) fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    if max_chars == 0 {
        return vec![String::new()];
    }

    let mut chunks = Vec::new();
    let mut iter = text.chars();
    loop {
        let chunk = iter.by_ref().take(max_chars).collect::<String>();
        if chunk.is_empty() {
            break;
        }
        chunks.push(chunk);
    }

    if chunks.is_empty() {
        chunks.push(String::new());
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::{horizontal_slice, max_horizontal_offset, wrap_text};

    #[test]
    fn horizontal_slice_respects_offset_and_width() {
        let value = "0x0123456789abcdef";
        assert_eq!(horizontal_slice(value, 0, 6), "0x0123".to_string());
        assert_eq!(horizontal_slice(value, 4, 6), "234567".to_string());
        assert_eq!(horizontal_slice(value, 99, 6), "".to_string());
    }

    #[test]
    fn max_horizontal_offset_is_clamped() {
        let value = "0x0123456789";
        assert_eq!(max_horizontal_offset(value, 5), 7);
        assert_eq!(max_horizontal_offset(value, 32), 0);
    }

    #[test]
    fn wrap_text_splits_long_values() {
        let wrapped = wrap_text("abcdef012345", 4);
        assert_eq!(
            wrapped,
            vec!["abcd".to_string(), "ef01".to_string(), "2345".to_string()]
        );
    }

    #[test]
    fn wrap_text_handles_empty_or_zero_width() {
        assert_eq!(wrap_text("", 8), vec!["".to_string()]);
        assert_eq!(wrap_text("abc", 0), vec!["".to_string()]);
    }
}
