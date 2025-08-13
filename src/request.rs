use crate::Error;
use crate::XkcdComic;
pub async fn get_json(url: &str) -> Result<XkcdComic, Error> {
    let response = reqwest::get(url).await?.json::<XkcdComic>().await?;
    Ok(response)
}

#[derive(Debug, Error)]
pub enum FetchComicError {
    #[error("Failed to fetch comic {index} after max attempts: {source}")]
    MaxAttemptsReached {
        index: u32,
        #[source]
        source: reqwest::Error,
    },
}
pub async fn fetch_comic(index: u32) -> Result<XkcdComic, FetchComicError> {
    let mut failed_attempts = 0;
    let max_attempts = 3;
    let url = match index {
        0 => "https://xkcd.com/info.0.json".to_owned(),
        _ => format!("https://xkcd.com/{index}/info.0.json"),
    };
    loop {
        match get_json(&url).await {
            Ok(comic) => return Ok(comic),
            Err(error) => {
                log_error!("Failed to fetch comic {}: {}", index, error);
                failed_attempts += 1;
                if failed_attempts >= max_attempts {
                    log_error!(
                        "Reached max failed attempts ({}) while fetching comics. Stopping fetches",
                        max_attempts
                    );
                    return Err(FetchComicError::MaxAttemptsReached {
                        index,
                        source: error,
                    });
                }
                continue;
            }
        }
    }
}
