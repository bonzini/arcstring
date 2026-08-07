/* comparing a freshly built builder against a literal is the point of these
   tests, not an accident */
#![allow(clippy::cmp_owned)]
/* likewise, the collections being iterated over are Vecs on purpose */
#![allow(clippy::useless_vec)]

use std::borrow::Cow;
use std::collections::BTreeMap;

use arcstring::{ArcString, ArcStringBuilder};

#[test]
fn test_deref() {
	let mut b = ArcStringBuilder::from("テスト・テキスト");
	/* str methods and range indexing, without going through as_str() */
	assert!(b.starts_with("テスト"));
	assert_eq!(b.chars().count(), 8);
	assert_eq!(&b[0..9], "テスト");
	fn takes_str(s: &str) -> usize {
		s.len()
	}
	assert_eq!(takes_str(&b), 24);

	/* DerefMut, so in-place str mutation works too */
	let mut ascii = ArcStringBuilder::from("a long string, boxed");
	ascii.make_ascii_uppercase();
	assert_eq!(ascii.as_str(), "A LONG STRING, BOXED");
	b.make_ascii_uppercase();
	assert_eq!(b.as_str(), "テスト・テキスト");
}

#[test]
fn test_ord() {
	assert!(ArcStringBuilder::from("abc") < ArcStringBuilder::from("abd"));
	assert!(ArcStringBuilder::from("abc") < ArcStringBuilder::from("abcd"));
	assert!(ArcStringBuilder::from("a long string, boxed") < ArcStringBuilder::from("b long string, boxed"));
	assert!(ArcStringBuilder::from("abc") >= ArcStringBuilder::from("abc"));

	let mut sorted = vec![ArcStringBuilder::from("b"), ArcStringBuilder::from("c"), ArcStringBuilder::from("a")];
	sorted.sort();
	assert_eq!(sorted, [ArcStringBuilder::from("a"), ArcStringBuilder::from("b"), ArcStringBuilder::from("c")]);

	let mut map = BTreeMap::new();
	map.insert(ArcStringBuilder::from("key"), 1);
	assert_eq!(map.get("key"), Some(&1));
}

#[test]
fn test_partial_eq_str() {
	let b = ArcStringBuilder::from("hello");
	assert!(b == "hello");
	assert!("hello" == b);
	assert!(b != "world");
	assert!(*"hello" == b);
	assert!(b == *"hello");
	let boxed = ArcStringBuilder::from("a long string, boxed");
	assert!(boxed == "a long string, boxed");
	assert!("a long string, boxed" == boxed);
	assert!(boxed == ArcString::from("a long string, boxed"));
	assert!(ArcString::from("a long string, boxed") == boxed);
}

#[test]
fn test_extend() {
	let mut b = ArcStringBuilder::from("a ");
	b.extend(["long ", "string, "]);
	b.extend([String::from("boxed")]);
	assert_eq!(b, "a long string, boxed");

	/* iterating over a Vec<String> by reference yields &String items */
	let pieces = vec![String::from("ab"), String::from("cd")];
	let mut b = ArcStringBuilder::new();
	b.extend(pieces.iter());
	assert_eq!(b, "abcd");
	assert_eq!(pieces.iter().collect::<ArcStringBuilder>(), "abcd");

	let mut b = ArcStringBuilder::new();
	b.extend("テスト".chars());
	b.extend(['・'].iter());
	b.extend([Box::<str>::from("テキ")]);
	b.extend([Cow::Borrowed("スト")]);
	assert_eq!(b, "テスト・テキスト");

	/* extending an empty builder past the inline size still grows correctly */
	let mut b = ArcStringBuilder::new();
	b.extend(std::iter::repeat_n("ab", 10));
	assert_eq!(b, "abababababababababab");
}

#[test]
fn test_from_iterator() {
	assert_eq!(["a ", "long ", "string, ", "boxed"].into_iter().collect::<ArcStringBuilder>(), "a long string, boxed");
	assert_eq!("テスト・テキスト".chars().collect::<ArcStringBuilder>(), "テスト・テキスト");
	assert_eq!(['a', 'b', 'c'].iter().collect::<ArcStringBuilder>(), "abc");
	assert_eq!(vec![String::from("ab"), String::from("cd")].into_iter().collect::<ArcStringBuilder>(), "abcd");
	assert_eq!(vec![Box::<str>::from("ab"), Box::<str>::from("cd")].into_iter().collect::<ArcStringBuilder>(), "abcd");
	assert_eq!([Cow::Borrowed("ab"), Cow::Owned(String::from("cd"))].into_iter().collect::<ArcStringBuilder>(), "abcd");
	assert_eq!(Vec::<&str>::new().into_iter().collect::<ArcStringBuilder>(), "");
}

#[test]
fn test_add_assign() {
	let mut b = ArcStringBuilder::from("a");
	b += " long";
	b += ' ';
	b += "string, boxed";
	assert_eq!(b, "a long string, boxed");
	assert_eq!(b.into_arcstring(), "a long string, boxed");
}

#[test]
fn test_from() {
	assert_eq!(ArcStringBuilder::from(&String::from("abc")), "abc");
	assert_eq!(ArcStringBuilder::from(Box::<str>::from("a long string, boxed")), "a long string, boxed");
	assert_eq!(ArcStringBuilder::from(Cow::Borrowed("abc")), "abc");
	assert_eq!(ArcStringBuilder::from(Cow::Owned(String::from("abc"))), "abc");
}
