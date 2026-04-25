use tiny_http::{Server, Response, Header};
use std::io::{Write, BufReader, BufRead};
use std::net::TcpStream;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct SessionData {
    username: String,
    role: String,
    created_at: String,
}

fn main() {
    let server = Server::http("0.0.0.0:6060").unwrap();
    println!("Ouroboros Auth Server running on http://localhost:6060");

    for request in server.incoming_requests() {
        let url = request.url().to_string();

        if url == "/" {
            let html = include_str!("auth_index.html");
            let response = Response::from_string(html)
                .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap());
            request.respond(response).unwrap();
        } else if url.starts_with("/api/login") {
            // In a real app, you'd check password here. 
            // We just create a session for any username.
            if let Some((_, query)) = url.split_once('?') {
                let username = query.split('=').nth(1).unwrap_or("anon");
                let session_id = Uuid::new_v4().to_string();
                let data = SessionData {
                    username: username.to_string(),
                    role: "user".to_string(),
                    created_at: chrono::Local::now().to_rfc3339(),
                };
                
                let json = serde_json::to_string(&data).unwrap();
                let mesh_key = format!("auth:session:{}", session_id);
                
                if execute_mesh_cmd(&format!("SET {} {}", mesh_key, json)) {
                    let resp_json = format!("{{\"token\": \"{}\"}}", session_id);
                    request.respond(Response::from_string(resp_json)).unwrap();
                } else {
                    request.respond(Response::from_string("Mesh Error").with_status_code(500)).unwrap();
                }
            }
        } else if url.starts_with("/api/validate") {
            if let Some((_, token)) = url.split_once('=') {
                let mesh_key = format!("auth:session:{}", token);
                if let Some(data) = get_mesh_data(&mesh_key) {
                    request.respond(Response::from_string(data)).unwrap();
                } else {
                    request.respond(Response::from_string("Invalid Session").with_status_code(401)).unwrap();
                }
            }
        } else if url.starts_with("/api/sessions") {
            let sessions = list_all_sessions();
            request.respond(Response::from_string(sessions)).unwrap();
        } else if url.starts_with("/api/logout") {
            if let Some((_, token)) = url.split_once('=') {
                let mesh_key = format!("auth:session:{}", token);
                execute_mesh_cmd(&format!("DEL {}", mesh_key));
                request.respond(Response::from_string("Logged out")).unwrap();
            }
        }
    }
}

fn execute_mesh_cmd(cmd: &str) -> bool {
    if let Ok(mut stream) = TcpStream::connect("127.0.0.1:8825") {
        let _ = stream.write_all(format!("{}\n", cmd).as_bytes());
        return true;
    }
    false
}

fn get_mesh_data(key: &str) -> Option<String> {
    if let Ok(mut stream) = TcpStream::connect("127.0.0.1:8825") {
        let _ = stream.write_all(format!("GET {}\n", key).as_bytes());
        let mut reader = BufReader::new(&stream);
        let mut line = String::new();
        if reader.read_line(&mut line).is_ok() && line.starts_with('+') {
            return Some(line[1..].trim().to_string());
        }
    }
    None
}

fn list_all_sessions() -> String {
    let mut sessions = Vec::new();
    if let Ok(mut stream) = TcpStream::connect("127.0.0.1:8825") {
        let _ = stream.write_all(b"KEYS auth:session:*\n");
        let mut reader = BufReader::new(&stream);
        let mut line = String::new();
        if reader.read_line(&mut line).is_ok() && line.starts_with('*') {
            let count: usize = line.trim()[1..].parse().unwrap_or(0);
            for _ in 0..count {
                let mut l1 = String::new(); reader.read_line(&mut l1).unwrap();
                let mut l2 = String::new(); reader.read_line(&mut l2).unwrap();
                let key = l2.trim();
                if let Some(data) = get_mesh_data(key) {
                    sessions.push(data);
                }
            }
        }
    }
    format!("[{}]", sessions.join(","))
}
