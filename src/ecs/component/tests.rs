#![cfg(test)]

use super::{super::EntityStorage, *};

#[test]
fn test_insertion() {
    let mut entities = EntityStorage::with_key();
    let mut storage = ComponentStorage::new();

    let entity = entities.insert(());
    let component = "foo";

    storage.insert(entity, component);
    assert!(storage.attached(entity));
    assert_eq!(storage[entity], "foo");

    storage.remove(entity);
    assert!(!storage.attached(entity));
    assert_eq!(storage.get(entity), None);
}

#[test]
#[should_panic]
fn test_insertion_assert() {
    use std::time::Instant;

    let mut entities = EntityStorage::with_key();
    let mut storage = ComponentStorage::new();

    let entity1 = entities.insert(());
    let entity2 = entities.insert(());

    storage.insert(entity1, Instant::now());
    storage.insert(entity2, Instant::now());
    storage.insert(entity1, Instant::now());
}

#[test]
#[should_panic]
fn test_index() {
    let mut entities = EntityStorage::with_key();
    let mut storage = ComponentStorage::new();

    let entity = entities.insert(());
    storage[entity] = 0;
    assert_eq!(storage[entity], 0);

    let entity = entities.insert(());
    let _component = storage[entity];
}

#[test]
fn test_iterator() {
    let mut entities = EntityStorage::with_key();
    let mut storage = ComponentStorage::new();

    let _entities: Vec<_> = (0..100)
        .map(|int| {
            let entity = entities.insert(());
            storage.insert(entity, int);
            entity
        })
        .collect();

    for (_, component) in storage.iter_mut() {
        *component += 10;
    }
    for ((_, component), value) in storage.iter().zip(10..110) {
        assert_eq!(*component, value);
    }
    let iterator: IntoIter<i32> = storage.into_iter();
    let range: Vec<_> = iterator.map(|tuple| tuple.1).collect();
    assert_eq!(range, (10..110).collect::<Vec<_>>());
}
