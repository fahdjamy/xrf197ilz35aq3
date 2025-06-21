use crate::core::Currency;
use rust_decimal::Decimal;
use sha3::{Digest, Sha3_256};
use std::collections::HashMap;

pub async fn get_exchange_rate(
    from_curr: Currency,
    to_currency: Currency,
) -> Result<Decimal, String> {
    let hash_code = get_hash_code(&from_curr.to_string(), &to_currency.to_string());
    // TODO: This should be coming from redis DB
    let mut hash_set: HashMap<String, Decimal> = HashMap::new();
    if !hash_set.contains_key(hash_code.as_str()) {
        hash_set.insert(hash_code.clone(), Decimal::from(0));
    }

    match hash_set.get(hash_code.as_str()) {
        Some(hash) => Ok(*hash),
        None => Err(format!(
            "no currency rate found for {} to {}",
            from_curr, to_currency
        )),
    }
}

/// Generates a unique identifier from two string slices.
///
/// This function creates a canonical representation of the two strings by joining them
/// with a null byte separator. This unambiguous representation is then hashed using
/// the SHA3-256 cryptographic algorithm.
///
/// The resulting 256-bit hash is returned as a 64-character hexadecimal string.
///
/// # Arguments
///
/// * `s1` - The first string slice.
/// * `s2` - The second string slice.
///
/// # Returns
///
/// A `String` representing the hexadecimal value of the SHA-256 hash. The probability
/// of a collision is negligible for any practical purpose.
///
/// # Panics
///
/// This function will not panic.
fn get_hash_code(s1: &str, s2: &str) -> String {
    // 1. Create a new SHA-256 hasher instance.
    let mut hasher = Sha3_256::new();

    // 2. Create the unambiguous input.
    // We update the hasher with the bytes of the first string,
    // then a separator, then the bytes of the second string.
    // The separator is crucial to distinguish between inputs like
    // ("AB", "C") and ("A", "BC"). A null byte is a good choice.
    hasher.update(s1.as_bytes());
    hasher.update(b"\0");
    hasher.update(s2.as_bytes());

    // 3. Finalize the hash computation.
    // The `finalize()` method consumes the hasher and returns a GenericArray.
    let result = hasher.finalize();

    // 4. Format the raw byte array as a hexadecimal string for easy use.
    // The `Output` type from the digest crate (v0.10+) doesn't implement
    // formatting traits like `LowerHex` directly. We manually iterate over
    // the bytes and format them into a hex string.

    // hasher.finalize() call produces a GenericArray of bytes (e.g., [56, 193, 167, ...] for SHA3-256).
    // .iter() creates an iterator that lets us process each byte one by one
    result
        .iter()
        // - (x): Format the number as lowercase hexadecimal
        // - (2): Ensure the output string is at least 2 characters wide
        // - (0): If the output is less than 2 characters, pad it with a leading 0
        // Why is: 02x so important? A single byte can range from 0 to 255. In hexadecimal,
        // this is 00 to ff. A byte with the value 10 is 0a in hex. If we only used {:x}
        // it would format as "a". By using {:02x}, we guarantee it formats as "0a". This ensures
        // every byte becomes exactly two hex characters, giving us a correctly formatted, fixed-length hash string
        .map(|byte| format!("{:02x}", byte))
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_hash_code() {
        let str1 = get_hash_code("test", "string");
        let str2 = get_hash_code("test", "string");
        assert_eq!(
            str1, str2,
            "Hashing the same inputs should always produce the same output."
        );

        let str1 = get_hash_code("hello", "world");
        let str2 = get_hash_code("hello", "world");
        assert_eq!(
            str1, str2,
            "Hashing the same inputs should always produce the same output."
        );

        let str1 = get_hash_code("helloworld", "");
        let str2 = get_hash_code("hello", "world");
        assert_ne!(
            str1, str2,
            "Hashing the same inputs should always produce the same output."
        );
    }
}
