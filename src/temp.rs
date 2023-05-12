use epub::doc::EpubDoc;
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read, Write};
use std::path::Path;
use std::{fs, string};

#[derive(Serialize, Deserialize, Debug)]
struct Book {
    cover_location: String,
    title: String,
}

fn create_book_vec(items: &Vec<String>, write_directory: &String) -> Vec<Book> {
    let mut books: Vec<Book> = Vec::new();
    for item in items {
        let title = EpubDoc::new(&item).unwrap().mdata("title").unwrap();

        let new_book = Book {
            cover_location: create_cover(item.to_string(), &write_directory),
            title,
        };
        books.push(new_book);
    }
    books.sort_by(|a, b| a.title.cmp(&b.title));

    return books;
}
fn create_covers(dir: String) -> Vec<Book> {
    //file name to long
    let paths = fs::read_dir(&dir);
    // let mut books: Vec<Book> = Vec::new();
    let mut book_json: Vec<Book> = Vec::new();
    //Later: call dedupe earlier to avoid extra mapping
    let json_path = format!("{}/book_cache.json", &dir);
    let out_file: File;
    //Check if cache exists
    if Path::new(&json_path).exists() {
        println!("Here");
        out_file = File::open(&json_path).unwrap();
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&json_path);
        book_json = match serde_json::from_reader(BufReader::new(file.unwrap())) {
            Ok(data) => data,
            Err(_) => Vec::new(),
        };

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
            .filter_map(|b| chunk_binary_search(&book_json, b))
            .collect();
        for book in create_book_vec(&epubs, &dir) {
            book_json.push(book);
        }
    } else {
        println!("not Here");
        out_file = File::create(&json_path).unwrap();

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
        book_json = create_book_vec(&epubs, &dir);
    }

    //call dedupe function here

    serde_json::to_writer_pretty(&out_file, &book_json);

    return book_json;
}

fn chunk_binary_search(dataset: &Vec<Book>, key: String) -> Option<String> {
    let doc = EpubDoc::new(&key).unwrap();

    let title = doc.mdata("title").unwrap();
    let temp = format!("{:?}", dataset);
    println!("{}", &key);
    let mut low = dataset
        .iter()
        .position(|b| b.title[..1] == title[..1])
        .unwrap();
    let mut high = dataset
        .iter()
        .rposition(|b| b.title[..1] == title[..1])
        .unwrap();

    while low <= high {
        let mid = (low + high) / 2;
        if dataset[mid].title == title {
            return None;
        } else if dataset[mid].title < title {
            low = mid + 1;
        } else {
            high = mid - 1;
        }
    }

    Some(key)
}
fn create_cover(book_directory: String, write_directory: &String) -> String {
    use rand::Rng;

    let mut rng = rand::thread_rng();
    //Bug could get two of the same number
    let random_num = rng.gen_range(0..=10000).to_string();
    let doc = EpubDoc::new(&book_directory);
    let mut doc = doc.unwrap();
    let cover_data = doc.get_cover().unwrap();
    let cover_path = format!("{}/covers/{}.jpg", &write_directory, random_num);
    println!("{}", &cover_path);
    let f = fs::File::create(&cover_path);
    let mut f = f.unwrap();
    let resp = f.write_all(&cover_data.0);
    return cover_path;
}

fn main() {
    println!("Hello, world!");
    let temp = create_covers("E:/Documents/BookShare/DIY".to_string());
    println!("{}", "done".to_owned());
}
