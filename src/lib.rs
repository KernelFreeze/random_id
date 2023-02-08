use aes::Aes256;
use fpe::ff1::{FlexibleNumeralString, FF1};

/// An iterator that returns decimal numbers in a random order.
/// All returned numbers are in the range of [0, 10^digits).
///
/// Numbers are generated using the FF1 algorithm.
pub struct RandomIdGenerator {
    key: [u8; 32],
    digits: u16,
    next: u16,
    tweak: Vec<u8>,
}

impl RandomIdGenerator {
    pub fn new(key: [u8; 32], tweak: Vec<u8>, digits: u16) -> Self {
        Self {
            key,
            tweak,
            digits,
            next: 0,
        }
    }

    /// Splits a 4 digits decimal number into its digits. Adds leading zeros if needed.
    fn split_number_digits(&self, mut number: u16) -> Vec<u16> {
        let mut digits = Vec::new();
        while number > 0 {
            digits.push(number % 10);
            number /= 10;
        }
        while digits.len() < self.digits as usize {
            digits.push(0);
        }
        digits.reverse();
        digits
    }

    fn join_number_digits(&self, digits: &[u16]) -> u16 {
        digits.iter().fold(0, |acc, &digit| acc * 10 + digit)
    }

    fn remaining(&self) -> usize {
        (self.total() - self.next) as usize
    }

    fn total(&self) -> u16 {
        10u16.pow(self.digits as u32)
    }

    fn next_random(&mut self) -> u16 {
        let input = self.split_number_digits(self.next);
        let numeral_string = FlexibleNumeralString::from(input);

        let ff = FF1::<Aes256>::new(&self.key, 10).unwrap();
        let output = ff.encrypt(&self.tweak, &numeral_string).unwrap();
        let output = Vec::from(output);

        self.next += 1;
        self.join_number_digits(&output)
    }
}

impl Iterator for RandomIdGenerator {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next >= self.total() {
            return None;
        }
        Some(self.next_random())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining(), Some(self.remaining()))
    }
}

#[cfg(test)]
mod tests {
    use rand::{Rng, SeedableRng};
    use rand_xoshiro::Xoshiro256PlusPlus;

    use super::*;

    #[test]
    fn test_split_number_digits() {
        let mut rng = Xoshiro256PlusPlus::from_entropy();
        let tweak = vec![0, 1, 2];

        let mut key = [0u8; 32];
        rng.fill(&mut key);

        let id_generator = RandomIdGenerator::new(key.clone(), tweak.clone(), 4);

        assert_eq!(id_generator.split_number_digits(0), [0, 0, 0, 0]);
        assert_eq!(id_generator.split_number_digits(1), [0, 0, 0, 1]);
        assert_eq!(id_generator.split_number_digits(8), [0, 0, 0, 8]);
        assert_eq!(id_generator.split_number_digits(10), [0, 0, 1, 0]);
        assert_eq!(id_generator.split_number_digits(123), [0, 1, 2, 3]);
        assert_eq!(id_generator.split_number_digits(1234), [1, 2, 3, 4]);

        let id_generator = RandomIdGenerator::new(key.clone(), tweak.clone(), 3);
        assert_eq!(id_generator.split_number_digits(0), [0, 0, 0]);
        assert_eq!(id_generator.split_number_digits(1), [0, 0, 1]);
        assert_eq!(id_generator.split_number_digits(8), [0, 0, 8]);
        assert_eq!(id_generator.split_number_digits(10), [0, 1, 0]);
        assert_eq!(id_generator.split_number_digits(123), [1, 2, 3]);

        let id_generator = RandomIdGenerator::new(key, tweak.clone(), 2);
        assert_eq!(id_generator.split_number_digits(0), [0, 0]);
        assert_eq!(id_generator.split_number_digits(1), [0, 1]);
        assert_eq!(id_generator.split_number_digits(8), [0, 8]);
        assert_eq!(id_generator.split_number_digits(10), [1, 0]);
    }

    #[test]
    fn test_join_number_digits() {
        let mut rng = Xoshiro256PlusPlus::from_entropy();
        let tweak = vec![0, 1, 2];

        let mut key = [0u8; 32];
        rng.fill(&mut key);

        let id_generator = RandomIdGenerator::new(key, tweak, 4);

        assert_eq!(id_generator.join_number_digits(&[]), 0);
        assert_eq!(id_generator.join_number_digits(&[1]), 1);
        assert_eq!(id_generator.join_number_digits(&[8]), 8);
        assert_eq!(id_generator.join_number_digits(&[1, 0]), 10);
        assert_eq!(id_generator.join_number_digits(&[1, 2, 3]), 123);
        assert_eq!(id_generator.join_number_digits(&[1, 2, 3, 4]), 1234);

        assert_eq!(id_generator.join_number_digits(&[0, 0, 0, 0]), 0);
        assert_eq!(id_generator.join_number_digits(&[0, 0, 0, 1]), 1);
        assert_eq!(id_generator.join_number_digits(&[0, 0, 0, 8]), 8);
        assert_eq!(id_generator.join_number_digits(&[0, 0, 1, 0]), 10);
        assert_eq!(id_generator.join_number_digits(&[0, 1, 2, 3]), 123);
        assert_eq!(id_generator.join_number_digits(&[1, 2, 3, 4]), 1234);
    }

    #[test]
    fn test_random_id() {
        let mut rng = Xoshiro256PlusPlus::from_entropy();
        let tweak = vec![0, 1, 2];

        let mut key = [0u8; 32];
        rng.fill(&mut key);

        let id_generator = RandomIdGenerator::new(key, tweak, 2);

        let mut ids = id_generator.collect::<Vec<_>>();
        ids.sort();
        assert_eq!(ids, (0..100).collect::<Vec<_>>());
    }

    #[test]
    fn test_iterator_finished_return_none() {
        let mut rng = Xoshiro256PlusPlus::from_entropy();
        let tweak = vec![0, 1, 2];

        let mut key = [0u8; 32];
        rng.fill(&mut key);

        let mut id_generator = RandomIdGenerator::new(key, tweak, 2);

        for _ in 0..100 {
            assert!(id_generator.next().is_some());
        }

        assert!(id_generator.next().is_none());
    }
}
