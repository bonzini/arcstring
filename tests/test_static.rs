use arcstring::{ArcString, arcstring};

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
