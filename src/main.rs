use flate2::read::GzDecoder;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking;
use std::{error::Error, path::PathBuf};
use tar::Archive;
fn main() {
    download().unwrap();
}

fn download() -> Result<(), Box<dyn std::error::Error>> {
    let dest = PathBuf::from("./test/");
    let client = blocking::Client::new();
    // let url = "https://github.com/styxpilled/termdraw/archive/HEAD.tar.gz";
    let url = "https://github.com/seanmonstar/reqwest/archive/HEAD.tar.gz";
    let response = client.get(url).send()?;
    let total_size = response.content_length();

    let pb = match total_size {
        Some(x) => ProgressBar::with_style(
            ProgressBar::new(x),
            ProgressStyle::default_bar()
                .template(
                    "[{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}",
                )?
                .progress_chars("#>-"),
        ),
        None => ProgressBar::new_spinner(),
    };
    // The header is probably 512B but it can be larger so I'm not hardcoding it in.
    let tar = GzDecoder::new(pb.wrap_read(response));
    let mut archive = Archive::new(pb.wrap_read(tar));
    pb.wrap_iter(
        archive
            .entries()?
            .filter_map(|e| e.ok())
            .map(|mut entry| -> Result<PathBuf, Box<dyn Error>> {
                let path = entry.path()?;
                let path = path
                    .strip_prefix(path.components().next().unwrap())?
                    .to_owned();
                entry.unpack(dest.join(&path))?;
                Ok(path)
            })
            .filter_map(|e| e.ok()),
    )
    .for_each(drop);
    // while let Some(mut item) = response.chunk().await? {
    //     file.write_all_buf(item.borrow_mut()).await?;
    // }
    pb.finish_with_message("\nDone...");
    // pb.finish();
    Ok(())
}
