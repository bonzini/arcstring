use arcstring::{ArcString, ArcStringBuilder, arcstring};

const LONG: &str = "this string is far too long to be stored inline";

#[test]
fn test_static() {
	// strings that fit inline do not need a descriptor at all
	let sso = arcstring!("88888888");
	assert!(!sso.is_boxed());
	assert_eq!(sso.as_str(), "88888888");
	assert_eq!(sso.as_static(), None);
	assert_eq!(arcstring!("").as_str(), "");

	// longer ones are literals: neither inline nor heap allocated
	let long = arcstring!(LONG);
	assert!(!long.is_boxed());
	assert!(!long.is_empty());
	assert_eq!(long.as_static(), Some(LONG));
	assert_eq!(long.as_str(), LONG);

	// literals are not reference counted, so cloning and dropping is a no-op
	assert!(!long.clone().is_boxed());

	// the descriptor is promoted, so the same one is reused on every evaluation
	fn promoted() -> ArcString {
		arcstring!("yet another string that does not fit inline")
	}
	assert_eq!(promoted().as_str().as_ptr(), promoted().as_str().as_ptr());
}

#[test]
fn test_leak() {
	const TEXT: &str = "a string that is leaked instead of being reference counted";

	// a leaked string is indistinguishable from a literal
	let leaked = ArcString::leak_from(TEXT);
	assert_eq!(leaked.as_str(), TEXT);
	assert!(!leaked.is_boxed());
	assert!(!leaked.clone().is_boxed());

	// leak_from takes anything a builder can be built from
	assert_eq!(ArcString::leak_from(TEXT.to_owned()).as_str(), TEXT);
	assert_eq!(ArcString::leak_from('x').as_str(), "x");

	// the builder hands its own buffer over to the leaked string
	let mut builder = ArcStringBuilder::from(TEXT);
	builder.push_str(" is cool");
	let leaked = builder.leak();
	assert!(!leaked.is_boxed());
	assert_eq!(leaked.as_str(), "a string that is leaked instead of being reference counted is cool");

	// short strings are stored inline, so nothing is leaked at all
	let inline = ArcString::leak_from("sso");
	assert!(!inline.is_boxed());
	assert_eq!(inline.as_str(), "sso");
	assert_eq!(ArcString::leak_from("").as_str(), "");
	assert_eq!(ArcStringBuilder::new().leak(), ArcString::empty());
}
