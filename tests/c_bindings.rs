use interoptopus::{Interop, Error};
use interoptopus_backend_c::{Generator, Config};

#[test]
fn bindings_c() -> Result<(), Error> {
    Generator::new(
        Config {
            ifndef: "rosu_ffi".to_string(),
            ..Config::default()
        },
        rosu_ffi::my_inventory(),
    )
    .write_file("bindings/rosu_ffi.h")?;

    Ok(())
}
