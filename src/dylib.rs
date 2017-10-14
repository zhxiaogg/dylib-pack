use std::fs;
use std::path::Path;
use std::process::Command;
use std::collections::HashSet;

#[derive(Hash, Eq, PartialEq, Debug)]
pub struct Dylib {
    pub path: String
}

impl Dylib {
    pub fn new(path: &str) -> Dylib {
        Dylib { path: path.to_string() }
    }

    pub fn file_path(&self) -> String {
        let parent = Path::new(&self.path).parent().expect("wtf: no parent?");
        let real_path = fs::read_link(&self.path)
                            .map(|pb| _get_absolute_path(pb.as_path(), parent))
                            .unwrap_or(None);
        match real_path {
            Some(ref path) => path.to_string(),
            None => self.path.clone()
        }
    }

    pub fn file_name(&self) -> String {
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


pub fn do_replace(all_libs: &HashSet<Dylib>, file:&str, libs_prefix:&str, libs_dir:&str) {
    // replace each libs in given image file
    println!("\nreplace:\n\t{}", file);
    all_libs.iter().for_each(|ref lib| _replace(file, libs_prefix, lib));

    // replace dylibs for each lib image
    all_libs.iter().for_each(|ref lib| {
        // copy lib to dest libs dir
        let target_lib_img = &format!("{}/{}", libs_dir,  &lib.file_name());
        if !Path::new(&target_lib_img).exists() {
            fs::copy(&lib.file_path(), target_lib_img).expect(&format!("copy {} failed", target_lib_img));
            let mut permission = fs::metadata(target_lib_img).expect("wtf: cannot read metadata").permissions();
            permission.set_readonly(false);
            fs::set_permissions(target_lib_img, permission).expect("wtf: set permission failed");
        }

        println!("\t{}", target_lib_img);
        all_libs.iter()
            .for_each(|ref l| _replace(target_lib_img, libs_prefix, l));
    });
}

pub fn find_dylibs_recursively(file:&str) -> HashSet<Dylib> {
    let mut all_libs = HashSet::new();

    // find direct libs
    let mut direct_libs = _find_dylibs_for_img(file);
    for lib in direct_libs.drain(..){
        all_libs.insert(lib);
    }

    let mut size = 0;
    while size != all_libs.len() {
        size = all_libs.len();

        let mut new_libs = HashSet::new();
        all_libs.iter().for_each(|lib| {
                let mut libs = _find_dylibs_for_img(&lib.file_path());
                for lib in libs.drain(..) {
                    new_libs.insert(lib);
                }
        });
        for lib in new_libs.drain() {
            all_libs.insert(lib);
        }
    }

    all_libs
}

fn _replace(img_file: &str, libs_prefix: &str, lib: &Dylib) {
    // exec install_name_tool
    let replaced_lib = &format!("{}/{}", libs_prefix, &lib.file_name());
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
