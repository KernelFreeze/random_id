use aes::Aes256;
use fpe::ff1::{FlexibleNumeralString, FF1};

/// An iterator that returns decimal numbers in a random order.
/// All returned numbers are in the range of [0, 10^digits).
///
/// Numbers are generated using the FF1 algorithm.
///
/// ## Usage
/// ```
/// use random_id::RandomIdGenerator;
/// use rand::prelude::*;
///
/// let mut rng = rand::thread_rng();
/// let mut key = [0u8; 32];
/// rng.fill(&mut key);
///
/// let mut id_generator = RandomIdGenerator::new(key, 0, 1);
///
/// for i in id_generator.take(10) {
///    println!("{}", i);
/// }
/// ```
pub struct RandomIdGenerator {
    key: [u8; 32],
    digits: u16,
    next: u16,
    tweak: Vec<u8>,
}

impl RandomIdGenerator {
    pub fn new(key: [u8; 32], tweak: u64, digits: u16) -> Self {
        let tweak = tweak.to_be_bytes().to_vec();
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

    fn join_number_digits(digits: &[u16]) -> u16 {
        digits.iter().fold(0, |acc, &digit| acc * 10 + digit)
    }

    fn remaining(&self) -> usize {
        (self.len() - self.next) as usize
    }

    fn len(&self) -> u16 {
        10u16.pow(self.digits as u32)
    }
}

impl Iterator for RandomIdGenerator {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        let input = self.split_number_digits(self.next);
        let numeral_string = FlexibleNumeralString::from(input);

        let ff = FF1::<Aes256>::new(&self.key, 10).ok()?;
        let output = ff.encrypt(&self.tweak, &numeral_string).ok()?;
        let output = Vec::from(output);

        self.next += 1;
        Some(Self::join_number_digits(&output))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining(), Some(self.remaining()))
    }

    fn count(mut self) -> usize {
        let remaining = self.remaining();

        // Set next to total to make sure that the iterator is exhausted.
        self.next = self.len();
        remaining
    }

    fn last(mut self) -> Option<Self::Item> {
        if self.next >= self.len() {
            return None;
        }

        // Set next to total - 1 to make sure that the iterator returns the last element.
        self.next = self.len() - 1;
        self.next()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if self.next + n as u16 >= self.len() {
            return None;
        }

        self.next += n as u16;
        self.next()
    }
}

#[cfg(test)]
mod tests {
    use rand::prelude::*;

    use super::*;

    #[test]
    fn test_split_number_digits() {
        let mut rng = rand::thread_rng();
        let mut key = [0u8; 32];
        rng.fill(&mut key);

        let id_generator = RandomIdGenerator::new(key.clone(), 0, 4);
        assert_eq!(id_generator.split_number_digits(0), [0, 0, 0, 0]);
        assert_eq!(id_generator.split_number_digits(1), [0, 0, 0, 1]);
        assert_eq!(id_generator.split_number_digits(8), [0, 0, 0, 8]);
        assert_eq!(id_generator.split_number_digits(10), [0, 0, 1, 0]);
        assert_eq!(id_generator.split_number_digits(123), [0, 1, 2, 3]);
        assert_eq!(id_generator.split_number_digits(1234), [1, 2, 3, 4]);

        let id_generator = RandomIdGenerator::new(key.clone(), 0, 3);
        assert_eq!(id_generator.split_number_digits(0), [0, 0, 0]);
        assert_eq!(id_generator.split_number_digits(1), [0, 0, 1]);
        assert_eq!(id_generator.split_number_digits(8), [0, 0, 8]);
        assert_eq!(id_generator.split_number_digits(10), [0, 1, 0]);
        assert_eq!(id_generator.split_number_digits(123), [1, 2, 3]);

        let id_generator = RandomIdGenerator::new(key, 0, 2);
        assert_eq!(id_generator.split_number_digits(0), [0, 0]);
        assert_eq!(id_generator.split_number_digits(1), [0, 1]);
        assert_eq!(id_generator.split_number_digits(8), [0, 8]);
        assert_eq!(id_generator.split_number_digits(10), [1, 0]);
    }

    #[test]
    fn test_join_number_digits() {
        let mut rng = rand::thread_rng();
        let mut key = [0u8; 32];
        rng.fill(&mut key);

        assert_eq!(RandomIdGenerator::join_number_digits(&[]), 0);
        assert_eq!(RandomIdGenerator::join_number_digits(&[1]), 1);
        assert_eq!(RandomIdGenerator::join_number_digits(&[8]), 8);
        assert_eq!(RandomIdGenerator::join_number_digits(&[1, 0]), 10);
        assert_eq!(RandomIdGenerator::join_number_digits(&[1, 2, 3]), 123);
        assert_eq!(RandomIdGenerator::join_number_digits(&[1, 2, 3, 4]), 1234);
        assert_eq!(RandomIdGenerator::join_number_digits(&[0, 0, 0, 0]), 0);
        assert_eq!(RandomIdGenerator::join_number_digits(&[0, 0, 0, 1]), 1);
        assert_eq!(RandomIdGenerator::join_number_digits(&[0, 0, 0, 8]), 8);
        assert_eq!(RandomIdGenerator::join_number_digits(&[0, 0, 1, 0]), 10);
        assert_eq!(RandomIdGenerator::join_number_digits(&[0, 1, 2, 3]), 123);
        assert_eq!(RandomIdGenerator::join_number_digits(&[1, 2, 3, 4]), 1234);
    }

    #[test]
    fn test_random_id() {
        let mut rng = rand::thread_rng();
        let mut key = [0u8; 32];
        rng.fill(&mut key);

        let id_generator = RandomIdGenerator::new(key, 0, 2);
        let mut ids = id_generator.collect::<Vec<_>>();
        ids.sort();

        assert_eq!(ids, (0..100).collect::<Vec<_>>());
    }

    #[test]
    fn test_iterator_finished_return_none() {
        let mut rng = rand::thread_rng();
        let mut key = [0u8; 32];
        rng.fill(&mut key);

        let mut id_generator = RandomIdGenerator::new(key, 0, 2);
        for _ in 0..100 {
            assert!(id_generator.next().is_some());
        }

        assert!(id_generator.next().is_none());
    }

    #[test]
    fn test_iterator_size_hint() {
        let mut rng = rand::thread_rng();
        let mut key = [0u8; 32];
        rng.fill(&mut key);

        let id_generator = RandomIdGenerator::new(key, 0, 2);
        assert_eq!(id_generator.size_hint(), (100, Some(100)));
    }

    #[test]
    fn test_iterator_count() {
        let mut rng = rand::thread_rng();
        let mut key = [0u8; 32];
        rng.fill(&mut key);

        let id_generator = RandomIdGenerator::new(key, 0, 2);
        assert_eq!(id_generator.count(), 100);
    }

    #[test]
    fn iterator_nth() {
        let mut rng = rand::thread_rng();
        let mut key = [0u8; 32];
        rng.fill(&mut key);

        let mut id_generator = RandomIdGenerator::new(key, 0, 2);
        assert!(id_generator.nth(98).is_some());
        assert!(id_generator.nth(0).is_some());
        assert!(id_generator.nth(0).is_none());
    }
}
