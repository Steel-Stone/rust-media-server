use std::{fs, path::PathBuf};

pub fn get_movie() -> std::io::Result<Vec<u8>> {
    let file_name = "example.m3u8";
    let path = std::format!("./videos/hls_converted/{}", file_name);
    let filepath = PathBuf::from(path);
    fs::read(&filepath)
}

pub fn play_movie(file_name: String) -> std::io::Result<Vec<u8>> {
    let path = format!("./videos/hls_converted/{}", file_name);
    let filepath = PathBuf::from(path);
    fs::read(&filepath)
}
