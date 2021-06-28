use crate::LinkedHashMap;
use ahash::RandomState;

#[test]
fn test_with_capacity_and_hasher() {
    let _ = LinkedHashMap::<i32, i32>::with_capacity(1);
    let _ = LinkedHashMap::<i32, i32, RandomState>::with_hasher(RandomState::new());
    let _ = LinkedHashMap::<i32, i32, RandomState>::with_capacity_and_hasher(1, RandomState::new());
}

#[test]
fn test_empty() {
    let mut map = LinkedHashMap::new();
    assert!(map.is_empty());
    map.insert(1, 0);
    assert!(!map.is_empty());
    map.remove(&1);
    assert!(map.is_empty());
}

#[test]
fn test_clear() {
    let mut map = LinkedHashMap::new();
    map.insert(1, 0);
    map.clear();
    assert!(map.is_empty());
}

#[test]
fn test_iter() {
    let mut map = LinkedHashMap::default();
    map.extend(vec![(1, 1), (2, 2), (3, 3)]);
    assert_eq!(
        map.clone()
            .into_iter()
            .map(|(_, v)| v)
            .collect::<Vec<i32>>(),
        vec![1, 2, 3]
    );
    assert_eq!(
        map.iter().map(|(k, _)| *k).collect::<Vec<i32>>(),
        vec![1, 2, 3]
    );
}

#[test]
fn test_back_an_front() {
    let mut map = LinkedHashMap::new();
    map.insert(1, 2);
    assert_eq!(map.front(), Some((&1, &2)));
    assert_eq!(map.back(), Some((&1, &2)));
    map.push_front(3, 4);
    assert_eq!(map.front(), Some((&3, &4)));
    assert_eq!(map.back(), Some((&1, &2)));
}

#[test]
fn test_push_and_pop() {
    let mut map = LinkedHashMap::with_capacity(2);
    map.push_front(1, "a");
    map.push_front(2, "b");
    assert_eq!(map.pop_front(), Some((2, "b")));
    assert_eq!(map.pop_front(), Some((1, "a")));
    assert_eq!(map.pop_front(), None);
    assert_eq!(map.len(), 0);

    map.push_back(1, "a");
    assert_eq!(map.pop_back(), Some((1, "a")));
    assert_eq!(map.pop_back(), None);
    assert_eq!(map.len(), 0);

    map.push_back(2, "b");
    map.push_front(1, "a");
    map.push_back(3, "c");
    assert_eq!(
        map.iter().map(|(k, _)| *k).collect::<Vec<i32>>(),
        vec![1, 2, 3]
    );
}

#[test]
fn test_insert_get_and_remove() {
    let mut map = LinkedHashMap::new();

    map.insert(1, "a");
    map.insert(2, "b");
    map.insert(3, "c");

    map.get_mut(&1).map(|v| *v = "A");
    assert_eq!(map.get(&1), Some(&"A"));

    assert!(map.contains(&1));

    assert_eq!(map.remove(&2), Some((2, "b")));
    assert!(!map.contains(&2));
    assert_eq!(map.get(&3), Some(&"c"));
}

#[test]
fn test_remove() {
    let mut map = LinkedHashMap::new();
    map.insert(1, "a");
    map.insert(2, "b");
    assert_eq!(map.remove(&1), Some((1, "a")));
    map.pop_front_node();
    assert!(map.is_empty());
}

#[test]
fn test_take() {
    let mut map = LinkedHashMap::new();
    map.insert(1, "a");
    assert_eq!(map.take(&1), Some((1, "a")));
    assert!(map.is_empty());
}

#[test]
fn test_pos() {
    let mut map = LinkedHashMap::new();

    map.insert(1, "a");
    map.insert(2, "b");

    *map.position_mut(1).unwrap().1 = "bb";

    assert_eq!(map.position(0), Some((&1, &"a")));
    assert_eq!(map.position(1), Some((&2, &"bb")));
    assert_eq!(map.position(2), None);
}

#[test]
fn test_debug() {
    let mut map = LinkedHashMap::new();
    map.insert(1, "a");
    assert_eq!(format!("{:?}", map), r###"{1: "a"}"###);
}

#[cfg(all(test, feature = "serde"))]
mod test_serde {
    use crate::LinkedHashMap;

    const JSON: &'static str = r#"{"1":"a","2":"b"}"#;

    #[test]
    fn test_ser() {
        let mut map = LinkedHashMap::new();
        map.insert(1, "a");
        map.insert(2, "b");
        assert_eq!(serde_json::to_string(&map).unwrap().as_str(), JSON);
    }

    #[test]
    fn test_de() {
        let map = serde_json::from_str::<LinkedHashMap<i32, String>>(JSON).unwrap();
        assert_eq!(map.front(), Some((&1i32, &"a".to_owned())));
        assert_eq!(map.back(), Some((&2i32, &"b".to_owned())));
    }
}
