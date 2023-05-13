//more tauri stuff
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use std::sync::{Arc, Mutex};
use std::time::Instant;

use base64::{decode, encode};
use epub::doc::EpubDoc;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read, Write};
use std::path::Path;
use std::{env, fs, string};

#[derive(Serialize, Deserialize, Debug)]
struct Book {
    cover_location: String,
    book_location: String,
    title: String,
}
//method invoked in tauri
fn create_book_vec(items: &Vec<String>, write_directory: &String) -> Vec<Book> {
    let books: Vec<Book> = items
        .par_iter()
        .filter_map(|item| {
            let title = EpubDoc::new(&item).unwrap().mdata("title").unwrap();

            let new_book = Book {
                cover_location: create_cover(item.to_string(), write_directory),
                book_location: item.replace("\\", "/"),
                title,
            };

            Some(new_book)
        })
        .collect();

    let mut sorted_books = books;
    sorted_books.sort_by(|a, b| a.title.cmp(&b.title));

    sorted_books
}

fn create_covers(dir: String) -> Vec<Book> {
    //file name to long
    let paths = fs::read_dir(&dir);
    //  let mut books: Vec<Book> = Vec::new();
    let mut book_json: Vec<Book>;
    //Later: call dedupe earlier to avoid extra mapping
    let json_path = format!("{}/book_cache.json", &dir);
    let start_time = Instant::now();
    let epubs: Vec<String> = paths
        .unwrap()
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            if path.is_file() && path.extension().unwrap() == "epub" {
                Some(path.to_str().unwrap().to_owned())
            } else {
                None
            }
        })
        .collect();
    //Check if cache exists

    if Path::new(&json_path).exists() {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&json_path);
        book_json = match serde_json::from_reader(BufReader::new(file.unwrap())) {
            Ok(data) => data,
            Err(_) => Vec::new(),
        };
        let book_json_test = Arc::new(Mutex::new(book_json));

        epubs.par_iter().for_each(|item| {
            let item_normalized = item.replace("\\", "/");
            let title = EpubDoc::new(&item_normalized)
                .unwrap()
                .mdata("title")
                .unwrap();

            let mut book_json_guard = book_json_test.lock().unwrap();
            let index = chunk_binary_search_index(&book_json_guard, &title);

            if !index.is_none() {
                let new_book = Book {
                    cover_location: create_cover(item_normalized.to_string(), &dir),
                    book_location: item_normalized,
                    title: title.clone(),
                };
                book_json_guard.insert(index.unwrap(), new_book);
            }
        });

        book_json = Arc::try_unwrap(book_json_test)
            .unwrap()
            .into_inner()
            .unwrap();

        // epubs.iter().for_each(|item| {
        //     let item_normalized = item.replace("\\", "/");
        //     let title = EpubDoc::new(&item_normalized)
        //         .unwrap()
        //         .mdata("title")
        //         .unwrap();
        //     let index = chunk_binary_search_index(&book_json, &title);
        //     if !index.is_none() {
        //         let new_book = Book {
        //             cover_location: create_cover(item_normalized.to_string(), &dir),
        //             book_location: item_normalized,
        //             title: title.clone(),
        //         };

        //         book_json.insert(index.unwrap(), new_book);
        //     }
        // });
        // book_json.extend(create_book_vec(&epubs, &dir));
        // //Could this sort be done on the fly?
        // book_json.sort_by(|a, b| a.title.cmp(&b.title));
    } else {
        book_json = create_book_vec(&epubs, &dir);
    }

    let file = File::create(json_path).unwrap();
    serde_json::to_writer_pretty(file, &book_json);
    let elapsed_time = start_time.elapsed();
    println!("Execution time: {} ms", elapsed_time.as_millis());
    return book_json;
}

fn chunk_binary_search_index(dataset: &Vec<Book>, key: &String) -> Option<usize> {
    let title = key.to_string();
    //handel lower case
    let low = dataset.iter().position(|b| b.title[..1] == title[..1]);

    if let Some(index) = low {
        let mut high = dataset
            .iter()
            .rposition(|b| b.title[..1] == title[..1])
            .unwrap();
        let mut unwrapped_low = index;
        while unwrapped_low <= high {
            let mid = (unwrapped_low + high) / 2;
            if dataset[mid].title == title {
                return None;
            } else if dataset[mid].title < title {
                unwrapped_low = mid + 1;
            } else {
                high = mid - 1;
            }
        }
        Some(unwrapped_low)
    } else {
        Some(dataset.len())
    }
}
fn create_cover(book_directory: String, write_directory: &String) -> String {
    use rand::Rng;

    let mut rng = rand::thread_rng();
    //Bug could get two of the same number

    let random_num = rng.gen_range(0..=10000).to_string();
    let cover_path = format!("{}/covers/{}.jpg", &write_directory, random_num);
    let doc = EpubDoc::new(&book_directory);
    let mut doc = doc.unwrap();
    if let Some(cover) = doc.get_cover() {
        let cover_data = cover.0;
        let f = fs::File::create(&cover_path);
        let mut f = f.unwrap();
        if let Err(err) = f.write_all(&cover_data) {
            eprintln!("Failed to write cover data: {:?}", err);
            // handle the error in an appropriate way
        }
    }

    return cover_path;
}
//For tauri
fn base64_encode_file(file_path: String) -> String {
    let mut file = File::open(&file_path).expect("Failed to open file");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Failed to read file");

    // Encode the image data as base64
    let base64_data = encode(&buffer);
    return base64_data;
}

fn main() {
    println!(
        "{}/{}",
        env::current_dir()
            .unwrap()
            .to_string_lossy()
            .replace("\\", "/"),
        "sample_books"
    );
    let test = format!(
        "{}/{}",
        env::current_dir()
            .unwrap()
            .to_string_lossy()
            .replace("\\", "/"),
        "sample_books"
    );
    //dont delete covers folder lol
    //after you run it the first time the json file will be made
    //after adding the epub in the new book folder it will be at the end but it should be with the other "p" books
    create_covers(test);
    println!("{}", "done".to_owned());
}
