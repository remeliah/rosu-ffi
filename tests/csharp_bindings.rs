use interoptopus::{Interop, Error};
use interoptopus_backend_csharp::{Generator, Config};

#[test]
fn bindings_cs() -> Result<(), Error> {
    Generator::new(
        Config {
            dll_name: "rosu_ffi".to_string(),
            ..Config::default()
        },
        rosu_ffi::my_inventory(),
    )
    .write_file("bindings/rosu_ffi.cs")?;

    Ok(())
}
