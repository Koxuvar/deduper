use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    env::current_dir,
    fs::{ReadDir, read_dir},
    path::PathBuf,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
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

    /* let new_thing: Arc<Mutex<HashMap<Vec<u8>, Vec<PathBuf>>>> =
        Arc::new(Mutex::new(HashMap::new()));
    //Take DirEntry and generate a hashmap that has valus of vec<PathBuf> where
    //hashed file contents are the key's
    hasher(files_iter, new_thing.clone(), recurse_check); */
    let new_hmap = new_hasher(files_iter, recurse_check);

    //Pass hmap into function that iterates over it and finds number of duplicates
    //prints total number of duplicate file instances then lists the paths to those files
    find_duplicates_and_print(new_hmap);
}

fn hasher(
    files_iter: ReadDir,
    hmap: Arc<Mutex<HashMap<Vec<u8>, Vec<PathBuf>>>>,
    recurse_check: bool,
) {
    let mut thread_handles: Vec<JoinHandle<()>> = Vec::new();
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

            let h_clone = Arc::clone(&hmap);

            let thread_handle = thread::spawn(move || {
                let mut hmap_guard = h_clone.lock().expect("error here 32");
                let hash_result = Sha256::digest(data).as_slice().to_vec();
                let pbuf_vec = hmap_guard.entry(hash_result).or_default();
                pbuf_vec.push(file_path_buf);
            });
            thread_handles.push(thread_handle);
        } else {
            let new_dir = match read_dir(file_path_buf) {
                Ok(i) => i,
                Err(err) => {
                    println!("Error with reading Directory: {err}");
                    return;
                }
            };
            if recurse_check {
                hasher(new_dir, hmap.clone(), recurse_check);
            } else {
                continue;
            }
        }
    }

    for handle in thread_handles {
        handle.join().unwrap();
    }
}

fn new_hasher(files_iter: ReadDir, recurse_check: bool) -> HashMap<Vec<u8>, Vec<PathBuf>> {
    let mut file_paths: Vec<PathBuf> = Vec::new();

    for file in files_iter {
        file_paths.push(match file {
            Ok(f) => f.path(),
            Err(_) => continue,
        });
    }

    let new_hmap = file_paths
        .into_par_iter()
        .fold(
            HashMap::new,
            |mut new_hmap: HashMap<Vec<u8>, Vec<PathBuf>>, file: PathBuf| {
                if file.is_file() {
                    let data = match std::fs::read(&file) {
                        Ok(d) => d,
                        Err(_) => return new_hmap,
                    };

                    let hash_result = Sha256::digest(data).as_slice().to_vec();
                    let pbuf_vec = new_hmap.entry(hash_result).or_default();
                    pbuf_vec.push(file);

                    new_hmap
                } else {
                    let new_dir = read_dir(file).unwrap();
                    if recurse_check {
                        let rec_hmap = new_hasher(new_dir, recurse_check);

                        for (k, v) in rec_hmap {
                            let pbuf_vec = new_hmap.entry(k).or_default();
                            pbuf_vec.extend(v);
                        }
                    };
                    new_hmap
                }
            },
        )
        .reduce(
            HashMap::new,
            |mut new_hmap: HashMap<Vec<u8>, Vec<PathBuf>>,
             ret_hmap: HashMap<Vec<u8>, Vec<PathBuf>>| {
                for (k, v) in ret_hmap {
                    let pbuf_vec = new_hmap.entry(k).or_default();
                    pbuf_vec.extend(v);
                }

                new_hmap
            },
        );
    return new_hmap;
}

fn find_duplicates_and_print(hmap: HashMap<Vec<u8>, Vec<PathBuf>>) {
    let mut counter: u8 = 0;
    // let h_guard = hmap.lock().unwrap();
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
