use std::fs::File;
use std::io::{prelude::*, Bytes};
use std::fs;
use std::io::BufReader;
use std::time::Instant;
use std::path::Path;
use futures::stream::StreamExt;
use reqwest::Client;
use indicatif::{ProgressBar, ProgressStyle};
use tar::Archive;
use flate2::read::GzDecoder;

pub async fn download_and_extract(url: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 创建一个 Reqwest 客户端
    let client = Client::new();

    // 下载文件
    let res = client.get(url).send().await?;

    // 获取文件大小
    let total_size = res.content_length().ok_or("Failed to get content length")?;

    // 设置进度条
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        .progress_chars("#>-"));
    pb.set_message(&format!("Downloading {}", url));

    // 创建一个文件用于保存下载的文件
    let mut file = File::create(output_path)?;

    // 读取响应流并写入文件
    let mut stream = res.bytes_stream();
    let mut downloaded: u64 = 0;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk)?;
        let new = downloaded + (chunk.len() as u64);
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish_with_message(&format!("Downloaded {} to {}", url, output_path));

    // 解压文件
    let file = File::open(output_path)?;
    let mut archive = Archive::new(GzDecoder::new(file));
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        let dest = Path::new(output_path).join(path);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        entry.unpack(dest)?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://github.com/zhengzhang01/Pixel-GS/archive/refs/heads/main.zip";
    let output_path = "temp.zip";

    download_and_extract(url, output_path).await?;

    Ok(())
}