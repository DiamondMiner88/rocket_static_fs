extern crate rocket_static_fs;

use rocket_static_fs::fs::create_package_from_dir;
use std::fs::File;

fn main() {
    let test_package_path = concat!(env!("CARGO_MANIFEST_DIR"), "/target/test.package");
    let mut f = File::create(test_package_path).unwrap();

    let testdata_assets_path = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/assets");

    create_package_from_dir(testdata_assets_path, &mut f).unwrap();
}
