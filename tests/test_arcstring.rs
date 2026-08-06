use std::fmt::Display;

use arcstring::{ArcString, ArcStringBuilder};

struct FormattableThing(String);

impl Display for FormattableThing {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

#[test]
fn test_arcstring() {
	const _: ArcString = ArcString::try_new_sso("sso").unwrap();

	assert_eq!(ArcString::from("ヤミからこんにちは！").as_str(), "ヤミからこんにちは！");
	assert_ne!(ArcString::from("闇からこんにちは！").as_str(), "ヤミからこんにちは！");
	assert_eq!(ArcString::empty().as_str(), "");
	assert_eq!(ArcString::from("").as_str(), "");
	assert_eq!(ArcString::from("1").as_str(), "1");
	assert_eq!(ArcString::from("二").as_str(), "二");
	assert_eq!(ArcString::from("333").as_str(), "333");
	assert_eq!(ArcString::from("4444").as_str(), "4444");
	assert_eq!(ArcString::from("55555").as_str(), "55555");
	assert_eq!(ArcString::from("666666").as_str(), "666666");
	assert_eq!(ArcString::from("7777777").as_str(), "7777777");
	assert_eq!(ArcString::from("88888888").as_str(), "88888888");
	assert_ne!(ArcString::from("999999999").as_str(), "88888888");
	assert_eq!(ArcString::from("999999999").as_str(), "999999999");
	assert_ne!(ArcString::from("XXXXXXXXXX").as_str(), "999999999");
	assert_eq!(ArcString::from("XXXXXXXXXX").as_str(), "XXXXXXXXXX");
	assert_ne!(ArcString::from("XXXXXXXXXXX").as_str(), "XXXXXXXXXX");
	assert_eq!(ArcString::from("XXXXXXXXXXX").as_str(), "XXXXXXXXXXX");
	assert_ne!(ArcString::from("XXXXXXXXXXXX").as_str(), "XXXXXXXXXXX");
	assert_eq!(ArcString::from("XXXXXXXXXXXX").as_str(), "XXXXXXXXXXXX");
	assert_ne!(ArcString::from("XXXXXXXXXXXXX").as_str(), "XXXXXXXXXXXX");
	assert_eq!(ArcString::from("XXXXXXXXXXXXX").as_str(), "XXXXXXXXXXXXX");
	assert_ne!(ArcString::from("XXXXXXXXXXXXXX").as_str(), "XXXXXXXXXXXXX");
	assert_eq!(ArcString::from("XXXXXXXXXXXXXX").as_str(), "XXXXXXXXXXXXXX");
	assert_ne!(ArcString::from("XXXXXXXXXXXXXXX").as_str(), "XXXXXXXXXXXXXX");
	assert_eq!(ArcString::from("XXXXXXXXXXXXXXX").as_str(), "XXXXXXXXXXXXXXX");
	assert_ne!(ArcString::from("XXXXXXXXXXXXXXXX").as_str(), "XXXXXXXXXXXXXXX");
	assert_eq!(ArcString::from("XXXXXXXXXXXXXXXX").as_str(), "XXXXXXXXXXXXXXXX");
	assert_ne!(ArcString::from("XXXXXXXXXXXXXXXXX").as_str(), "XXXXXXXXXXXXXXXX");
	assert_eq!(ArcString::from("XXXXXXXXXXXXXXXXX").as_str(), "XXXXXXXXXXXXXXXXX");
	assert_ne!(ArcString::from("XXXXXXXXXXXXXXXXXX").as_str(), "XXXXXXXXXXXXXXXXX");
	assert_eq!(ArcString::from("XXXXXXXXXXXXXXXXXX").as_str(), "XXXXXXXXXXXXXXXXXX");
	assert_eq!(ArcString::from("\0\0").as_str(), "\0\0");
	#[cfg(target_pointer_width = "32")]
	assert!(ArcString::from("\0\0\0\0").is_boxed());
	assert_eq!(ArcString::from("\0\0\0\0").as_str(), "\0\0\0\0");
	assert_eq!(ArcString::from("\0\0\0\0\0\0\0").as_str(), "\0\0\0\0\0\0\0");
	assert!(ArcString::from("\0\0\0\0\0\0\0\0").is_boxed());
	assert_eq!(ArcString::from("\0\0\0\0\0\0\0\0").as_str(), "\0\0\0\0\0\0\0\0");
	assert_eq!(ArcString::from("\0\0\0\0\0\0\0\0\0").as_str(), "\0\0\0\0\0\0\0\0\0");
	assert_eq!(ArcString::from_iter(["\0", "\0"].into_iter()).as_str(), "\0\0");
	assert_eq!(ArcString::from_iter(["\0\0", "\0\0"].into_iter()).as_str(), "\0\0\0\0");
	assert_eq!(ArcString::from_iter(["\0\0\0\0", "\0\0\0"].into_iter()).as_str(), "\0\0\0\0\0\0\0");
	assert_eq!(ArcString::from_iter(["\0\0\0\0", "\0\0\0\0"].into_iter()).as_str(), "\0\0\0\0\0\0\0\0");
	assert_eq!(ArcString::from_iter(["\0\0\0\0", "\0\0\0\0\0"].into_iter()).as_str(), "\0\0\0\0\0\0\0\0\0");
	assert_eq!(ArcString::from_iter(["1234", "5678"].into_iter()).as_str(), "12345678");
	assert_eq!(ArcString::from_iter(["123", "45678"].into_iter()).as_str(), "12345678");
	assert_eq!(ArcString::from_iter(["12345", "678"].into_iter()).as_str(), "12345678");
	assert_eq!(ArcString::from_iter(["12", "34"].into_iter()).as_str(), "1234");
	assert_eq!(ArcString::from_iter(["1", "234"].into_iter()).as_str(), "1234");
	assert_eq!(ArcString::from_iter(["123", "4"].into_iter()).as_str(), "1234");
	assert_eq!(ArcString::from_iter(["1", "2"].into_iter()).as_str(), "12");
	assert_eq!(ArcString::from_iter(["", "12"].into_iter()).as_str(), "12");
	assert_eq!(ArcString::from_iter(["12", ""].into_iter()).as_str(), "12");
	assert_eq!(ArcString::from_iter(["12", "3"].into_iter()).as_str(), "123");
	assert_eq!(ArcString::from_iter(["テスト・", "テキ"].into_iter()).as_str(), "テスト・テキ");
	assert_eq!(ArcString::from_iter(["テスト・", "テキ", "スト"].into_iter()).as_str(), "テスト・テキスト");
	let s1 = ArcString::from("テスト・テキスト");
	let s2 = ArcString::from("テスト・テキスト");
	let s3 = ArcString::from("テスト・テキスト");
	assert_eq!(s1, s2);
	assert_eq!(s2, s3);
	assert_eq!(ArcString::from("テスト"), ArcString::from("テスト"));
	assert_eq!(ArcString::from_display(FormattableThing("abc".into())).as_str(), "abc");
	assert_eq!(ArcString::from_display(FormattableThing("abcdefghijklmnopqrstuvwxyz".into())).as_str(), "abcdefghijklmnopqrstuvwxyz");

	assert_eq!(ArcString::from("テスト"), ArcStringBuilder::from("テスト"));
}
