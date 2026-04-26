use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs::{ReadDir, read_dir},
    path::PathBuf,
};

fn main() {
    let user_args: Vec<String> = std::env::args().collect();

    //check for arg
    if user_args.len() < 2 {
        return;
    }

    //define var
    let current_dir: &String = &user_args[1];

    //go through all files in that dir and push them as DirEntry
    let files_iter = match read_dir(current_dir) {
        Ok(i) => i,
        Err(err) => {
            println!("Error with reading Directory: {err}");
            return;
        }
    };

    //Take DirEntry and generate a hashmap that has valus of vec<PathBuf> where
    //hashed file contents are the key's
    let hmap = hasher(files_iter);

    let mut counter: u8 = 0;
    for (_k, v) in hmap {
        if v.len() > 1 {
            for p in v {
                counter += 1;
                let x = p.to_str().unwrap();
                println!("{x}");
            }
        }
    }
    println!("Found {counter} duplicates:");
}

fn hasher(files_iter: ReadDir) -> HashMap<Vec<u8>, Vec<PathBuf>> {
    let mut hmap: HashMap<Vec<u8>, Vec<PathBuf>> = HashMap::new();

    for file in files_iter {
        let f_res = match file {
            Ok(f) => f,
            Err(_) => continue,
        };

        let file_path_buf: PathBuf = f_res.path();
        if file_path_buf.is_file() {
            let data = match std::fs::read(&file_path_buf) {
                Ok(d) => d,
                Err(_) => continue,
            };
            let hash_result = Sha256::digest(data).as_slice().to_vec();
            let pbuf_vec = hmap.entry(hash_result).or_insert(Vec::new());
            pbuf_vec.push(file_path_buf);
        }
    }

    hmap
}
