use std::fs;

fn main() {
    let paths = fs::read_dir("src/bin").unwrap();
    
    let names = paths.filter_map(|entry| {
        entry.ok().and_then(|e|
          e.path().file_stem()
          .and_then(|n| n.to_str().map(|s| String::from(s)))
        )
      }).collect::<Vec<String>>();

    let mut base_address: u32 = 0x80400000;
    for b in names {
        println!("cargo:rustc-link-arg-bin={}=--defsym=BASE_ADDRESS={}", b, base_address);
        base_address += 0x20000;
    }
}
