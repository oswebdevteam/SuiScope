use colored::*;

/// Print a styled section header.
pub fn print_header(text: &str) {
    println!("\n{}", text.bold().cyan());
}

/// Print a success message with a checkmark.
pub fn print_success(text: &str) {
    println!("{} {}", "✔".green(), text);
}

/// Print an error message with a red cross.
pub fn print_error(text: &str) {
    println!("{} {}", "✖".red(), text);
}

/// Print a warning message.
pub fn print_warning(text: &str) {
    println!("{} {}", "⚠".yellow(), text);
}

/// Print informational message.
pub fn print_info(text: &str) {
    println!("{} {}", "ℹ".blue(), text);
}

/// Truncate an object ID for display (e.g., 0xabcd...1234).
pub fn truncate_id(id: &str, len: usize) -> String {
    if id.len() <= len * 2 + 2 {
        return id.to_string();
    }
    
    // Check for "0x" prefix
    let (prefix, rest) = if let Some(stripped) = id.strip_prefix("0x") {
        ("0x", stripped)
    } else {
        ("", id)
    };

    if rest.len() <= len * 2 {
        return id.to_string();
    }

    format!(
        "{}{}...{}",
        prefix,
        &rest[..len],
        &rest[rest.len() - len..]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_id() {
        let id = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        assert_eq!(truncate_id(id, 4), "0x1234...cdef");
    }

    #[test]
    fn test_truncate_id_short() {
        let id = "0x1234";
        assert_eq!(truncate_id(id, 4), "0x1234");
    }
}
