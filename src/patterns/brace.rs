// patterns/brace.rs
use crate::error::GlobError;

/// Maximum number of expansions to prevent DoS attacks
const MAX_EXPANSIONS: usize = 1000;
/// Maximum nesting depth to prevent stack overflow
const MAX_DEPTH: usize = 10;

/// Expands brace patterns in a string
///
/// This function supports nested braces and numeric ranges (e.g., {1..3}),
/// with protection against excessive expansion and deep recursion.
///
/// # Arguments
///
/// * `input` - Input string containing brace patterns
///
/// # Returns
///
/// `Ok(Vec<String>)` with expanded strings, or `Err(GlobError)` on failure
///
/// # Errors
///
/// Returns `GlobError::BraceExpansionDepth` if maximum depth exceeded
/// Returns `GlobError::BraceExpansionCount` if maximum expansions exceeded
pub fn expand(input: &str) -> Result<Vec<String>, GlobError> {
    /// Inner recursive expansion function with depth tracking
    fn expand_inner(input: &str, depth: usize) -> Result<Vec<String>, GlobError> {
        if depth > MAX_DEPTH {
            return Err(GlobError::BraceExpansionDepth);
        }

        /// Finds the matching brace pair in the input string
        fn find_brace(s: &str) -> Option<(usize, usize)> {
            let mut depth = 0usize;
            let mut start = None;

            for (i, ch) in s.char_indices() {
                if ch == '{' {
                    if depth == 0 {
                        start = Some(i);
                    }
                    depth += 1;
                } else if ch == '}' {
                    if depth == 0 {
                        return None; // Unbalanced closing brace
                    }
                    depth -= 1;
                    if depth == 0 {
                        return start.map(|st| (st, i));
                    }
                }
            }
            None // No complete brace pair found
        }

        // Find the first complete brace pair
        if let Some((st, en)) = find_brace(input) {
            let before = &input[..st];
            let inner = &input[st + 1..en];
            let after = &input[en + 1..];

            let new_depth = depth + 1;
            if new_depth > MAX_DEPTH {
                return Err(GlobError::BraceExpansionDepth);
            }

            // Split inner content by commas, handling nested braces
            let mut items = Vec::new();
            let mut buf = String::new();
            let mut inner_depth = 0usize;

            for ch in inner.chars() {
                if ch == ',' && inner_depth == 0 {
                    items.push(buf.clone());
                    buf.clear();
                } else {
                    if ch == '{' {
                        inner_depth += 1;
                    } else if ch == '}' {
                        inner_depth = inner_depth.saturating_sub(1);
                    }
                    buf.push(ch);
                }
            }

            if !buf.is_empty() {
                items.push(buf);
            }

            // Handle numeric ranges (e.g., {1..3})
            let mut expanded_items = Vec::new();
            for it in items {
                if let Some((a, b)) = parse_range(&it) {
                    for v in a..=b {
                        expanded_items.push(v.to_string());
                    }
                } else {
                    expanded_items.push(it);
                }
            }

            // Recursively expand each alternative
            let mut out = Vec::new();
            for it in expanded_items {
                for mid in expand_inner(&it, depth + 1)? {
                    for suf in expand_inner(after, depth + 1)? {
                        out.push(format!("{}{}{}", before, mid, suf));
                        if out.len() > MAX_EXPANSIONS {
                            return Err(GlobError::BraceExpansionCount);
                        }
                    }
                }
            }

            Ok(out)
        } else {
            // No braces found, return input as single item
            Ok(vec![input.to_string()])
        }
    }

    expand_inner(input, 0)
}

/// Parses a numeric range string (e.g., "1..3")
///
/// # Arguments
///
/// * `s` - String to parse as a range
///
/// # Returns
///
/// `Some((start, end))` if successful, `None` otherwise
fn parse_range(s: &str) -> Option<(i64, i64)> {
    let parts: Vec<&str> = s.split("..").collect();
    if parts.len() == 2 {
        if let (Ok(a), Ok(b)) = (parts[0].parse::<i64>(), parts[1].parse::<i64>()) {
            return Some((a, b));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brace_expansion() {
        assert_eq!(
            expand("file.{txt,md}").unwrap(),
            vec!["file.txt", "file.md"]
        );
        assert_eq!(
            expand("test{1..3}").unwrap(),
            vec!["test1", "test2", "test3"]
        );
        assert_eq!(expand("a{b,c}d").unwrap(), vec!["abd", "acd"]);
    }

    #[test]
    fn test_brace_expansion_depth() {
        let result = expand("{a,b{1,2}}");
        assert!(result.is_ok());
    }

    #[test]
    fn test_brace_expansion_too_deep() {
        // 11 levels deep (exceeds MAX_DEPTH = 10)
        let deep = "{{{{{{{{{{{a,b}}}}}}}}}}}";
        let result = expand(deep);
        assert!(matches!(result, Err(GlobError::BraceExpansionDepth)));
    }
}
