use core::time;
use serde::Deserialize;
use std::{
    fs,
    io::{ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

use base64::{engine::general_purpose, Engine};
use sha1::{Digest, Sha1};
use std::rc::Rc;

#[derive(Copy, Clone)]
enum PacketType {
    Open,
    Close,
    Ping,
    Pong,
    Message,
}

impl PacketType {
    fn to_u8(&self) -> u8 {
        *self as u8
    }

    fn from_number(num: u8) -> Option<PacketType> {
        match num {
            0 => Some(PacketType::Open),
            1 => Some(PacketType::Close),
            2 => Some(PacketType::Ping),
            3 => Some(PacketType::Pong),
            4 => Some(PacketType::Message),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct DirPair {
    dir1: String,
    dir2: String,
}

impl DirPair {
    fn parse(text: &str) -> Self {
        let parts: Vec<&str> = text.split(";").collect();
        Self {
            dir1: parts[0].split(":").last().unwrap().to_string(),
            dir2: parts[1].split(":").last().unwrap().to_string(),
        }
    }
}

enum Command {
    Search(DirPair),
    Prev(usize),
    Next(usize),
}

impl Command {
    fn parse(content: String) -> Self {
        let parts: Vec<&str> = content.splitn(2, ";").collect();
        let comm = parts[0].split(":").last().unwrap();
        match comm {
            "search" => Command::Search(DirPair::parse(parts[1])),
            "prev" => Command::Prev(parts[1].parse::<usize>().unwrap() - 1),
            "next" => Command::Next(parts[1].parse::<usize>().unwrap() - 1),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct Duplicate {
    file_name: String,
    first_dir_match: String,
    second_dir_match: Rc<Vec<String>>,
}

struct CompareWindow {
    files: Rc<Vec<String>>,
    index: usize,
    to_update: bool,
}

impl CompareWindow {
    fn new(files: Rc<Vec<String>>) -> Self {
        Self {
            files,
            index: 0,
            to_update: true,
        }
    }

    fn change_index(&mut self, direction: isize) {
        // wrap around?
        if 0 < self.index && direction < 0 {
            self.index -= 1;
            self.to_update = true;
        } else if self.index < self.files.len() - 1 && direction > 0 {
            self.index += 1;
            self.to_update = true;
        }
    }

    fn reset(&mut self, files: Rc<Vec<String>>) {
        self.files = files;
        self.index = 0;
        self.to_update = true;
    }

    fn get_updated_file(&self) -> Option<String> {
        if self.to_update {
            Some(self.files[self.index].clone())
        } else {
            None
        }
    }
}

struct CompareManager {
    duplicates: Rc<Vec<Duplicate>>,
    compare_windows: [CompareWindow; 2],
}

impl CompareManager {
    fn new(dups: Vec<Duplicate>) -> Self {
        let duplicates = Rc::new(dups);
        let temp: Vec<String> = duplicates
            .iter()
            .map(|d| d.first_dir_match.clone())
            .collect();
        let first_dir_match_rc: Rc<Vec<String>> = Rc::new(temp);
        let compare_windows = [
            CompareWindow::new(Rc::clone(&first_dir_match_rc)),
            CompareWindow::new(Rc::clone(&duplicates[0].second_dir_match)),
        ];
        Self {
            duplicates,
            compare_windows,
        }
    }

    fn change_file(&mut self, window_idx: usize, direction: isize) {
        for win in self.compare_windows.iter_mut() {
            win.to_update = false;
        }

        self.compare_windows[window_idx].change_index(direction);
        if window_idx == 0 {
            self.compare_windows[1].reset(Rc::clone(
                &self.duplicates[self.compare_windows[0].index].second_dir_match,
            ));
        }
    }

    fn get_updated_files(&mut self) -> Vec<Option<String>> {
        self.compare_windows
            .iter_mut()
            .map(|win| win.get_updated_file())
            .collect()
    }
}

fn get_payload_size(frame_data: &[u8], byte_idx: usize, byte_count: usize) -> u64 {
    // most significant must be 0
    let total_bits = byte_count * 8;
    let mut new_size = 0u64;
    for i in 0..byte_count {
        new_size |= (frame_data[i + byte_idx] as u64) << (total_bits - 8 * i);
    }
    return new_size;
}

fn perform_handshake(buf: &Vec<u8>, stream: &mut TcpStream) -> bool {
    let mut headers = [httparse::EMPTY_HEADER; 13];
    let mut req = httparse::Request::new(&mut headers);
    match req.parse(buf) {
        Ok(offset) => {
            let ws_key = match headers.iter().find(|v| v.name == "Sec-WebSocket-Key") {
                Some(val) => val.value,
                None => b"0",
            };
            let magic = b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
            let mut hasher = Sha1::new();
            hasher.update(ws_key);
            hasher.update(magic);
            let result = hasher.finalize();

            let result_base64 = general_purpose::STANDARD.encode(result);
            let handshake_response = format!(
                "HTTP/1.1 101 Switching Protocols\r\n\
                               Upgrade: websocket\r\n\
                               Connection: Upgrade\r\n\
                               Sec-WebSocket-Accept: {}\r\n\
                               Access-Control-Allow-Origin: *\r\n\r\n",
                result_base64
            );
            match stream.write(handshake_response.as_bytes()) {
                Ok(r) => {
                    eprintln!("handshake_response written: {r:?}")
                }
                Err(e) => {
                    eprintln!("handshake_response err: {e}")
                }
            }

            let handshake_payload = format!(
                "{}{{\"sid\":\"9SdNm98ck4q-QicDAAAA\",\
                              \"upgrades\":[],\
                              \"pingInterval\":30000,\
                              \"pingTimeout\":20000,\
                              \"maxPayload\":1000000}}",
                PacketType::Open.to_u8()
            );
            let mut handshake_final: Vec<u8> = Vec::new();
            handshake_final.extend([129, 107]);
            handshake_final.extend_from_slice(handshake_payload.as_bytes());
            match stream.write(&handshake_final) {
                Ok(r) => {
                    eprintln!("written {r:?}");
                    true
                }
                Err(e) => {
                    eprintln!("write err {e}");
                    false
                }
            }
        }
        Err(e) => {
            eprintln!("Parse err: {e}");
            false
        }
    }
}

fn handle_message(
    frame_data: &[u8],
    stream: &mut TcpStream,
    compare_manager: &mut Option<CompareManager>,
) {
    let mut offset = 6;
    let mut byte_idx = 0;
    let fin: u8 = frame_data[byte_idx] & 0x80;
    let rsv: u8 = frame_data[byte_idx] & 0x70;
    let opcode: u8 = frame_data[byte_idx] & 0x0F;
    byte_idx += 1;
    let mask_set: u8 = frame_data[byte_idx] & 0x80;
    let mut payload_size: u64 = (frame_data[byte_idx] & 0x7F).into();
    byte_idx += 1;
    match payload_size {
        126 => {
            let bytes_to_read = 2;
            payload_size = get_payload_size(frame_data, byte_idx, bytes_to_read);
            offset += bytes_to_read;
        }
        127 => {
            let bytes_to_read = 8;
            payload_size = get_payload_size(frame_data, byte_idx, bytes_to_read);
            offset += bytes_to_read;
        }
        _ => (),
    }
    let mask: &[u8] = &frame_data[offset - 4..offset];

    let decoded_payload: Vec<u8> = frame_data
        .iter()
        .skip(offset)
        .enumerate()
        .map(|(i, x)| {
            return x ^ mask[i % 4];
        })
        .collect();

    if decoded_payload[0] - 48 == PacketType::Message.to_u8() {
        let received_text = String::from_utf8_lossy(&decoded_payload);
        println!("payload as text: {:?}", received_text);
        let command = Command::parse(received_text.to_string());
        match command {
            Command::Search(dirs) => {
                let spawned_proc =
                    std::process::Command::new("../dir_diff/target/release/dir_diff")
                        .arg(dirs.dir1)
                        //.arg(dirs.dir2)
                        .spawn();
                match spawned_proc {
                    Ok(mut child_proc) => {
                        child_proc.wait().expect("Command wasn't running");
                        let dups_json = fs::read_to_string("./duplicates.json").unwrap();
                        let duplicates = serde_json::from_str(&dups_json).unwrap();
                        *compare_manager = Some(CompareManager::new(duplicates));
                    }
                    Err(e) => eprintln!("Process failed to start: {e}"),
                }
            }
            Command::Prev(idx) => {
                if let Some(ref mut cm) = compare_manager {
                    cm.change_file(idx, -1);
                }
            }
            Command::Next(idx) => {
                if let Some(ref mut cm) = compare_manager {
                    cm.change_file(idx, 1);
                }
            }
        };

        if let Some(ref mut cm) = compare_manager {
            let files = cm.get_updated_files();
            println!("files to rerender {:?}", files);
            for (idx, file) in files.into_iter().enumerate() {
                if let Some(f) = file {
                    let payload_to_send = format!("4file{}{}", idx + 1, f);
                    let mut bytes_to_send: Vec<u8> = Vec::new();
                    bytes_to_send.extend([129, payload_to_send.len() as u8]); // len might be larger than
                                                                              // u8
                    bytes_to_send.extend(payload_to_send.as_bytes());
                    stream.write(&bytes_to_send);
                }
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) -> std::io::Result<()> {
    stream.set_nonblocking(true)?;
    let block_duration = 100;
    let mut buf = vec![0u8; 1024];
    let mut did_handshake = false;
    let mut compare_manager: Option<CompareManager> = None;

    loop {
        eprintln!(".");

        match stream.read(&mut buf) {
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                thread::sleep(time::Duration::from_millis(block_duration));
                continue;
            }
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            }
            Ok(0) => {
                eprintln!("No data received");
                break;
            }
            Ok(size) => {
                println!("message size: {size}");
                if !did_handshake {
                    did_handshake = perform_handshake(&buf, &mut stream);
                } else {
                    let frame_data = &buf[..size];
                    handle_message(frame_data, &mut stream, &mut compare_manager);
                }
            }
        };
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8000")?;
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                eprintln!("handle_conn");
                handle_connection(stream)?;
            }
            Err(e) => {
                eprintln!("{e}");
            }
        }
    }
    Ok(())
}
