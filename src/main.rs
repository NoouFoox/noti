use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    time::Duration,
};

use notify_rust::Notification;
use percent_encoding::percent_decode;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("0.0.0.0:5988")?;
    for stream in listener.incoming() {
        let stream = stream?;
        handle_client(stream);
    }
    Ok(())
}
fn handle_client(mut stream: TcpStream) {
    let buffer = BufReader::new(&stream);
    let req_line = buffer.lines().next().unwrap().unwrap();
    if let Some(start) = req_line.find("GET /msg") {
        if let Some(end) = req_line.find(" HTTP/1.1") {
            let url = &req_line[start + 4..end];
            if let Some(query_start) = url.find('?') {
                let query_str = &url[query_start + 1..];
                let params: Vec<(&str, &str)> = query_str
                    .split('&')
                    .filter_map(|pair| {
                        let mut split = pair.split('=');
                        if let (Some(key), Some(value)) = (split.next(), split.next()) {
                            Some((key, value))
                        } else {
                            None
                        }
                    })
                    .collect();
                let mut title = "通知⚠️".to_string();
                let mut msg = "通知".to_string();
                for (key, value) in params {
                    match de_code(value) {
                        Ok(decode_value) => match key {
                            "title" => title = decode_value,
                            "msg" => msg = decode_value,
                            _ => {}
                        },
                        Err(_) => {
                            continue;
                        }
                    }
                    send_notification(&title, &msg).unwrap();
                }
                let response = "HTTP/1.1 200 OK\r\n\r\n";
                stream.write_all(response.as_bytes()).unwrap();
            }
        }
    }
}
fn send_notification(title: &str, txt: &str) -> Result<(), Box<dyn std::error::Error>> {
    Notification::new()
        .summary(title)
        .body(txt)
        .timeout(Duration::from_secs(10))
        .show()
        .expect("显示通知失败");
    Ok(())
}
fn de_code(s: &str) -> Result<String, Box<dyn std::error::Error>> {
    Ok(percent_decode(s.as_bytes()).decode_utf8()?.to_string())
}
