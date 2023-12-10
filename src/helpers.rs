use std::io::{Cursor, Read};
use zip::ZipArchive;
use std::sync::Arc;
use std::time::Duration;
use indicatif::ProgressBar;
use tokio::sync::Mutex;

pub fn unzip_and_concatenate(data_bytes: Vec<u8>) -> Result<String, Box<dyn std::error::Error>> {
    let cursor = Cursor::new(data_bytes);
    let mut archive = ZipArchive::new(cursor)?;

    let mut result = String::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let file_name = file.name().to_string();

        // Пропустить файлы в поддиректориях, пока не обработаем все файлы в корне
        if file_name.contains("/") {
            continue;
        }

        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        result.push_str(&contents);
    }

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let file_name = file.name().to_string();

        // Теперь обрабатываем только файлы в поддиректориях
        if !file_name.contains("/") {
            continue;
        }

        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        result.push_str("\n");
        result.push_str("--------------\n");
        result.push_str(&file_name);
        result.push_str("\n--------------\n");
        result.push_str(&contents);
    }

    Ok(result)
}

pub(crate) async fn update_progress_bar(pb: Arc<Mutex<ProgressBar>>) {
    loop {
        {
            let pb = pb.lock().await;
            pb.tick();
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}