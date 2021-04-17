use {ioc_rs::Dynamic, std::sync::Arc};

type ServiceRegistrar = unsafe extern "C" fn(&mut ioc_rs::ServiceCollection);

///
/// # Expected output
///
/// plugin: Register Service
/// plugin: I duplicate 2
/// Runtime: service.call(2) = 4
/// Runtime: Get 42 multiplied by 3: 126
///
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Lib must be outside of unsafe block, because it's dropped otherwise resulting in a segfault
    let lib = libloading::Library::new("target/debug/libplugin.dylib")?;
    let mut container = ioc_rs::ServiceCollection::new();
    container.register(|| 42);
    
    unsafe {
        let func: libloading::Symbol<ServiceRegistrar> = lib.get(b"register")?;
        func(&mut container);
    }

    let provider = container
        .build()
        .expect("Expected all dependencies to resolve");

    let service = provider
        .get::<Dynamic<Arc<dyn interface::Service>>>()
        .expect("Expected plugin to register a &dyn Service");

    println!("Runtime: service.call(2) = {}", service.call(2));

    let number = provider
        .get::<ioc_rs::Dynamic<i64>>()
        .expect("Expected plugin to register i64");

    println!("Runtime: Get 42 multiplied by 3: {}", number);
    Ok(())
}
