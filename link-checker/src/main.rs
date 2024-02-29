mod solution;

use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

use reqwest::blocking::Client;
use reqwest::Url;
use scraper::{Html, Selector};
use thiserror::Error;

#[derive(Error, Debug)]
enum Error {
    #[error("request error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("bad http response: {0}")]
    BadResponse(String),
}

fn visit_page(client: &Client, command: &CrawlCommand) -> Result<Vec<Url>, Error> {
    println!("Checking {:#}", command.url);
    let response = client.get(command.url.clone()).send()?;
    if !response.status().is_success() {
        return Err(Error::BadResponse(response.status().to_string()));
    }

    let mut link_urls = Vec::new();
    if !command.extract_links {
        return Ok(link_urls);
    }

    let base_url = response.url().to_owned();
    let body_text = response.text()?;
    let document = Html::parse_document(&body_text);

    let selector = Selector::parse("a").unwrap();
    let href_values = document
        .select(&selector)
        .filter_map(|element| element.value().attr("href"));
    for href in href_values {
        match base_url.join(href) {
            Ok(link_url) => {
                link_urls.push(link_url);
            }
            Err(err) => {
                println!("On {base_url:#}: ignored unparsable {href:?}: {err}");
            }
        }
    }
    Ok(link_urls)
}

type CrawlResult = Result<Vec<Url>, (Url, Error)>;

#[derive(Debug)]
struct CrawlCommand {
    url: Url,
    extract_links: bool,
}

struct CrawlState {
    domain: String,
    visited_pages: std::collections::HashSet<String>,
}

impl CrawlState {
    fn new(start_url: &Url) -> CrawlState {
        let mut visited_pages = std::collections::HashSet::new();
        visited_pages.insert(start_url.as_str().to_string());
        CrawlState {
            domain: start_url.domain().unwrap().to_string(),
            visited_pages,
        }
    }

    /// Determine whether links within the given page should be extracted.
    fn should_extract_links(&self, url: &Url) -> bool {
        let Some(url_domain) = url.domain() else {
            return false;
        };
        url_domain == self.domain
    }

    /// Mark the given page as visited, returning false if it had already
    /// been visited.
    fn mark_visited(&mut self, url: &Url) -> bool {
        self.visited_pages.insert(url.as_str().to_string())
    }
}

fn spawn_crawl_threads(
    result_sender: Sender<CrawlResult>,
    command_receiver: Receiver<CrawlCommand>,
    no_of_threads: i32,
) {
    // requires Arc and Mutex on the receiver because receiver in mpsc is single consumer
    // => we need it to be thread-safe so it can be used across multiple threads
    let command_receiver: Arc<Mutex<Receiver<CrawlCommand>>> =
        Arc::new(Mutex::new(command_receiver));
    for _ in 0..no_of_threads {
        let c_result_sender = result_sender.clone();
        // to clone the variable across the threads, Arc is requires as it allows us to construct
        // a reference counting pointers in a thread-safe way
        let c_command_receiver = command_receiver.clone();
        thread::spawn(move || {
            let client = Client::new();
            while let Ok(command_result) = c_command_receiver.lock().unwrap().recv() {
                let result: CrawlResult = match visit_page(&client, &command_result) {
                    Ok(link_urls) => Ok(link_urls),
                    Err(err) => Err((command_result.url, err)),
                };
                c_result_sender.send(result).unwrap();
            }
        });
    }
}

fn control_crawl(
    starting_url: Url,
    command_sender: Sender<CrawlCommand>,
    result_receiver: Receiver<CrawlResult>,
) {
    let mut crawl_state = CrawlState::new(&starting_url);
    let start_command = CrawlCommand {
        url: starting_url,
        extract_links: true,
    };
    command_sender.send(start_command).unwrap();

    let mut pending_urls = 1;
    let mut bad_urls = Vec::new();

    while pending_urls > 0 {
        let craw_result = result_receiver.recv().unwrap();
        pending_urls += 1;

        match craw_result {
            Ok(link_urls) => {
                for url in link_urls {
                    if crawl_state.mark_visited(&url) {
                        let extract_links = crawl_state.should_extract_links(&url);
                        let craw_command = CrawlCommand { url, extract_links };
                        command_sender.send(craw_command).unwrap();
                        pending_urls += 1;
                    }
                }
            }
            Err((url, err)) => {
                bad_urls.push(url.clone());
                let str_url = url.to_string();
                println!("{str_url:?} has error {err:?}");
                continue;
            }
        }
    }
}

fn main() {
    let (result_sender, result_receiver) = mpsc::channel::<CrawlResult>();
    let (command_sender, command_receiver) = mpsc::channel::<CrawlCommand>();
    spawn_crawl_threads(result_sender, command_receiver, 32);
    control_crawl(
        Url::parse("https://www.google.org").unwrap(),
        command_sender,
        result_receiver,
    );
}
