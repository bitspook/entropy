use reqwest::{self, header};
use tokio::{
    self,
    sync::mpsc::{self, Receiver, Sender},
};

mod meetup;

#[derive(Debug)]
pub enum ScraperMessage<'a> {
    ResultItem(meetup::MeetupGroup),
    Error(meetup::Error<'a>),
}

#[tokio::main]
async fn main() -> Result<(), meetup::Error<'static>> {
    let user_agent = "Mozilla/5.0 (X11; Linux x86_64; rv:88.0) Gecko/20100101 Firefox/88.0";
    let mut default_headers = header::HeaderMap::new();
    default_headers.insert("Accept", header::HeaderValue::from_static("*/*"));

    let client = reqwest::Client::builder()
        .user_agent(user_agent)
        .build()
        .map_err(|err| meetup::Error::HttpError(err))?;

    let (tx, mut rx): (Sender<ScraperMessage>, Receiver<ScraperMessage>) = mpsc::channel(100);

    tokio::spawn(async move {
        if let Err(err) = meetup::search_groups(&client, tx.clone()).await {
            println!("Encountered fatal error when searching for groups: {:#?}", err);
        };
    });

    while let Some(group) = rx.recv().await {
        match group {
            ScraperMessage::Error(err) => {
                println!("Encountered error when searching groups: {:#?}", err)
            }
            ScraperMessage::ResultItem(group) => {
                println!("Found group: {:#?}", group);
            }
        }
    };

    println!("Done!");

    Ok(())
}
