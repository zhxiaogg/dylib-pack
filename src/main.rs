mod dylib;

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let args:Vec<String> = env::args().collect();
    let file = &args[1].trim();
    let libs_dir = &args[2].trim().trim_right_matches("/");
    let libs_prefix = &args[3].trim().trim_right_matches("/");

    // create libs dir if not existed
    if !Path::new(libs_dir).exists() {
        fs::create_dir(libs_dir).expect("create libs dir failed.");
    }

    // find all dylibs recursively
    let all_libs = dylib::find_dylibs_recursively(file);

    // print all libs
    println!("all found libs:");
    all_libs.iter().for_each(|l| println!("\t{}", &l.path));

    // replace dylib for input image and all its libs
    dylib::do_replace(&all_libs, file, libs_prefix, libs_dir);
}
