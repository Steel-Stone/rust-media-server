pub async fn get_public_ip() -> String {
    reqwest::get("http://whatismyip.akamai.com/")
        .await
        .unwrap()
        .text()
        .await
        .unwrap()
}
