use crossbeam::channel;
use reqwest::blocking;
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

// Constants
const MAX_URLS: usize = 5;
const NUM_THREADS: usize = 5;

fn main() {
    let urls = vec![
        "https://www.rust-lang.org",
        "https://www.wikipedia.org",
        "https://www.wikipedia.org",
    ];

    // Prepare thread handlers (for joining), channel, and shared visited pool
    let mut handlers = vec![];
    let (s, r) = channel::bounded::<String>(MAX_URLS);
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

    for handler in handlers {
        // Let the threads finish before exiting program
        handler.join().unwrap();
    }
}

/// Takes a url, acquires links from it, and returns a vector of URLs
/// to enqueue onto the channel, wrapped in a result.
///
/// url: the url to fetch links from
fn fetch_links(url: &str) -> reqwest::Result<Vec<String>> {
    let body = blocking::get(url)?.text()?; // Gets html of each page
    println!("Fetched: {}\n\n{}\n\n\n\n", url, body);

    let mut urls: Vec<String> = vec![];

    // Getting links
    let document = Html::parse_document(&body);
    let selector = Selector::parse("a").unwrap();
    for node in document.select(&selector) {
        let url = node.attr("href").unwrap(); // Link inside `href` attr
        urls.push(url.to_string());
    }

    Ok(urls)
}
