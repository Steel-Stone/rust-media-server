use std::process::Command;

pub fn transcode(directory_path: String, video_name: String) {
    list_file_details("/home/daniel/test2/Recording.mp4");
    let mut output = Command::new("ffmpeg")
        .arg("-i")
        // .arg(std::format!("{}/{}", directory_path, video_name))
        .arg("/home/daniel/test2/Recording.mp4".to_string())
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

pub fn list_file_details(file_path: &str) {
    let output = Command::new("ls")
        .arg("-l")
        .arg(file_path)
        .output()
        .expect("Failed to execute ls command");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("File details:\n{}", stdout);
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Error retrieving file details:\n{}", stderr);
    }
}



