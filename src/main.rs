use crossbeam::channel;
use reqwest::blocking;
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::string;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::thread;

// Experimentation imports
use std::fs::*;
use std::io::prelude::*;
use std::usize::MAX;

// Constants
const MAX_URLS: usize = 5;
const NUM_THREADS: usize = 5;

fn main() {
    let urls = vec![
            "https://www.rust-lang.org",
            "https://www.wikipedia.org",
            "https://www.wikipedia.org",
        ];
    
        let mut handlers = vec![];
        let (s, r) = channel::bounded::<String>(MAX_URLS);
    
        let visited: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
        for i in 0..NUM_THREADS {
            let r = r.clone();
            let visited_t = Arc::clone(&visited);
            let handler = thread::spawn(move || {
                for url in r.iter() {
                    let mut visited = visited_t.lock().unwrap();
                    if visited.contains(&url) {
                        continue;
                    }
    
                    let _ = fetch_and_print_links(&url);
                    (*visited).insert(url);
                }
            });
    
            handlers.push(handler);
        }
    
        for url in urls {
            s.send(url.to_string()).unwrap();
        }
    
        drop(s); // Close the channel
    
        for handler in handlers {
            // Let the threads finish before exiting program
            handler.join().unwrap();
        }
}

fn fetch_and_print_links(url: &str) -> reqwest::Result<()> {
    let body = reqwest::blocking::get(url)?.text()?; // Gets html of each page
    println!("Fetched: {}\n\n{}\n\n\n\n", url, body);

    // Parsing...?

    Ok(())
}


