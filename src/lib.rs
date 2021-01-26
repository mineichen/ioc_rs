use {
    core::{
        marker::PhantomData,
        any::{Any, TypeId}
    }
};

mod binary_search;

// The family trait for type constructors that have one input lifetime.
pub trait FamilyLt<'a> {
    type Out: 'a;
}

#[derive(Debug)]
pub struct IdFamily<T: Any>(PhantomData<T>);
impl<'a, T: 'a + Any> FamilyLt<'a> for IdFamily<T> {
    type Out = T;
}

#[derive(Debug)]
pub struct RefFamily<T: Any>(PhantomData<T>);
impl<'a, T: 'a + Any> FamilyLt<'a> for RefFamily<T> {
    type Out = &'a T;
}

impl<'a, T: FamilyLt<'a> + 'a> FamilyLt<'a> for Option<T> {
    type Out = Option<T::Out>;
}

impl<'a, T0: FamilyLt<'a> + 'a, T1: FamilyLt<'a> + 'a> FamilyLt<'a> for (T0, T1) {
    type Out = (
        T0::Out,
        T1::Out
    );
}
impl<'a, T0: FamilyLt<'a> + 'a, T1: FamilyLt<'a> + 'a, T2: FamilyLt<'a> + 'a> FamilyLt<'a> for (T0, T1, T2) {
    type Out = (
        T0::Out,
        T1::Out,
        T2::Out
    );
}
impl<'a, T0: FamilyLt<'a>, T1: FamilyLt<'a>, T2: FamilyLt<'a>, T3: FamilyLt<'a>> FamilyLt<'a> for (T0, T1, T2, T3) {
    type Out = (
        <T0 as FamilyLt<'a>>::Out,
        <T1 as FamilyLt<'a>>::Out,
        <T2 as FamilyLt<'a>>::Out,
        <T3 as FamilyLt<'a>>::Out
    );
}

pub trait Resolvable: Any {
    type Item: for<'a> FamilyLt<'a>;
    type ItemPreChecked: for<'a> FamilyLt<'a>;

    fn resolve<'s>(provider: &'s ServiceProvider) -> <Self::Item as FamilyLt<'s>>::Out;
    fn resolve_prechecked<'s>(provider: &'s ServiceProvider) -> <Self::ItemPreChecked as FamilyLt<'s>>::Out;
}

impl Resolvable for () {
    type Item = IdFamily<()>;
    type ItemPreChecked = IdFamily<()>;

    fn resolve<'s>(_: &'s ServiceProvider) -> <Self::Item as FamilyLt<'s>>::Out {
        ()
    }
    fn resolve_prechecked<'s>(_: &'s ServiceProvider) -> <Self::ItemPreChecked as FamilyLt<'s>>::Out {
        ()
    }
}

impl<T0: Resolvable, T1: Resolvable> Resolvable for (T0, T1) {
    type Item = (T0::Item, T1::Item);
    type ItemPreChecked = (T0::ItemPreChecked, T1::ItemPreChecked);
  
    fn resolve<'s>(container: &'s ServiceProvider) -> <Self::Item as FamilyLt<'s>>::Out {
        (container.get::<T0>(), container.get::<T1>())
    }
  
    fn resolve_prechecked<'s>(container: &'s ServiceProvider) -> <Self::ItemPreChecked as FamilyLt<'s>>::Out {
        (T0::resolve_prechecked(container), T1::resolve_prechecked(container))
    }
}
impl<T0: Resolvable, T1: Resolvable, T2: Resolvable> Resolvable for (T0, T1, T2) {
    type Item = (T0::Item, T1::Item, T2::Item);
    type ItemPreChecked = (T0::ItemPreChecked, T1::ItemPreChecked, T2::ItemPreChecked);
  
    fn resolve<'s>(container: &'s ServiceProvider) -> <Self::Item as FamilyLt<'s>>::Out {
        (container.get::<T0>(), container.get::<T1>(), container.get::<T2>())
    }
    fn resolve_prechecked<'s>(container: &'s ServiceProvider) -> <Self::ItemPreChecked as FamilyLt<'s>>::Out {
        (T0::resolve_prechecked(container), T1::resolve_prechecked(container), T2::resolve_prechecked(container))
    }
}
impl<T0: Resolvable, T1: Resolvable, T2: Resolvable, T3: Resolvable> Resolvable for (T0, T1, T2, T3) {
    type Item = (T0::Item, T1::Item, T2::Item, T3::Item);
    type ItemPreChecked = (T0::ItemPreChecked, T1::ItemPreChecked, T2::ItemPreChecked, T3::ItemPreChecked);

    fn resolve<'s>(container: &'s ServiceProvider) -> <Self::Item as FamilyLt<'s>>::Out {
        (
            container.get::<T0>(), 
            container.get::<T1>(),
            container.get::<T2>(),
            container.get::<T3>()
        )
    }
    fn resolve_prechecked<'s>(container: &'s ServiceProvider) -> <Self::ItemPreChecked as FamilyLt<'s>>::Out {
        (
            T0::resolve_prechecked(container), 
            T1::resolve_prechecked(container),
            T2::resolve_prechecked(container),
            T3::resolve_prechecked(container)
        )
    }
}

impl Resolvable for ServiceProvider {
    type Item = RefFamily<ServiceProvider>;
    type ItemPreChecked  = RefFamily<ServiceProvider>;

    fn resolve<'s>(container: &'s ServiceProvider) -> <Self::Item as FamilyLt<'s>>::Out {
        container
    }

    fn resolve_prechecked<'s>(container: &'s ServiceProvider) -> <Self::Item as FamilyLt<'s>>::Out {
        Self::resolve(container)
    }
}
pub struct Singleton<T: Any>(PhantomData<T>);
impl<T: Any> Resolvable for Singleton<T> {
    type Item = Option<RefFamily<T>>;
    type ItemPreChecked = RefFamily<T>;

    fn resolve<'s>(container: &'s ServiceProvider) -> <Self::Item as FamilyLt<'s>>::Out {
        binary_search::binary_search_by_last_key(&container.producers, &TypeId::of::<Self>(), |(id, _)| id)
            .map(|f| {  
                unsafe { resolve_unchecked::<Self>(container, f) }
            })
    }

    fn resolve_prechecked<'s>(container: &'s ServiceProvider) -> <Self::ItemPreChecked as FamilyLt<'s>>::Out {
        Self::resolve(container).unwrap()
    }
}

unsafe fn resolve_unchecked<'a, T: Resolvable>(container: &'a ServiceProvider, pos: usize) -> <T::ItemPreChecked as FamilyLt<'a>>::Out {
    ({
        let func_ptr = container.producers.get_unchecked(pos).1 as *const dyn Fn(&ServiceProvider) -> <T::ItemPreChecked as FamilyLt<'a>>::Out;
        &* func_ptr
    })(&container)
}

pub struct ServiceIterator<'a, T> {
    next_pos: Option<usize>,
    provider: &'a ServiceProvider, 
    item_type: PhantomData<T>
}

impl<'a, T: Resolvable> std::iter::Iterator for ServiceIterator<'a, T> {
    type Item = <T::ItemPreChecked as FamilyLt<'a>>::Out;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_pos.map(|i| {
            self.next_pos = if let Some(next) = self.provider.producers.get(i + 1) {
                if next.0 == TypeId::of::<T>() { 
                    Some(i + 1) 
                } else {
                    None
                }
            } else {
                None
            };
            
            unsafe { resolve_unchecked::<T>(self.provider, i) }
        })
    }

    fn last(self) -> Option<Self::Item> where Self: Sized {
        self.next_pos.map(|i| {
            // If has_next, last must exist
            let pos = binary_search::binary_search_by_last_key(&self.provider.producers[i..], &TypeId::of::<T>(), |(id, _)| id).unwrap();
            unsafe { resolve_unchecked::<T>(self.provider, i+pos)}            
        }) 
    }
    fn count(self) -> usize where Self: Sized {
        self.next_pos.map(|i| {
            let pos = binary_search::binary_search_by_last_key(&self.provider.producers[i..], &TypeId::of::<T>(), |(id, _)| id).unwrap();
            pos + 1       
        }).unwrap_or(0)
    }
}
pub struct ServiceIteratorFamily<T>(PhantomData<T>);

impl<'a, T: Resolvable> FamilyLt<'a> for ServiceIteratorFamily<T> {
    type Out = ServiceIterator<'a, T>;
}

pub struct TransientServices<T: Any>(PhantomData<T>);
impl<T: Any> Resolvable for TransientServices<T> {
    type Item = ServiceIteratorFamily<Transient<T>>;
    type ItemPreChecked = ServiceIteratorFamily<Transient<T>>;

    fn resolve<'s>(container: &'s ServiceProvider) -> <Self::Item as FamilyLt<'s>>::Out {
        let next_pos = binary_search::binary_search_by_first_key(&container.producers, &TypeId::of::<Transient<T>>(), |(id, _)| id);
        ServiceIterator { provider: &container, item_type: PhantomData, next_pos }
    }

    fn resolve_prechecked<'s>(container: &'s ServiceProvider) -> <Self::ItemPreChecked as FamilyLt<'s>>::Out {
        Self::resolve(container)
    }
}

pub struct Transient<T: Any>(PhantomData<T>);
impl<T: Any> Resolvable for Transient<T> {
    type Item = Option<IdFamily<T>>;
    type ItemPreChecked = IdFamily<T>;

    fn resolve<'s>(container: &'s ServiceProvider) -> <Self::Item as FamilyLt<'s>>::Out {
        binary_search::binary_search_by_last_key(&container.producers, &TypeId::of::<Self>(), |(id, _)| id)
            .map(|f| {    
                unsafe { resolve_unchecked::<Self>(container, f) }
            })
    }

    fn resolve_prechecked<'s>(container: &'s ServiceProvider) -> <Self::ItemPreChecked as FamilyLt<'s>>::Out {
        Self::resolve(container).unwrap()
    }
}


pub struct ServiceCollection {
    producers: Vec<(TypeId, *const dyn Fn())>,
}

impl ServiceCollection {
    pub fn new() -> Self {
        Self {
            producers: Vec::new()
        }
    }
} 
impl Drop for ServiceCollection {
    fn drop(&mut self) {
        for p in self.producers.iter_mut() {
            unsafe { drop(Box::from_raw(p.1 as *mut dyn Fn())) };
        }
    }
}

impl ServiceCollection {
    pub fn with<T: Resolvable>(&mut self) -> ServiceCollectionWithDependency<'_, T> {
        ServiceCollectionWithDependency(self, PhantomData)
    }

    pub fn register_transient<'s, 'a: 's, T: Any>(&'s mut self, creator: fn() -> T) {
        let func : Box<dyn Fn(&'a ServiceProvider) -> T> = Box::new(move |_: &'a ServiceProvider| {
            creator()
        });
        
        self.producers.push((
            TypeId::of::<Transient<T>>(), 
            Box::into_raw(func) as *const dyn Fn()
        ));
    }

    pub fn register_singleton<'s, 'a: 's, T: Any + Sync>(&'s mut self, creator: fn() -> T) {
        let cell = once_cell::sync::OnceCell::new();
        let func : Box<dyn Fn(&'a ServiceProvider) -> &T> = Box::new(move |_: &'a ServiceProvider| { 
            unsafe { 
                // Deref is valid because provider never alters any producers
                // Unless destroying itself
                &*(cell.get_or_init(|| {
                    creator()
                }) as *const T)
            }
        });
        
        self.producers.push((
            TypeId::of::<Singleton<T>>(), 
            Box::into_raw(func) as *const dyn Fn()
        ));
    }
    pub fn build(mut self) -> Result<ServiceProvider, ()> {
        self.producers.sort_by_key(|(id,_)| *id);
        let mut producers = Vec::new();
        core::mem::swap(&mut self.producers, &mut producers);
        Ok (ServiceProvider { producers })
    }
}

pub struct ServiceCollectionWithDependency<'col, T: Resolvable>(&'col mut ServiceCollection, PhantomData<T>);
impl<'col, TDep: Resolvable> ServiceCollectionWithDependency<'col, TDep> {
    pub fn register_transient<'s, 'a: 's, T: Any>(&'s mut self, creator: fn(<TDep::ItemPreChecked as FamilyLt<'a>>::Out) -> T) {
        let func : Box<dyn Fn(&'a ServiceProvider) -> T> = Box::new(move |container: &'a ServiceProvider| {
            let arg = TDep::resolve_prechecked(container);
            creator(arg)
        });
        
        self.0.producers.push((
            TypeId::of::<Transient<T>>(), 
            Box::into_raw(func) as *const dyn Fn()
        ));
    }
    pub fn register_singleton<'s, 'a: 's, T: Any + Sync>(&'s mut self, creator: fn(<TDep::ItemPreChecked as FamilyLt<'a>>::Out) -> T) {
        let cell = once_cell::sync::OnceCell::new();
        let func : Box<dyn Fn(&'a ServiceProvider) -> &T> = Box::new(move |container: &'a ServiceProvider| { 
            unsafe { 
                // Deref is valid because provider never alters any producers
                // Unless destroying itself
                &*(cell.get_or_init(|| {
                    let arg = TDep::resolve_prechecked(container);
                    creator(arg)
                }) as *const T)
            }
        });
        
        self.0.producers.push((
            TypeId::of::<Singleton<T>>(), 
            Box::into_raw(func) as *const dyn Fn()
        ));
    }
}
pub struct ServiceProvider {
    /// Mustn't be changed because `resolve_unchecked` relies on it.
    producers: Vec<(TypeId, *const dyn Fn())>
}

impl Drop for ServiceProvider {
    fn drop(&mut self) {
        for p in self.producers.iter_mut() {
            unsafe { drop(Box::from_raw(p.1 as *mut dyn Fn())) };
        }
    }
}

impl ServiceProvider {
    pub fn get<'s, T: Resolvable>(&'s self) -> <T::Item as FamilyLt<'s>>::Out {
        T::resolve(self)
    }
}

#[cfg(test)]
mod tests {
    use {super::* };
    
    #[test]
    fn resolve_last_transient() {
        let mut col = ServiceCollection::new();
        col.register_transient(|| 0);
        col.register_transient(|| 5);
        col.register_transient(|| 1);
        col.register_transient(|| 2);
        let provider = col.build().expect("Expected to have all dependencies");
        let nr = provider.get::<Transient::<i32>>().unwrap();
        assert_eq!(2, nr);
    }

    #[test]
    fn resolve_last_singleton() {
        let mut container = ServiceCollection::new();
        container.register_singleton(|| 0);
        container.register_singleton(|| 1);
        container.register_singleton(|| 2);
        let provider = container.build().expect("Expected to have all dependencies");
        let nr_ref = provider.get::<Singleton::<i32>>().unwrap();
        assert_eq!(
            2, 
            *nr_ref
        );
    }

    #[test]
    fn resolve_transient_services() {
        let mut container = ServiceCollection::new();
        container.register_transient(|| 0);
        container.register_transient(|| 5);
        container.register_transient(|| 2);
        let provider = container.build().expect("Expected to have all dependencies");

        // Count
        let mut count_subset = provider.get::<TransientServices::<i32>>();
        count_subset.next();
        assert_eq!(2, count_subset.count());
        assert_eq!(3, provider.get::<TransientServices::<i32>>().count());

        // Last
        assert_eq!(Some(2), provider.get::<TransientServices::<i32>>().last());
        
        let mut sub = provider.get::<TransientServices::<i32>>();
        sub.next();
        assert_eq!(Some(2), sub.last());

        let mut consumed = provider.get::<TransientServices::<i32>>();
        consumed.by_ref().for_each(|_| {});
        assert_eq!(None, consumed.last());
        
        let mut iter = provider.get::<TransientServices::<i32>>();
        assert_eq!(Some(0), iter.next());
        assert_eq!(Some(5), iter.next());
        assert_eq!(Some(2), iter.next());
        assert_eq!(None, iter.next());        
    }

    #[test]
    fn resolve_test() {
        let mut container = ServiceCollection::new();
        container.register_transient(|| 42);
        container.register_singleton(|| 42);
        let provider = container.build().expect("Expected to have all dependencies");
        assert_eq!(
            provider.get::<Transient::<i32>>().unwrap(), 
            provider.get::<Singleton::<i32>>().map(|f| *f).unwrap()
        );
    }

    #[test]
    fn get_registered_dynamic_id() {
        let mut container = ServiceCollection::new();
        container.register_transient(|| 42);
        assert_eq!(
            Some(42i32), 
            container.build()
                .expect("Expected to have all dependencies")
                .get::<Transient<i32>>()
        );
    }
    #[test]
    fn get_registered_dynamic_ref() {
        let mut container = ServiceCollection::new();
        container.register_singleton(|| 42);
        assert_eq!(
            Some(&42i32), 
            container.build()
                .expect("Expected to have all dependencies")
                .get::<Singleton<i32>>());
    }

    #[test]
    fn tuple_dependency_resolves_to_prechecked_type() {
        let mut container = ServiceCollection::new();
        container.register_transient(|| 64i64);
        container.with::<(Transient<i64>, Transient<i64>)>().register_singleton(|(a, b)| {
            assert_eq!(64, a);
            assert_eq!(64, b);
            42
        });
        assert_eq!(Some(&42i32), container.build().expect("Expected to have all dependencies").get::<Singleton<i32>>());
    }

    #[test]
    fn get_unkown_returns_none() {
        let container = ServiceCollection::new();
        assert_eq!(
            None, 
            container.build()
                .expect("Expected to have all dependencies")
                .get::<Transient<i32>>()
        );
    }

    #[test]
    fn resolve_tuple_2() {
        let mut container = ServiceCollection::new();
        container.register_transient(|| 32i32);
        container.register_singleton(|| 64i64);
        assert_eq!(
            (Some(32), Some(&64)), 
            container.build()
                .expect("Expected to have all dependencies")
                .get::<(Transient<i32>, Singleton<i64>)>()
        );
    }

    #[test]
    fn register_struct_as_dynamic() {
        let mut container = ServiceCollection::new();   
        container.register_singleton(|| 42i32);
        container.with::<Singleton<i32>>().register_singleton(|i| ServiceImpl(i));
        container.with::<Singleton<ServiceImpl>>().register_transient(|c| c as &dyn Service);
        let provider = container.build().expect("Expected to have all dependencies");
        let service = provider.get::<Transient<&dyn Service>>()
            .expect("Expected to get a service");
       
        assert_eq!(42, service.get_value());
    }

    trait Service {
        fn get_value(&self) -> i32;
    }

    struct ServiceImpl<'a>(&'a i32);
    impl<'a> Service for ServiceImpl<'a> {
        fn get_value(&self) -> i32 {
            println!("Before getting");
            *self.0
        }
    }

    #[test]
    fn drop_singletons_after_provider_drop() {
        let mut col = ServiceCollection::new();
        col.register_singleton(|| Test);
        let prov = col.build().unwrap();
        drop(prov);
        assert_eq!(0, unsafe { DROP_COUNT });
        
        let mut col = ServiceCollection::new();
        col.register_singleton(|| Test);
        let prov = col.build().expect("Expected to have all dependencies");
        prov.get::<Singleton<Test>>().expect("Expected to receive the service");
        drop(prov);
        assert_eq!(1, unsafe { DROP_COUNT });
    }

    static mut DROP_COUNT: u8 = 0;
    struct Test;
    impl Drop for Test {
        fn drop(&mut self) {
            unsafe { DROP_COUNT += 1 };
        }
    }
}