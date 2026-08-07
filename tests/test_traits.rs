/* comparing a freshly built ArcString against a literal is the point of these
   tests, not an accident */
#![allow(clippy::cmp_owned)]
/* likewise, the collections being iterated over are Vecs on purpose */
#![allow(clippy::useless_vec)]

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::Path;

use arcstring::ArcString;

#[test]
fn test_deref() {
	let s = ArcString::from("テスト・テキスト");
	/* str methods and range indexing, without going through as_str() */
	assert!(s.starts_with("テスト"));
	assert_eq!(s.chars().count(), 8);
	assert_eq!(&s[0..9], "テスト");
	assert_eq!(&*ArcString::from("sso"), "sso");
	fn takes_str(s: &str) -> usize {
		s.len()
	}
	assert_eq!(takes_str(&s), 24);
}

#[test]
fn test_ord() {
	assert!(ArcString::from("abc") < ArcString::from("abd"));
	assert!(ArcString::from("abc") < ArcString::from("abcd"));
	/* the two strings are boxed, so the comparison cannot look at the word */
	assert!(ArcString::from("a long string, boxed") < ArcString::from("b long string, boxed"));
	assert!(ArcString::from("abc") >= ArcString::from("abc"));

	let mut sorted = vec![ArcString::from("b"), ArcString::from("c"), ArcString::from("a")];
	sorted.sort();
	assert_eq!(sorted, [ArcString::from("a"), ArcString::from("b"), ArcString::from("c")]);

	let mut map = BTreeMap::new();
	map.insert(ArcString::from("key"), 1);
	assert_eq!(map.get("key"), Some(&1));
}

#[test]
fn test_partial_eq_str() {
	let s = ArcString::from("hello");
	assert!(s == "hello");
	assert!("hello" == s);
	assert!(s != "world");
	assert!(*"hello" == s);
	assert!(s == *"hello");
	let boxed = ArcString::from("a long string, boxed");
	assert!(boxed == "a long string, boxed");
	assert!("a long string, boxed" == boxed);
}

#[test]
fn test_from_iterator() {
	assert_eq!(["a ", "long ", "string, ", "boxed"].into_iter().collect::<ArcString>(), "a long string, boxed");
	assert_eq!(["ab", "cd"].into_iter().collect::<ArcString>(), "abcd");
	assert_eq!("テスト・テキスト".chars().collect::<ArcString>(), "テスト・テキスト");
	assert_eq!(['a', 'b', 'c'].iter().collect::<ArcString>(), "abc");
	assert_eq!(vec![String::from("ab"), String::from("cd")].into_iter().collect::<ArcString>(), "abcd");
	assert_eq!(vec![String::from("ab"), String::from("cd")].iter().collect::<ArcString>(), "abcd");
	assert_eq!(vec![Box::<str>::from("ab"), Box::<str>::from("cd")].into_iter().collect::<ArcString>(), "abcd");
	assert_eq!([Cow::Borrowed("ab"), Cow::Owned(String::from("cd"))].into_iter().collect::<ArcString>(), "abcd");
	assert_eq!(Vec::<&str>::new().into_iter().collect::<ArcString>(), "");
	/* the inherent from_iter() is gone, so this now resolves to the trait */
	assert_eq!(ArcString::from_iter(["12", "3"]).as_str(), "123");
}

#[test]
fn test_from_str() {
	assert_eq!("sso".parse::<ArcString>().unwrap(), "sso");
	assert_eq!("a long string, boxed".parse::<ArcString>().unwrap(), "a long string, boxed");
}

#[test]
fn test_from() {
	assert_eq!(ArcString::from(&String::from("abc")), "abc");
	assert_eq!(ArcString::from(Box::<str>::from("a long string, boxed")), "a long string, boxed");
	assert_eq!(ArcString::from(Cow::Borrowed("abc")), "abc");
	assert_eq!(ArcString::from(Cow::Owned(String::from("abc"))), "abc");
}

#[test]
fn test_try_from_utf8() {
	assert_eq!(ArcString::try_from(b"abc".as_slice()).unwrap(), "abc");
	assert_eq!(ArcString::try_from("テスト".as_bytes()).unwrap(), "テスト");
	assert_eq!(ArcString::try_from(vec![b'a', b'b']).unwrap(), "ab");
	assert!(ArcString::try_from([0xff, 0xfe].as_slice()).is_err());
	assert!(ArcString::try_from(vec![0xff, 0xfe]).is_err());
}

#[test]
fn test_as_ref() {
	let s = ArcString::from("a long string, boxed");
	let bytes: &[u8] = s.as_ref();
	assert_eq!(bytes, b"a long string, boxed");
	let os: &OsStr = s.as_ref();
	assert_eq!(os, OsStr::new("a long string, boxed"));
	let filename = ArcString::from("/etc/passwd");
	let path: &Path = filename.as_ref();
	assert_eq!(path, Path::new("/etc/passwd"));
	/* the AsRef impls are what let an ArcString be passed to std APIs directly */
	assert!(Path::new("/etc").join(ArcString::from("passwd")).ends_with("passwd"));
}
