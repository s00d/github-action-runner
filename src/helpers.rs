use std::io::{Cursor, Read};
#[cfg(feature = "rodio")]
use std::io::BufReader;
use zip::ZipArchive;
use std::sync::Arc;
use std::time::Duration;
use indicatif::ProgressBar;
#[cfg(feature = "rodio")]
use rodio::{Decoder, OutputStream, Sink};
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

#[cfg(feature = "rodio")]
pub(crate) fn beep(count: u8) {
    let (_stream, handle) = OutputStream::try_default().unwrap();
    let beep_mp3_data = include_bytes!("../beep.mp3").to_vec();
    for _ in 0..count {
        let cursor = Cursor::new(beep_mp3_data.clone());
        match Decoder::new(BufReader::new(cursor)) {
            Ok(source) => {
                let sink = Sink::try_new(&handle).unwrap();
                sink.append(source);
                sink.sleep_until_end();
            }
            Err(_e) => {
                // Если декодирование не удалось, просто вернуться из функции
                return;
            }
        }
    }
}

#[cfg(not(feature = "rodio"))]
pub(crate) fn beep(_count: u8) {}
