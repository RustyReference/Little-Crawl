use little_crawl::*;
use crossbeam::channel;
use reqwest::blocking;
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::ops::Deref;

// Constants
const CHANNEL_CAP: usize = 100;
const MAX_URLS: usize = 10;
const NUM_THREADS: usize = 5;

fn main() {
    let urls = vec![
        "https://www.rust-lang.org",
        "https://www.wikipedia.org",
        "https://www.wikipedia.org",
    ];

    // Prepare thread handlers (for joining), channel, and shared visited pool
    let mut handlers = vec![];
    let (s, r) = channel::bounded::<String>(CHANNEL_CAP);
    let visited: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));

    // Create threads.
    for _ in 0..NUM_THREADS {
        let r = r.clone(); // For polling Urls from channel to process
        let s = s.clone(); // For sending new Urls into the channel
        let visited_t = Arc::clone(&visited);

        // Spawn thread
        let handler = thread::spawn(move || {
            for url in r.iter() {
                let mut visited = visited_t.lock().unwrap();
                
                println!("{}", url);
                // If number of visited URLs exceeds limit, stop fetching them.
                if visited.len() >= MAX_URLS {
                    break;     
                }

                if visited.contains(&url) {
                    continue;
                }

                // Fetch new links from the page at URL
                let new_links = fetch_links(&url).unwrap();
                for link in new_links {
                    s.send(link).unwrap();
                }

                (*visited).insert(url);
            }
        });

        handlers.push(handler);
    }

    // Send seed set of URLs to channel
    for url in urls {
        s.send(url.to_string()).unwrap();
    }

    drop(s); // Close the channel

    // Let the threads finish before exiting program
    for handler in handlers {
        handler.join().unwrap();
    }

    // Display all the URLs
    for url in visited.lock().unwrap().deref() {
        println!("{}", url);
    }
}


