//! Shamir secret sharing over GF(2^8).
//!
//! Splits an arbitrary byte string into `n` shares so that any `k` of them
//! reconstruct it, while `k - 1` shares reveal nothing about the secret.
//! Used by the recovery workflow: the container key is split, the shares are
//! printed or stored separately, and no single share is enough on its own.

#![forbid(unsafe_code)]

use getrandom::getrandom as random_bytes;
use zeroize::Zeroize;

/// One share: the x coordinate plus one byte per secret byte.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Share {
    /// Non-zero x coordinate (share number, 1..=255).
    pub x: u8,
    /// Evaluated polynomial values, one byte per secret byte.
    pub y: Vec<u8>,
}

impl Drop for Share {
    fn drop(&mut self) {
        self.y.zeroize();
    }
}

/// Why a split or combine request was rejected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShamirError {
    /// Fewer than two shares or a threshold outside `2..=n`.
    BadThreshold,
    /// More than 255 shares requested.
    TooManyShares,
    /// The secret is empty.
    EmptySecret,
    /// A share has x = 0, which is the secret itself.
    BadShare,
    /// Two shares share the same x coordinate.
    DuplicateShare,
    /// Fewer shares than the threshold.
    NotEnoughShares,
    /// The system random source failed.
    Random,
}

impl core::fmt::Display for ShamirError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let text = match self {
            Self::BadThreshold => "threshold must be between 2 and the number of shares",
            Self::TooManyShares => "at most 255 shares are supported",
            Self::EmptySecret => "the secret is empty",
            Self::BadShare => "share number 0 is not allowed",
            Self::DuplicateShare => "duplicate share numbers",
            Self::NotEnoughShares => "not enough shares to reconstruct the secret",
            Self::Random => "the system random source is unavailable",
        };
        f.write_str(text)
    }
}

impl std::error::Error for ShamirError {}

/// Multiplication in GF(2^8) with the AES polynomial `x^8 + x^4 + x^3 + x + 1`.
fn mul(mut a: u8, mut b: u8) -> u8 {
    let mut result = 0_u8;
    for _ in 0..8 {
        if b & 1 == 1 {
            result ^= a;
        }
        let high = a & 0x80;
        a <<= 1;
        if high != 0 {
            a ^= 0x1b;
        }
        b >>= 1;
    }
    result
}

/// Multiplicative inverse in GF(2^8); zero has no inverse and maps to zero.
fn inverse(a: u8) -> u8 {
    if a == 0 {
        return 0;
    }
    // a^254 = a^-1 in GF(2^8)*.
    let mut result = 1_u8;
    let mut base = a;
    let mut exponent = 254_u8;
    while exponent != 0 {
        if exponent & 1 == 1 {
            result = mul(result, base);
        }
        base = mul(base, base);
        exponent >>= 1;
    }
    result
}

/// Splits `secret` into `n` shares, any `k` of which rebuild it.
///
/// # Errors
///
/// Returns [`ShamirError`] for an empty secret, an unusable threshold, or more
/// than 255 shares.
pub fn split(secret: &[u8], k: u8, n: u8) -> Result<Vec<Share>, ShamirError> {
    if secret.is_empty() {
        return Err(ShamirError::EmptySecret);
    }
    if k < 2 || k > n || n < 2 {
        return Err(ShamirError::BadThreshold);
    }
    let mut coefficients = vec![0_u8; usize::from(k) * secret.len()];
    random_bytes(&mut coefficients).map_err(|_| ShamirError::Random)?;
    let mut shares = Vec::with_capacity(usize::from(n));
    for x in 1..=n {
        let mut y = Vec::with_capacity(secret.len());
        for (index, byte) in secret.iter().enumerate() {
            let offset = index * usize::from(k);
            let mut value = *byte;
            let mut power = x;
            for coefficient in &coefficients[offset + 1..offset + usize::from(k)] {
                value ^= mul(*coefficient, power);
                power = mul(power, x);
            }
            y.push(value);
        }
        shares.push(Share { x, y });
    }
    coefficients.zeroize();
    Ok(shares)
}

/// Rebuilds the secret from at least `k` distinct shares.
///
/// # Errors
///
/// Returns [`ShamirError`] when the share set is too small, contains duplicate
/// or invalid coordinates, or has mismatched lengths.
pub fn combine(k: u8, shares: &[Share]) -> Result<Vec<u8>, ShamirError> {
    let k = usize::from(k.max(2));
    if shares.len() < k {
        return Err(ShamirError::NotEnoughShares);
    }
    let shares = &shares[..k];
    let len = shares[0].y.len();
    if len == 0 || shares.iter().any(|share| share.y.len() != len) {
        return Err(ShamirError::BadShare);
    }
    let mut seen = [false; 256];
    for share in shares {
        if share.x == 0 || seen[usize::from(share.x)] {
            return Err(if share.x == 0 {
                ShamirError::BadShare
            } else {
                ShamirError::DuplicateShare
            });
        }
        seen[usize::from(share.x)] = true;
    }
    let mut secret = vec![0_u8; len];
    for (index, byte) in secret.iter_mut().enumerate() {
        let mut value = 0_u8;
        for (i, share) in shares.iter().enumerate() {
            let mut basis = 1_u8;
            for (j, other) in shares.iter().enumerate() {
                if i != j {
                    // In GF(2^8) subtraction is XOR: x_j / (x_j ^ x_i).
                    basis = mul(basis, mul(other.x, inverse(other.x ^ share.x)));
                }
            }
            value ^= mul(share.y[index], basis);
        }
        *byte = value;
    }
    Ok(secret)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_any_threshold_subset() {
        let secret = b"correct horse battery staple";
        let shares = split(secret, 3, 5).expect("split");
        for subset in [[0, 1, 2], [0, 2, 4], [1, 3, 4], [2, 3, 4]] {
            let picked: Vec<Share> = subset.iter().map(|i| shares[*i].clone()).collect();
            assert_eq!(combine(3, &picked).expect("combine"), secret);
        }
    }

    #[test]
    fn fewer_shares_do_not_match() {
        let secret = b"top secret";
        let shares = split(secret, 3, 5).expect("split");
        let picked = vec![shares[0].clone(), shares[1].clone()];
        assert_eq!(combine(2, &picked).unwrap().len(), secret.len());
        assert_ne!(combine(2, &picked).unwrap(), secret);
    }

    #[test]
    fn rejects_bad_parameters() {
        assert_eq!(split(b"x", 1, 3), Err(ShamirError::BadThreshold));
        assert_eq!(split(b"x", 4, 3), Err(ShamirError::BadThreshold));
        assert_eq!(split(b"", 2, 3), Err(ShamirError::EmptySecret));
        let shares = split(b"x", 2, 3).expect("split");
        assert_eq!(combine(4, &shares), Err(ShamirError::NotEnoughShares));
        let duplicate = vec![shares[0].clone(), shares[0].clone()];
        assert_eq!(combine(2, &duplicate), Err(ShamirError::DuplicateShare));
    }

    #[test]
    fn gf_inverse_property() {
        for value in 1..=255_u8 {
            assert_eq!(mul(value, inverse(value)), 1);
        }
    }
}
