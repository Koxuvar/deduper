use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    env::current_dir,
    fs::{ReadDir, read_dir},
    path::PathBuf,
};

fn main() {
    //Grab user input in arguments
    let user_args: Vec<String> = std::env::args().collect();

    let recurse_check: bool = user_args.iter().any(|x| x == "-r");

    //Set directory to cwd if no arg is passed or 2 arg passed in args vec
    let directory: String = if user_args.len() < 2 {
        match current_dir() {
            Ok(s) => match s.to_str() {
                Some(s) => s.to_string(),
                None => {
                    println!("Error Occured");
                    return;
                }
            },
            Err(err) => {
                println!("Error occured: {err}");
                return;
            }
        }
    } else {
        user_args
            .iter()
            .skip(1)
            .find(|x| !x.starts_with("-"))
            .expect("Directory error")
            .to_string()
    };

    //go through all files in that dir and push them as DirEntry
    let files_iter = match read_dir(directory) {
        Ok(i) => i,
        Err(err) => {
            println!("Error with reading Directory: {err}");
            return;
        }
    };

    println!("{recurse_check}");
    let mut hmap: HashMap<Vec<u8>, Vec<PathBuf>> = HashMap::new();
    //Take DirEntry and generate a hashmap that has valus of vec<PathBuf> where
    //hashed file contents are the key's
    hasher(files_iter, &mut hmap, recurse_check);

    //Pass hmap into function that iterates over it and finds number of duplicates
    //prints total number of duplicate file instances then lists the paths to those files
    find_duplicates_and_print(hmap);
}

fn hasher(files_iter: ReadDir, hmap: &mut HashMap<Vec<u8>, Vec<PathBuf>>, recurse_check: bool) {
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
            let pbuf_vec = hmap.entry(hash_result).or_default();
            pbuf_vec.push(file_path_buf);
        } else {
            let new_dir = match read_dir(file_path_buf) {
                Ok(i) => i,
                Err(err) => {
                    println!("Error with reading Directory: {err}");
                    return;
                }
            };
            if recurse_check {
                hasher(new_dir, hmap, recurse_check);
            } else {
                continue;
            }
        }
    }
}

fn find_duplicates_and_print(hmap: HashMap<Vec<u8>, Vec<PathBuf>>) {
    let mut counter: u8 = 0;
    hmap.values().for_each(|v| {
        if v.len() > 1 {
            counter += 1
        }
    });

    println!("Found {counter} duplicates:");

    hmap.values().for_each(|v| {
        if v.len() > 1 {
            for p in v {
                let x = p.to_str().expect("Non Unicode Chars in file path");
                println!("{x}");
            }
        }
    });
}
