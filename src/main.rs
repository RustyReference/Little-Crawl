use crossbeam::channel;
use little_crawl::*;
use std::collections::HashSet;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

// Constants
const CHANNEL_CAP: usize = 10000;
const NUM_THREADS: usize = 5;

fn main() {
    let urls = vec![
        "https://www.rust-lang.org",
        "https://www.wikipedia.org",
    ];

    // Prepare thread handlers (for joining), channel, and shared visited pool
    let mut handlers = vec![];
    let (s, r) = channel::unbounded::<String>();
    let visited: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));

    // Create threads.
    for _ in 0..NUM_THREADS {
        let r = r.clone(); // For polling Urls from channel to process
        let s = s.clone(); // For sending new Urls into the channel
        let visited_t = Arc::clone(&visited);

        // Spawn thread
        let handler = spawn_thread(s, r, visited_t);

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
