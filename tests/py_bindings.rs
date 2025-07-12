use interoptopus::{Interop, Error};
use interoptopus_backend_cpython::{Generator, Config};

#[test]
fn bindings_py() -> Result<(), Error> {
    Generator::new(
        Config {
            ..Config::default()
        },
        rosu_ffi::my_inventory(),
    )
    .write_file("bindings/rosu_ffi.py")?;

    Ok(())
}
