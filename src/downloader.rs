use std::cmp::min;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};

use eframe::epaint::ahash::HashMap;
use futures_util::StreamExt;
use regex::Regex;
use reqwest::Client;

pub mod ui;

#[derive(Clone)]
struct Downloader {
    client: Client,
    title_pattern: Regex,
    download_progress: Arc<Mutex<f32>>
}

pub struct PatchPackage {
    version: String,
    size: String,
    sha1: String,
    url: String,
    ps3_version: String,
}

pub struct PatchTitle {
    title: String,
    packages: HashMap<String, PatchPackage>,
}

impl Default for Downloader {
    fn default() -> Self {
        let client = Client::builder().danger_accept_invalid_certs(true)
            .build()
            .unwrap();

        let title_pattern = Regex::new("<TITLE>(.+)</TITLE>").unwrap();

        Self {
            client,
            title_pattern,
            download_progress: Arc::new(Mutex::new(0.0)),
        }
    }
}

impl Downloader {
    pub fn extract_info(&self, xml: &str) -> Option<PatchTitle> {
        let mut patch = PatchTitle {
            title: "".to_string(),
            packages: Default::default(),
        };

        let title_capture = self.title_pattern.captures(xml).unwrap();

        if let Some(title) = title_capture.get(1) {
            patch.title = title.as_str().to_string();
        } else {
            return None;
        }


        let document = roxmltree::Document::parse(xml).unwrap();
        for des in document.descendants().enumerate() {
            if let Some(package) = des.1.descendants().find(|n| n.tag_name().name() == "package") {
                let version = package.attribute("version").map_or("N/A".to_string(), |m| m.parse().unwrap());
                let package = PatchPackage {
                    version: version.clone(),
                    size: package.attribute("size").map_or("N/A".to_string(), |m| m.parse().unwrap()),
                    sha1: package.attribute("sha1sum").map_or("N/A".to_string(), |m| m.parse().unwrap()),
                    url: package.attribute("url").map_or("N/A".to_string(), |m| m.parse().unwrap()),
                    ps3_version: package.attribute("ps3_system_ver").map_or("N/A".to_string(), |m| m.parse().unwrap()),
                };

                patch.packages.insert(version, package);
            }
        }

        Some(patch)
    }

    pub fn find(&self, serial: String) -> Result<PatchTitle, Box<dyn Error>> {
        let result: Result<String, reqwest::Error> = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let res = self.client.get(format!("https://a0.ww.np.dl.playstation.net/tpl/np/{0}/{0}-ver.xml", serial))
                    .send().await?.text().await?;
                Ok(res)
            });
        let res = result?;
        if let Some(p_title) = self.extract_info(res.as_str()) {
            return Ok(p_title);
        } else {
            Err("Cock")?
        }
    }

    pub fn download_file(&self, filename: String, url: String) {
        let _ = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let res = self.client.get(url.clone())
                    .send()
                    .await
                    .or(Err(format!("Failed to download file from {}", &url)))
                    .unwrap();

                let total_size = res
                    .content_length()
                    .ok_or(format!("Failed to get content length from '{}'", &url)).unwrap();

                // download chunks
                let mut file = File::create(filename.clone()).or(Err(format!("Failed to create file '{}'", filename))).unwrap();
                let mut downloaded: u64 = 0;
                let mut stream = res.bytes_stream();

                while let Some(item) = stream.next().await {
                    let chunk = item.or(Err(format!("Error while downloading file"))).unwrap();
                    file.write_all(&chunk)
                        .or(Err(format!("Error while writing to file"))).unwrap();
                    let new = min(downloaded + (chunk.len() as u64), total_size);
                    downloaded = new;
                    *self.download_progress.lock().unwrap() = (new as f32/total_size as f32) as f32;
                    // println!("{:.3}", ;
                }

                *self.download_progress.lock().unwrap() = 0.0;
            });
    }
}