fn main() {
    let includes = &[std::path::PathBuf::from("src/proto")];
    let mut protos = Vec::new();
    for include in includes {
        for file in std::fs::read_dir(include).unwrap() {
            let file = file.unwrap();
            if file.file_type().unwrap().is_dir() {
                continue;
            }
            let path = file.path();
            if path.extension().unwrap() == "proto" {
                protos.push(path);
            }
        }
    }
    prost_build::compile_protos(&protos, includes).unwrap();
    for p in protos {
        println!("cargo:rerun-if-changed={}", p.display());
    }
}
