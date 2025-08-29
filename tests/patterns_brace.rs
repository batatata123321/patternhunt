// tests/patterns_brace.rs
use patternhunt::patterns::brace;

#[test]
fn test_brace_expansion() {
    let v = brace::expand("a{b,c}d").unwrap();
    assert!(v.contains(&"abd".to_string()));
    assert!(v.contains(&"acd".to_string()));
}

#[test]
fn test_brace_expansion_multiple() {
    let v = brace::expand("file.{txt,md}").unwrap();
    assert!(v.contains(&"file.txt".to_string()));
    assert!(v.contains(&"file.md".to_string()));
}

#[test]
fn test_brace_expansion_numeric_range() {
    let v = brace::expand("test{1..3}").unwrap();
    assert!(v.contains(&"test1".to_string()));
    assert!(v.contains(&"test2".to_string()));
    assert!(v.contains(&"test3".to_string()));
}

#[test]
fn test_brace_expansion_depth_limit() {
    // Pattern with 11 levels of nesting to exceed MAX_DEPTH = 10
    let result = brace::expand("{a,{b,{c,{d,{e,{f,{g,{h,{i,{j,{k,l}}}}}}}}}}}");
    assert!(result.is_err());
}

#[test]
fn test_brace_expansion_count_limit() {
    let result = brace::expand("{1..10000}");
    assert!(result.is_err());
}
