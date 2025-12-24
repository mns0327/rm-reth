use std::{hash::Hash, ops::Deref};

use dashmap::DashMap;
use parity_scale_codec::{Decode, Encode, Input, Output};

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct DashMapCodec<K: Eq + Hash + Clone + Encode + Decode, V: Clone + Encode + Decode>(
    DashMap<K, V>,
);

impl<K: Eq + Hash + Clone + Encode + Decode, V: Clone + Encode + Decode> DashMapCodec<K, V> {
    pub fn new() -> Self {
        Self(DashMap::new())
    }
}

impl<K: Eq + Hash + Clone + Encode + Decode, V: Clone + Encode + Decode> Encode
    for DashMapCodec<K, V>
{
    fn encode_to<T: Output + ?Sized>(&self, dest: &mut T) {
        let items: Vec<(K, V)> = self
            .0
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();

        items.encode_to(dest);
    }
}

impl<K: Eq + Hash + Clone + Encode + Decode, V: Clone + Encode + Decode> Decode
    for DashMapCodec<K, V>
{
    fn decode<I: Input>(input: &mut I) -> Result<Self, parity_scale_codec::Error> {
        let items = Vec::<(K, V)>::decode(input)?;

        let points = DashMap::with_capacity(items.len());
        for (key, value) in items {
            points.insert(key, value);
        }

        Ok(DashMapCodec(points))
    }
}

impl<K: Eq + Hash + Clone + Encode + Decode, V: Clone + Encode + Decode> Deref
    for DashMapCodec<K, V>
{
    type Target = DashMap<K, V>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parity_scale_codec::{Decode, Encode};
    use std::sync::Arc;
    use std::thread;

    #[derive(Debug, Clone, PartialEq, Encode, Decode)]
    struct TestValue(u64);

    #[test]
    fn test_encode_decode_empty() {
        let map: DashMapCodec<u32, TestValue> = DashMapCodec::new();

        let encoded = map.encode();
        let decoded = DashMapCodec::<u32, TestValue>::decode(&mut &encoded[..]).unwrap();

        assert_eq!(decoded.len(), 0);
    }

    #[test]
    fn test_encode_decode_single_item() {
        let map: DashMapCodec<u32, TestValue> = DashMapCodec::new();
        map.insert(42, TestValue(100));

        let encoded = map.encode();
        let decoded = DashMapCodec::<u32, TestValue>::decode(&mut &encoded[..]).unwrap();

        assert_eq!(decoded.len(), 1);
        assert!(decoded.contains_key(&42));
        assert_eq!(decoded.get(&42).unwrap().clone(), TestValue(100));
    }

    #[test]
    fn test_encode_decode_multiple_items() {
        let map: DashMapCodec<u32, TestValue> = DashMapCodec::new();

        let test_data = vec![
            (1, TestValue(10)),
            (2, TestValue(20)),
            (3, TestValue(30)),
            (100, TestValue(1000)),
            (u32::MAX, TestValue(u64::MAX)),
        ];

        for (key, value) in &test_data {
            map.insert(*key, value.clone());
        }

        let encoded = map.encode();
        let decoded = DashMapCodec::<u32, TestValue>::decode(&mut &encoded[..]).unwrap();

        assert_eq!(decoded.len(), test_data.len());

        for (key, expected_value) in test_data {
            assert!(decoded.contains_key(&key));
            assert_eq!(decoded.get(&key).unwrap().clone(), expected_value);
        }
    }

    #[test]
    fn test_encode_decode_string_keys() {
        let map: DashMapCodec<String, TestValue> = DashMapCodec::new();

        map.insert("key1".to_string(), TestValue(100));
        map.insert("key2".to_string(), TestValue(200));
        map.insert("key3".to_string(), TestValue(300));

        let encoded = map.encode();
        let decoded = DashMapCodec::<String, TestValue>::decode(&mut &encoded[..]).unwrap();

        assert_eq!(decoded.len(), 3);
        assert_eq!(decoded.get("key1").unwrap().clone(), TestValue(100));
        assert_eq!(decoded.get("key2").unwrap().clone(), TestValue(200));
        assert_eq!(decoded.get("key3").unwrap().clone(), TestValue(300));
    }

    #[test]
    fn test_encode_decode_large_dataset() {
        let map: DashMapCodec<u32, TestValue> = DashMapCodec::new();

        for i in 0..1000 {
            map.insert(i, TestValue(i as u64 * 10));
        }

        let encoded = map.encode();
        let decoded = DashMapCodec::<u32, TestValue>::decode(&mut &encoded[..]).unwrap();

        assert_eq!(decoded.len(), 1000);

        for i in [0, 100, 500, 999] {
            assert!(decoded.contains_key(&i));
            assert_eq!(decoded.get(&i).unwrap().clone(), TestValue(i as u64 * 10));
        }
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let map1: DashMapCodec<u32, TestValue> = DashMapCodec::new();
        map1.insert(1, TestValue(100));
        map1.insert(2, TestValue(200));

        let encoded1 = map1.encode();
        let decoded1 = DashMapCodec::<u32, TestValue>::decode(&mut &encoded1[..]).unwrap();

        let encoded2 = decoded1.encode();
        let decoded2 = DashMapCodec::<u32, TestValue>::decode(&mut &encoded2[..]).unwrap();

        assert_eq!(decoded2.len(), 2);
        assert_eq!(decoded2.get(&1).unwrap().clone(), TestValue(100));
        assert_eq!(decoded2.get(&2).unwrap().clone(), TestValue(200));
    }

    #[test]
    fn test_decode_invalid_data() {
        let invalid_data = vec![0xFF, 0xFF, 0xFF, 0xFF];
        let result = DashMapCodec::<u32, TestValue>::decode(&mut &invalid_data[..]);

        assert!(result.is_err());
    }

    #[test]
    fn test_decode_empty_bytes() {
        let empty_data = vec![];
        let result = DashMapCodec::<u32, TestValue>::decode(&mut &empty_data[..]);

        assert!(result.is_err());
    }

    #[test]
    fn test_concurrent_access() {
        let map: Arc<DashMapCodec<u32, TestValue>> = Arc::new(DashMapCodec::new());
        let mut handles = vec![];

        for i in 0..10 {
            let map_clone = Arc::clone(&map);
            let handle = thread::spawn(move || {
                map_clone.insert(i, TestValue(i as u64 * 100));
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(map.len(), 10);

        for i in 0..10 {
            assert_eq!(map.get(&i).unwrap().clone(), TestValue(i as u64 * 100));
        }
    }

    #[test]
    fn test_clone() {
        let map: DashMapCodec<u32, TestValue> = DashMapCodec::new();
        map.insert(1, TestValue(100));
        map.insert(2, TestValue(200));

        let cloned = map.clone();

        assert_eq!(cloned.len(), 2);
        assert_eq!(cloned.get(&1).unwrap().clone(), TestValue(100));
        assert_eq!(cloned.get(&2).unwrap().clone(), TestValue(200));
    }

    #[test]
    fn test_with_different_types() {
        #[derive(Debug, Clone, PartialEq, Encode, Decode)]
        struct ComplexValue {
            id: u32,
            name: String,
            data: Vec<u8>,
        }

        let map: DashMapCodec<String, ComplexValue> = DashMapCodec::new();

        let value1 = ComplexValue {
            id: 1,
            name: "test1".to_string(),
            data: vec![1, 2, 3],
        };
        let value2 = ComplexValue {
            id: 2,
            name: "test2".to_string(),
            data: vec![4, 5, 6],
        };

        map.insert("key1".to_string(), value1.clone());
        map.insert("key2".to_string(), value2.clone());

        let encoded = map.encode();
        let decoded = DashMapCodec::<String, ComplexValue>::decode(&mut &encoded[..]).unwrap();

        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded.get("key1").unwrap().clone(), value1);
        assert_eq!(decoded.get("key2").unwrap().clone(), value2);
    }
}
