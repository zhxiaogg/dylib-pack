use std::env;
use std::process::Command;
use std::fs;
use std::path::Path;

struct Dylib {
    path: String
}

impl Dylib {
    fn new(path: &str) -> Dylib {
        Dylib { path: path.to_string() }
    }

    fn file_path(&self) -> String {
        let parent = Path::new(&self.path).parent().expect("wtf: no parent?");
        let real_path = fs::read_link(&self.path)
                            .map(|pb| _get_absolute_path(pb.as_path(), parent))
                            .unwrap_or(None);
        match real_path {
            Some(ref path) => path.to_string(),
            None => self.path.clone()
        }
    }

    fn file_name(&self) -> String {
        self.file_path().split("/").last().expect("wtf").to_string()
    }
}

fn _get_absolute_path(path: &Path, parent: &Path) -> Option<String> {
    if path.is_absolute() {
        path.to_str().map(|s| s.to_string())
    } else {
        parent.join(path).canonicalize().map(|p| p.to_str().map(|s| s.to_string())).expect("wtf")
    }
}

fn main() {
    let args:Vec<String> = env::args().collect();
    let file = &args[1];
    let libs_dir = &args[2];
    let libs_prefix = &args[3];

    // create libs dir if not existed
    if !Path::new(libs_dir).exists() {
        fs::create_dir(libs_dir).expect("create libs dir failed.");
    }

    // find direct libs
    let direct_libs = _find_dylibs_for_img(file);

    //TODO: find hierarchy libs

    // replace each libs
    direct_libs.iter().for_each(|dylib| _replace(file, libs_dir, libs_prefix, &dylib))
}

fn _replace(img_file: &str, libs_dir: &str, libs_prefix: &str, lib: &Dylib) {
    let lib_name = &lib.file_name();

    // copy lib to the libs dir
    let target_lib_file = &format!("{}{}", libs_dir, lib_name);
    if !Path::new(&target_lib_file).exists() {
        fs::copy(&lib.file_path(), target_lib_file).expect(&format!("copy {} failed", target_lib_file));
    }

    // exec install_name_tool
    let replaced_lib = &format!("{}{}", libs_prefix, lib_name);
    let output = Command::new("install_name_tool")
        .arg("-change")
        .arg(&lib.path)
        .arg(replaced_lib)
        .arg(img_file)
        .output()
        .expect(&format!("fail to exec: install_name_tool -change {} {} {}", &lib.path, replaced_lib, img_file));

    if !output.status.success() {
        panic!("install_name_tool exec error: {}", String::from_utf8_lossy(&output.stderr));
    }
}

fn _find_dylibs_for_img(img_file: &str) -> Vec<Dylib> {
    let output = Command::new("otool")
        .arg("-L")
        .arg(img_file)
        .output()
        .expect(&format!("fail to exec: otool -L {}", img_file));

    if !output.status.success() {
        panic!("otool exec error: {}", String::from_utf8_lossy(&output.stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // println!("output: {}", stdout);

    let direct_libs:Vec<Dylib> = stdout.lines()
        .filter_map(|s| _find_libs(&s))
        .collect();
        // .for_each(|lib| println!("lib: {} {:?}", lib.path, lib.real_path));

    direct_libs
}

fn _find_libs(s:&str) -> Option<Dylib> {
    let lib = s.trim();
    if lib.contains("compatibility version") && lib.contains(".dylib") && !lib.starts_with("/usr/lib/") && !lib.starts_with("@"){
        let mut splits = lib.split(".dylib (");
        splits.next().map(|s| Dylib::new(&format!("{}.dylib", s)))
    } else {
        None
    }
}
