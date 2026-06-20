use rand::RngExt;
use wyrand::WyRand;

const LOWER_CASE: &str = "0123456789abcdefghijklmnopqrstuvwxyz";
const MIXED_CASE: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

pub fn random_identifier(wyrand: &mut WyRand, length: usize, mixed_case: bool) -> String {
	let charset = if mixed_case { MIXED_CASE } else { LOWER_CASE };
	let mut result = String::new();
	for index in 0..length {
		result.push(char::from(if index > 0 {
			charset.as_bytes()[wyrand.random_range(0..charset.len())]
		} else {
			b'a' + wyrand.random_range(0..26)
		}));
	}
	result
}

#[cfg(test)]
mod tests {
	#[allow(unused_imports)]
	use super::*;

	#[test]
	fn test_random_identifier() {
		let mut wyrand = WyRand::new(1);
		let identifier = random_identifier(&mut wyrand, 6, true);
		assert_eq!(identifier, "sfwI8I");
		let identifier = random_identifier(&mut wyrand, 6, false);
		assert_eq!(identifier, "xdshbp");
		let identifier = random_identifier(&mut wyrand, 0, true);
		assert_eq!(identifier, "");
	}
}
