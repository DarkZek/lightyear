use crate::utils::wrapping_id::WrappedId;
use std::marker::PhantomData;

/// Fixed size data structure with
/// - constant time insertion
/// - constant time get by key
/// - constant time removal
///
/// The key must be a WrappedId.
/// More optimized than HashMap
pub struct SequenceBuffer<K: WrappedId, T, const N: usize> {
    buffer: [Option<T>; N],
    _marker: PhantomData<K>,
}

impl<K: WrappedId, T, const N: usize> SequenceBuffer<K, T, N> {
    pub fn new() -> Self {
        Self {
            buffer: std::array::from_fn(|_| None),
            _marker: PhantomData,
        }
    }

    pub fn push(&mut self, key: &K, value: T) {
        let index = self.index(key);
        // TODO: risk that we keep around the previously buffered value
        //  solution would be to clear values between the last insert and K (if K is more recent)
        self.buffer[index] = Some(value);
    }

    pub fn get(&self, key: &K) -> Option<&T> {
        let index = self.index(key);
        self.buffer[index].as_ref()
    }

    pub fn remove(&mut self, key: &K) -> Option<T> {
        let index = self.index(key);
        self.buffer[index].take()
    }

    fn index(&self, key: &K) -> usize {
        key.rem(N)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packet::message::MessageId;

    #[test]
    fn test_sequence_buffer() {
        let mut buffer = SequenceBuffer::<MessageId, u8, 32>::new();

        // check basic behaviour
        buffer.push(&MessageId(0), 0);
        assert_eq!(buffer.get(&MessageId(0)), Some(&0));

        assert_eq!(buffer.remove(&MessageId(0)), Some(0));

        // check loop around behaviour
        buffer.push(&MessageId(0), 0);
        buffer.push(&MessageId(32), 1);
        assert_eq!(buffer.get(&MessageId(0)), Some(&1));
    }
}