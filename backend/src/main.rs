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

const DIR_COMPARE_PATH: &str = "../dir_compare/target/release/dir_compare";
const MAGIC: &[u8; 36] = b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

trait Bytes {
    fn to_bytes(&self, byte_count: usize) -> Vec<u8>;
}

impl Bytes for u64 {
    fn to_bytes(&self, byte_count: usize) -> Vec<u8> {
        // most significant must be 0
        (0..byte_count)
            .map(|i| (self >> (8 * (byte_count - 1 - i))) as u8)
            .collect()
    }
}

#[derive(Copy, Clone)]
enum OpCode {
    Continuation,
    Text,
    Binary,
    Close = 0x8,
    Ping = 0x9,
    Pong = 0xA,
}

impl OpCode {
    fn to_u8(&self) -> u8 {
        *self as u8
    }

    fn from_u8(num: u8) -> Result<OpCode, String> {
        match num {
            x if x == OpCode::Continuation.to_u8() => Ok(OpCode::Continuation),
            x if x == OpCode::Text.to_u8() => Ok(OpCode::Text),
            x if x == OpCode::Binary.to_u8() => Ok(OpCode::Binary),
            x if x == OpCode::Close.to_u8() => Ok(OpCode::Close),
            x if x == OpCode::Ping.to_u8() => Ok(OpCode::Ping),
            x if x == OpCode::Pong.to_u8() => Ok(OpCode::Pong),
            _ => Err("Received unknown opcode".into()),
        }
    }
}

struct WebSocketFrame {
    fin: bool,
    op_code: OpCode,
    mask: bool,
    payload_length: u64,
    masking_key: [u8; 4],
    payload_data: Vec<u8>,
}

impl WebSocketFrame {
    fn new(
        fin: bool,
        op_code: OpCode,
        mask: bool,
        masking_key: [u8; 4],
        payload_data: Vec<u8>,
    ) -> Self {
        Self {
            fin,
            op_code,
            mask,
            payload_length: payload_data.len() as u64,
            masking_key,
            payload_data,
        }
    }

    #[allow(unused_assignments)]
    fn from_bytes(frame_data: &[u8]) -> Result<Self, String> {
        let mut offset = 6;
        let mut byte_idx = 0;
        let fin = (frame_data[byte_idx] & 0x80) != 0;
        let _rsv = frame_data[byte_idx] & 0x70;
        let op_code = match OpCode::from_u8(frame_data[byte_idx] & 0x0F) {
            Ok(code) => code,
            Err(e) => return Err(e),
        };
        byte_idx += 1;

        let mask = (frame_data[byte_idx] & 0x80) != 0;
        let mut payload_size: u64 = (frame_data[byte_idx] & 0x7F).into();
        byte_idx += 1;

        match payload_size {
            126 => {
                let bytes_to_read = 2;
                payload_size = Self::decode_payload_length(frame_data, byte_idx, bytes_to_read);
                offset += bytes_to_read;
            }
            127 => {
                let bytes_to_read = 8;
                payload_size = Self::decode_payload_length(frame_data, byte_idx, bytes_to_read);
                offset += bytes_to_read;
            }
            _ => (),
        }
        let masking_key: &[u8; 4] = &frame_data[offset - 4..offset]
            .try_into()
            .expect("Masking key conversion: slice into fixed-size array shouldn't fail");

        let unmasked_data: Vec<u8> = frame_data
            .iter()
            .skip(offset)
            .enumerate()
            .map(|(i, x)| {
                return x ^ masking_key[i % 4];
            })
            .collect();

        Ok(Self::new(fin, op_code, mask, *masking_key, unmasked_data))
    }

    fn to_bytes(self) -> Vec<u8> {
        // could preallocate?
        let mut bytes_to_send = vec![];
        bytes_to_send.push(((self.fin as u8) << 7) | self.op_code.to_u8());

        bytes_to_send.push(self.mask as u8);
        if self.payload_length < 126 {
            bytes_to_send[1] |= self.payload_length as u8;
        } else if self.payload_length < (1 << 16) {
            bytes_to_send[1] |= 126;
            bytes_to_send.extend(self.payload_length.to_bytes(2));
        } else {
            bytes_to_send[1] |= 127;
            bytes_to_send.extend(self.payload_length.to_bytes(8));
        }

        if self.mask {
            bytes_to_send.extend(self.masking_key);
            // finish masking
            todo!("Unfinished masking in to_bytes()");
        }

        bytes_to_send.extend(self.payload_data);
        return bytes_to_send;
    }

    fn decode_payload_length(frame_data: &[u8], byte_idx: usize, byte_count: usize) -> u64 {
        // most significant must be 0
        // unnecessary? * 8
        let total_bits = byte_count * 8;
        let mut new_size = 0u64;
        for i in 0..byte_count {
            new_size |= (frame_data[i + byte_idx] as u64) << (total_bits - 8 * i);
        }
        return new_size;
    }
}

#[derive(Debug)]
struct CompareDirs {
    dirs: Vec<Option<String>>,
}

impl CompareDirs {
    const MAX_DIRS: usize = 2;
    fn parse(text: &str) -> Self {
        let parts: Vec<&str> = text.split(";").collect();
        let mut parts_iter = parts.iter();
        let mut dirs: Vec<Option<String>> = Vec::with_capacity(Self::MAX_DIRS);
        for _ in 0..Self::MAX_DIRS {
            dirs.push(if let Some(p) = parts_iter.next() {
                let path = p.split(":").last().unwrap().to_string();
                if !path.is_empty() {
                    Some(path)
                } else {
                    None
                }
            } else {
                None
            });
        }
        Self { dirs }
    }
}

enum Command {
    Search(CompareDirs),
    Prev(usize),
    Next(usize),
}

impl Command {
    fn parse(content: String) -> Self {
        let parts: Vec<&str> = content.splitn(2, ";").collect();
        let comm = parts[0].split(":").last().unwrap();
        match comm {
            "search" => Command::Search(CompareDirs::parse(parts[1])),
            "next" => Command::Next(parts[1].parse::<usize>().unwrap()),
            "prev" => Command::Prev(parts[1].parse::<usize>().unwrap()),
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
        // implement wraparound?
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

fn perform_handshake(buf: &Vec<u8>, stream: &mut TcpStream) -> bool {
    let mut headers = [httparse::EMPTY_HEADER; 13];
    let mut req = httparse::Request::new(&mut headers);
    match req.parse(buf) {
        Ok(offset) => {
            let ws_key = match headers.iter().find(|v| v.name == "Sec-WebSocket-Key") {
                Some(val) => val.value,
                None => b"0",
            };
            let mut hasher = Sha1::new();
            hasher.update(ws_key);
            hasher.update(MAGIC);
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
                    eprintln!("handshake_response written: {r:?}");
                    true
                }
                Err(e) => {
                    eprintln!("handshake_response err: {e}");
                    false
                }
            }
        }
        Err(e) => {
            eprintln!("Request parse err: {e}");
            false
        }
    }
}

fn handle_message(
    frame_data: &[u8],
    stream: &mut TcpStream,
    compare_manager: &mut Option<CompareManager>,
) -> Result<(), String> {
    let ws_frame = match WebSocketFrame::from_bytes(frame_data) {
        Ok(frame) => frame,
        Err(e) => return Err(e),
    };

    match ws_frame.op_code {
        OpCode::Text | OpCode::Binary => {
            let received_text = String::from_utf8_lossy(&ws_frame.payload_data);
            println!("payload as text: {:?}", received_text);
            let command = Command::parse(received_text.to_string());
            match command {
                Command::Search(dirs) => {
                    let mut process_cmd = std::process::Command::new(DIR_COMPARE_PATH);
                    let mut can_spawn_process = false;
                    for dir in dirs.dirs.iter() {
                        if let Some(d) = dir {
                            process_cmd.arg(d);
                            can_spawn_process = true;
                        }
                    }
                    if !can_spawn_process {
                        eprintln!("Not enough args received to start dir search");
                    }

                    match process_cmd.spawn() {
                        Ok(mut child_proc) => {
                            child_proc.wait().expect("Command wasn't running");
                            // might read old file if process fails to search
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

                for (idx, file) in files.into_iter().enumerate() {
                    if let Some(f) = file {
                        let content = fs::read(&f).unwrap();

                        // lazy, temporary implementation
                        let file_type =
                            if f.ends_with(".png") || f.ends_with(".jpg") || f.ends_with(".bmp") {
                                "img"
                            } else {
                                "txt"
                            };
                        // could be improved?
                        let mut message = format!("{}{}", file_type, idx).as_bytes().to_vec();
                        message.extend(content);
                        let ws_frame =
                            WebSocketFrame::new(true, OpCode::Binary, false, [0u8; 4], message);
                        let bytes_to_send = ws_frame.to_bytes();

                        match stream.write(&bytes_to_send) {
                            Ok(n) => println!("bytes responded: {n}"),
                            Err(e) => eprintln!("response error: {e}"),
                        };
                    }
                }
            }
        }
        _ => todo!("Implement other frame type behavior"),
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> std::io::Result<()> {
    const BLOCK_DURATION: u64 = 100;
    stream.set_nonblocking(true)?;

    let mut buf = vec![0u8; 1024];
    let mut did_handshake = false;
    let mut compare_manager: Option<CompareManager> = None;

    loop {
        println!(".");

        match stream.read(&mut buf) {
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                thread::sleep(time::Duration::from_millis(BLOCK_DURATION));
                continue;
            }
            Err(e) => {
                eprintln!("Stream read error: {e}");
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
                    if let Err(e) = handle_message(&buf[..size], &mut stream, &mut compare_manager)
                    {
                        eprintln!("{e}");
                    }
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
                handle_connection(stream)?;
            }
            Err(e) => {
                eprintln!("Incoming stream error: {e}");
            }
        }
    }
    Ok(())
}
