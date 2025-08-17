#[cfg(test)]
mod tests {
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
    use super::*;

    // Constants
    const MAX_URLS: usize = 5;
    const NUM_THREADS: usize = 5;

    /// Sends a GET request to the given URL and writes the URL to a file
    /// 
    /// NOTE: This is a helper function for `test_multith_crawl()`
    fn fetch_and_print_links_test(url: &str, mut file: MutexGuard<'_, File>) -> reqwest::Result<()> {
        let _body = reqwest::blocking::get(url)?.text()?; // Gets html of each page
    
        // TEST: write html to file
        let msg = format!("Fetched: {}\n", url);
        file.write_all(msg.as_bytes()).unwrap();
    
        Ok(())
    }

    /// Checks test.txt for only FOUR (4) "Fetched: <URL>" lines. 
    /// That would mean the duplicate URL was not used, which gives a 
    /// passing test.
    fn count_urls(num_uniq_urls: usize) {
        let mut file = File::open("test.txt").unwrap();
        let mut buf = String::new();
        let _ = file.read_to_string(&mut buf).unwrap();

        let mut num_lines = 0;
        for _ in buf.lines() {
            num_lines += 1;
        }
        
        assert_eq!(num_lines, num_uniq_urls);
    }

    /// Runs a quick multithreaded webcrawl of URLs to fetch HTML and 
    /// appends the HTML to a file. This is to test if the threads will 
    /// include any duplicates of the URLs. 
    /// 
    /// This function is intended to mimick the main function or an earlier
    /// version of it by taking a predefined set of URLs and spawning threads
    /// to crawl through them. 
    /// 
    /// POSTCONDITION: The file should contain "Fetched: <URL>", where <URL> 
    ///     is the URL of the page, followed by two new lines and  the HTML.
    #[test]
    fn test_multith_crawl() {
        let urls = vec![
            "https://www.rust-lang.org",
            "https://www.wikipedia.org",
            "https://www.wikipedia.org",
            "https://google.com",
            "https://google.com",
            "https://github.com",
            "https://leetcode.com",
            "https://www.onlinegdb.com"
        ];

        let mut url_set = HashSet::new();
        for url in &urls {
            url_set.insert(url);
        }
        
        let num_uniq_urls = url_set.len();
    
        let mut handlers = vec![];
        let (s, r) = channel::bounded::<String>(MAX_URLS);
    
        // TEST: File object
        let file_mut = Arc::new(Mutex::new(
            File::create("test.txt").unwrap()
        ));
    
        let visited: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
        for _ in 0..NUM_THREADS {
            let r = r.clone();
            let file_mut_t = Arc::clone(&file_mut);
            let visited_t = Arc::clone(&visited);
            let handler = thread::spawn(move || {
                for url in r.iter() {
                    let mut visited = visited_t.lock().unwrap();
                    let file_mut = file_mut_t.lock().unwrap();
                    if visited.contains(&url) {
                        continue;
                    }
    
                    let _ = fetch_and_print_links_test(&url, file_mut);
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

        count_urls(num_uniq_urls);

        let _ = remove_file("test.txt");
    }
    
    #[test]
    fn attempt() {
        let input = String::from("a");

        let html = r#"
            <!DOCTYPE html>
            <meta charset="utf-8">
            <html>
                <head>
                    <title>Hello, world!</title>
                </head>
                <body>
                    <h1 class="foo">Hello, <i>world!</i></h1>
                    <a href="https://www.w3schools.com/">Visit W3Schools.com!</a>
                    <a href="https://google.com">Google</a>
                    <div>
                        <a href="https://neetcode.com">Neetcode</a>
                    </div>
                </body>
            </html>
        "#;
        let document = Html::parse_document(html);
        let selector = Selector::parse(&input).unwrap();
        for node in document.select(&selector) {
            let formatted = format!("Value: {}", node.attr("href").unwrap());
            println!("\nBEGIN TEST\n{}\nEND TEST\n", formatted);
        }
    }
}