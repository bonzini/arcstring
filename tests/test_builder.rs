use std::fmt::Display;

use arcstring::{ArcString, ArcStringBuilder};

struct FormattableThing(String);

impl Display for FormattableThing {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

#[test]
fn test_builder() {
	const _: ArcStringBuilder = ArcStringBuilder::try_new_sso("sso").unwrap();

	assert_eq!(ArcStringBuilder::from_display(FormattableThing("abc".into())).as_str(), "abc");
	assert_eq!(ArcStringBuilder::from_display(FormattableThing("abcdefghijklmnopqrstuvwxyz".into())).as_str(), "abcdefghijklmnopqrstuvwxyz");

	let mut builder = ArcStringBuilder::new();
	assert_eq!(builder.as_str(), "");
	builder.push_str("abc");
	assert_eq!(builder.as_str(), "abc");
	builder.push_str("def");
	assert_eq!(builder.as_str(), "abcdef");
	builder.push_str("ghi");
	assert_eq!(builder.as_str(), "abcdefghi");
	assert_eq!(builder.into_arcstring().as_str(), "abcdefghi");

	assert_eq!(ArcStringBuilder::from("ヤミからこんにちは！").as_str(), "ヤミからこんにちは！");
	assert_ne!(ArcStringBuilder::from("闇からこんにちは！").as_str(), "ヤミからこんにちは！");
	assert_eq!(ArcStringBuilder::new().as_str(), "");
	assert_eq!(ArcStringBuilder::from("").as_str(), "");
	assert_eq!(ArcStringBuilder::from("1").as_str(), "1");
	assert_eq!(ArcStringBuilder::from("二").as_str(), "二");
	assert_eq!(ArcStringBuilder::from("333").as_str(), "333");
	assert_eq!(ArcStringBuilder::from("4444").as_str(), "4444");
	assert_eq!(ArcStringBuilder::from("55555").as_str(), "55555");
	assert_eq!(ArcStringBuilder::from("666666").as_str(), "666666");
	assert_eq!(ArcStringBuilder::from("7777777").as_str(), "7777777");
	assert_eq!(ArcStringBuilder::from("88888888").as_str(), "88888888");
	assert_ne!(ArcStringBuilder::from("999999999").as_str(), "88888888");
	assert_eq!(ArcStringBuilder::from("999999999").as_str(), "999999999");
	assert_ne!(ArcStringBuilder::from("XXXXXXXXXX").as_str(), "999999999");
	assert_eq!(ArcStringBuilder::from("XXXXXXXXXX").as_str(), "XXXXXXXXXX");
	assert_ne!(ArcStringBuilder::from("XXXXXXXXXXX").as_str(), "XXXXXXXXXX");
	assert_eq!(ArcStringBuilder::from("XXXXXXXXXXX").as_str(), "XXXXXXXXXXX");
	assert_ne!(ArcStringBuilder::from("XXXXXXXXXXXX").as_str(), "XXXXXXXXXXX");
	assert_eq!(ArcStringBuilder::from("XXXXXXXXXXXX").as_str(), "XXXXXXXXXXXX");
	assert_ne!(ArcStringBuilder::from("XXXXXXXXXXXXX").as_str(), "XXXXXXXXXXXX");
	assert_eq!(ArcStringBuilder::from("XXXXXXXXXXXXX").as_str(), "XXXXXXXXXXXXX");
	assert_ne!(ArcStringBuilder::from("XXXXXXXXXXXXXX").as_str(), "XXXXXXXXXXXXX");
	assert_eq!(ArcStringBuilder::from("XXXXXXXXXXXXXX").as_str(), "XXXXXXXXXXXXXX");
	assert_ne!(ArcStringBuilder::from("XXXXXXXXXXXXXXX").as_str(), "XXXXXXXXXXXXXX");
	assert_eq!(ArcStringBuilder::from("XXXXXXXXXXXXXXX").as_str(), "XXXXXXXXXXXXXXX");
	assert_ne!(ArcStringBuilder::from("XXXXXXXXXXXXXXXX").as_str(), "XXXXXXXXXXXXXXX");
	assert_eq!(ArcStringBuilder::from("XXXXXXXXXXXXXXXX").as_str(), "XXXXXXXXXXXXXXXX");
	assert_ne!(ArcStringBuilder::from("XXXXXXXXXXXXXXXXX").as_str(), "XXXXXXXXXXXXXXXX");
	assert_eq!(ArcStringBuilder::from("XXXXXXXXXXXXXXXXX").as_str(), "XXXXXXXXXXXXXXXXX");
	assert_ne!(ArcStringBuilder::from("XXXXXXXXXXXXXXXXXX").as_str(), "XXXXXXXXXXXXXXXXX");
	assert_eq!(ArcStringBuilder::from("XXXXXXXXXXXXXXXXXX").as_str(), "XXXXXXXXXXXXXXXXXX");
	assert_eq!(ArcStringBuilder::from("\0\0").as_str(), "\0\0");
	assert_eq!(ArcStringBuilder::from("\0\0\0\0").as_str(), "\0\0\0\0");
	assert_eq!(ArcStringBuilder::from("\0\0\0\0\0\0\0").as_str(), "\0\0\0\0\0\0\0");
	assert_eq!(ArcStringBuilder::from("\0\0\0\0\0\0\0\0").as_str(), "\0\0\0\0\0\0\0\0");
	assert_eq!(ArcStringBuilder::from("\0\0\0\0\0\0\0\0\0").as_str(), "\0\0\0\0\0\0\0\0\0");
	assert_eq!(ArcStringBuilder::from_iter(["\0", "\0"].into_iter()).as_str(), "\0\0");
	assert_eq!(ArcStringBuilder::from_iter(["\0\0", "\0\0"].into_iter()).as_str(), "\0\0\0\0");
	assert_eq!(ArcStringBuilder::from_iter(["\0\0\0\0", "\0\0\0"].into_iter()).as_str(), "\0\0\0\0\0\0\0");
	assert_eq!(ArcStringBuilder::from_iter(["\0\0\0\0", "\0\0\0\0"].into_iter()).as_str(), "\0\0\0\0\0\0\0\0");
	assert_eq!(ArcStringBuilder::from_iter(["\0\0\0\0", "\0\0\0\0\0"].into_iter()).as_str(), "\0\0\0\0\0\0\0\0\0");
	assert_eq!(ArcStringBuilder::from_iter(["1234", "5678"].into_iter()).as_str(), "12345678");
	assert_eq!(ArcStringBuilder::from_iter(["123", "45678"].into_iter()).as_str(), "12345678");
	assert_eq!(ArcStringBuilder::from_iter(["12345", "678"].into_iter()).as_str(), "12345678");
	assert_eq!(ArcStringBuilder::from_iter(["12", "34"].into_iter()).as_str(), "1234");
	assert_eq!(ArcStringBuilder::from_iter(["1", "234"].into_iter()).as_str(), "1234");
	assert_eq!(ArcStringBuilder::from_iter(["123", "4"].into_iter()).as_str(), "1234");
	assert_eq!(ArcStringBuilder::from_iter(["1", "2"].into_iter()).as_str(), "12");
	assert_eq!(ArcStringBuilder::from_iter(["", "12"].into_iter()).as_str(), "12");
	assert_eq!(ArcStringBuilder::from_iter(["12", ""].into_iter()).as_str(), "12");
	assert_eq!(ArcStringBuilder::from_iter(["12", "3"].into_iter()).as_str(), "123");
	assert_eq!(ArcStringBuilder::from_iter(["テスト・", "テキ"].into_iter()).as_str(), "テスト・テキ");
	assert_eq!(ArcStringBuilder::from_iter(["テスト・", "テキ", "スト"].into_iter()).as_str(), "テスト・テキスト");
	let s1 = ArcStringBuilder::from("テスト・テキスト");
	let s2 = ArcStringBuilder::from("テスト・テキスト");
	let s3 = ArcStringBuilder::from("テスト・テキスト");
	assert_eq!(s1, s2);
	assert_eq!(s2, s3);
	assert_eq!(ArcStringBuilder::from("テスト"), ArcStringBuilder::from("テスト"));
	assert_eq!(ArcStringBuilder::from_display(FormattableThing("abc".into())).as_str(), "abc");
	assert_eq!(ArcStringBuilder::from_display(FormattableThing("abcdefghijklmnopqrstuvwxyz".into())).as_str(), "abcdefghijklmnopqrstuvwxyz");

	assert_eq!(ArcStringBuilder::from("テスト"), ArcString::from("テスト"));
}

#[test]
fn test_builder_clone() {
	const TEXT: &str = "the quick brown fox jumps over the lazy dog";

	// an inline builder owns no allocation, so cloning it is a plain copy
	let sso = ArcStringBuilder::from("sso");
	assert_eq!(sso.clone().as_str(), "sso");

	// a builder longer than MAX_SSO_LEN owns a heap allocation, and the clone
	// must get one of its own without touching the allocation being cloned
	let mut builder = ArcStringBuilder::from(TEXT);
	assert!(builder.capacity() > arcstring::MAX_SSO_LEN);
	let clone = builder.clone();
	assert_eq!(clone.as_str(), TEXT);
	assert_eq!(clone.as_str(), builder.as_str());
	assert_eq!(clone, builder);
	builder.push_str(" twice");
	assert_ne!(clone, builder);

	// and dropping one leaves the other usable
	drop(clone);
	assert_eq!(builder.as_str(), "the quick brown fox jumps over the lazy dog twice");
}

#[test]
fn test_builder_shrink_to_fit() {
	// a string that fits inline again has to give up its buffer, since a capacity
	// of MAX_SSO_LEN is what marks a builder as inline
	let mut builder = ArcStringBuilder::with_capacity(64);
	builder.push_str("12345678");
	builder.shrink_to_fit();
	assert_eq!(builder.capacity(), arcstring::MAX_SSO_LEN);
	assert_eq!(builder.as_str(), "12345678");
	builder.push_str("9");
	assert_eq!(builder.as_str(), "123456789");
	assert!(builder.capacity() > arcstring::MAX_SSO_LEN);
}

#[test]
fn test_builder_inline() {
	const NULS: &str = "\0\0\0\0\0\0\0\0";

	// an inline builder holds the same encoding an inline ArcString wants
	// check that changing from ArcString to ArcStringBuilder works in both direction
	for s in ["", "1", "1234567", "12345678", NULS, &NULS[..7]] {
		let arcstring = ArcString::from(s);
		let builder = ArcStringBuilder::from(arcstring.clone());
		assert_eq!(builder.as_str(), s);
		assert_eq!(builder.len(), s.len());
		assert_eq!(builder.into_arcstring(), arcstring);

		let mut builder = ArcStringBuilder::new();
		for c in s.chars() {
			builder.push(c);
		}
		assert_eq!(builder.as_str(), s);
		assert_eq!(builder.clone().into_arcstring().as_str(), s);
		assert_eq!(builder.leak().as_str(), s);
	}

	// a boxed builder whose contents are short enough is encoded inline again
	let mut roomy = ArcStringBuilder::with_capacity(64);
	roomy.push_str("short");
	assert!(roomy.capacity() > arcstring::MAX_SSO_LEN);
	let short = roomy.into_arcstring();
	assert!(!short.is_boxed());
	assert_eq!(short.as_str(), "short");

	// MAX_SSO_LEN NUL bytes have no inline encoding, so they are boxed
	let boxed = ArcStringBuilder::from(NULS).into_arcstring();
	assert!(boxed.is_boxed());
	assert_eq!(boxed.as_str(), NULS);
	assert_eq!(ArcStringBuilder::from(NULS).leak().as_str(), NULS);
}
