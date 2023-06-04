use std::process::Command;

pub fn transcode(directory_path: String, video_name: String) {
    let mut output = Command::new("ffmpeg")
        .arg("-i")
        .arg(std::format!("{}/{}", directory_path, video_name))
        .arg("-c:v")
        .arg("h264")
        .arg("+cgop")
        .arg("-g")
        .arg("30")
        .arg("-hls_time")
        .arg("1")
        .arg(std::format!("{}.m3u8", video_name))
        .spawn()
        .unwrap();
    let status = output.wait();
    log::info!("Exited with status {:?}", status);
}

