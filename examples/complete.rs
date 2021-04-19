//! This example combines all features of this library, except errorhandling
//! If you can understand all assertions, you've seen all aspects of this library

use {
    ioc_rs::{Dynamic, DynamicServices, ServiceCollection, ServiceProvider},
    std::sync::Arc,
};

fn main() {
    let mut parent_collection = ServiceCollection::new();
    parent_collection
        .with::<DynamicServices<u8>>()
        .register(|bytes| bytes.map(|signed| signed as i8).sum::<i8>());

    parent_collection
        .with::<Dynamic<Arc<u16>>>()
        .register(|i| *i as u32 * 2);
    parent_collection.register(|| 1u8);
    parent_collection.register_arc(|| Arc::new(10u16));

    let parent_provider = parent_collection
        .build_factory()
        .expect("All dependencies of parent should be resolvable")
        .build(2u8);

    let mut child_collection = ServiceCollection::new();
    child_collection.register(|| 3u8);
    child_collection
        .with::<(ServiceProvider, DynamicServices<u8>, Dynamic<u32>)>()
        .register(|(provider, bytes, int)| {
            provider.get::<Dynamic<Arc<u16>>>().map(|i| *i as u64).unwrap_or(1000) // Optional Dependency, fallback not used
                + provider.get::<Dynamic<u128>>().map(|i| i as u64).unwrap_or(2000) // Optional Dependency, fallback
                + bytes.map(|i| { i as u64 }).sum::<u64>()
                + int as u64
        });

    let child_provider = child_collection
        .with_parent(parent_provider.clone())
        .build()
        .expect("All dependencies of child should be resolvable")
        .build(4u8);

    assert_eq!(Some(2), parent_provider.get::<Dynamic<u8>>()); // Last registered i8 on parent
    assert_eq!(Some(4), child_provider.get::<Dynamic<u8>>()); // Last registered i8 on client

    assert_eq!(
        Some(10 + 2000 + (1 + 2 + 3 + 4) + (2 * 10)),
        child_provider.get::<Dynamic<u64>>()
    );

    assert_eq!(
        1 + 2, // Despite u8 beign received by child_provider, u8 is just the sum of i8 from parent
        child_provider.get::<Dynamic<i8>>().unwrap()
    );
}
