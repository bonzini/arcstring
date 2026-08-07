#[cfg(feature = "serde")]
#[test]
fn test_serde() {
	let s = ArcString::from("a long string, boxed");
	assert_eq!(serde_json::to_string(&s).unwrap(), "\"a long string, boxed\"");
	assert_eq!(serde_json::from_str::<ArcString>("\"sso\"").unwrap(), "sso");
	assert_eq!(serde_json::from_str::<ArcString>("\"テスト・テキスト\"").unwrap(), "テスト・テキスト");
	assert_eq!(serde_json::to_string(&ArcString::empty()).unwrap(), "\"\"");
	assert!(serde_json::from_str::<ArcString>("42").is_err());
}
