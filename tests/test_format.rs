use arcstring::{ArcString, format as format_arcstring};

#[test]
fn test_format_sso() {
	let s = format_arcstring!("{}-{}", 12, 34);
	assert_eq!(s.as_str(), "12-34");
	assert!(ArcString::can_be_sso(s.as_str()));
}

#[test]
fn test_format_empty() {
	assert_eq!(format_arcstring!("").as_str(), "");
}

#[test]
fn test_format_literal() {
	assert_eq!(format_arcstring!("no arguments here at all").as_str(), "no arguments here at all");
}

#[test]
fn test_format_long() {
	let name = "a string that is definitely too long to be stored inline";
	let s = format_arcstring!("{name} ({} bytes)", name.len());
	assert_eq!(s.as_str(), format!("{name} ({} bytes)", name.len()));
}

#[test]
fn test_format_named_and_positional() {
	let x = 5;
	let s = format_arcstring!("{x:03} {0} {y}", "pos", y = true);
	assert_eq!(s.as_str(), "005 pos true");
}
