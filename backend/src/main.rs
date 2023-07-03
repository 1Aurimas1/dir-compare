use core::time;
use serde::Deserialize;
use std::{
    fs,
    io::{ErrorKind, Read, Write},
    net::{Ipv4Addr, Ipv6Addr, SocketAddrV6, TcpListener, TcpStream},
    thread,
};

use base64::{engine::general_purpose, Engine};
use sha1::{Digest, Sha1};

#[derive(Copy, Clone)]
enum PacketType {
    Open,
    Close,
    Ping,
    Pong,
    Message,
    Upgrade,
    Noop,
}

impl PacketType {
    //fn from_type(self) -> u8 {
    //    match self {
    //        PacketType::Open => 0,
    //        PacketType::Close => 1,
    //        PacketType::Ping => 2,
    //        PacketType::Pong => 3,
    //        PacketType::Message => 4,
    //        PacketType::Upgrade => 5,
    //        PacketType::Noop => 6,
    //    }
    //}
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
            5 => Some(PacketType::Upgrade),
            6 => Some(PacketType::Noop),
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
    Prev,
    Next,
}

impl Command {
    fn parse(content: String) -> Self {
        let parts: Vec<&str> = content.splitn(2, ";").collect();
        let comm = parts[0].split(":").last().unwrap();
        match comm {
            "search" => Command::Search(DirPair::parse(parts[1])),
            "next" => Command::Next,
            "prev" => Command::Prev,
            _ => todo!(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct Duplicate {
    file_name: String,
    first_dir_match: String,
    second_dir_match: Vec<String>,
}

//struct Message {
//    command: Command,
//    content: String,
//}
//
//impl Message {
//    fn new(command: &str) -> Self {
//        Self { command }
//    }
//}

fn get_payload_size(encoded: &[u8], byte_idx: usize, byte_count: usize) -> u64 {
    // most significant must be 0
    let total_bits = byte_count * 8;
    let mut new_size = 0u64;
    for i in 0..byte_count {
        new_size |= (encoded[i + byte_idx] as u64) << (total_bits - 8 * i);
    }
    return new_size;
}

fn handle_connection(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buf = vec![0u8; 1024];
    stream.set_nonblocking(true)?;
    let mut did_handshake = false;

    loop {
        eprintln!(".");
        if did_handshake {
            let mut combined: Vec<u8> = Vec::new();
            //let test_message = b"42[\"message\",\"ayaya1\"]";
            let test_message = b"4ayaya1";
            combined.extend([129, 7]);
            combined.extend(test_message);
            //combined.extend([129, 1]);
            //combined.extend(b"2");
            //println!("combined: {combined:?}");
            //stream.write(&combined);
            //println!("ping data: {combined:?}");
            //match stream.write(&combined) {
            //    Ok(size) => {
            //        eprintln!("pinged res: {size}");
            //    }
            //    Err(e) => {
            //        eprintln!("pinged err: {e}");
            //    }
            //}
        }

        let payload = match stream.read(&mut buf) {
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                //eprintln!("Error: {e}");
                thread::sleep(time::Duration::from_millis(500));
                //std::process::exit(1);
                continue;
            }
            Err(e) => {
                eprintln!("Error: {e}");
                //std::process::exit(1);
                break;
            }
            Ok(0) => {
                eprintln!("No data received");
                //std::process::exit(1);
                break;
            }
            Ok(size) => {
                println!("size: {size}");
                //let req_buf = b"Host: foo.bar\nAccept: */*\n\nblah blah";
                if !did_handshake {
                    let mut headers = [httparse::EMPTY_HEADER; 13];
                    let mut req = httparse::Request::new(&mut headers);
                    match req.parse(&buf) {
                        Ok(offset) => {
                            let ws_key =
                                match headers.iter().find(|v| v.name == "Sec-WebSocket-Key") {
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
                                    eprintln!("written {r:?}")
                                }
                                Err(e) => {
                                    eprintln!("write err {e}")
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Parse err: {e}");
                        }
                    }
                    did_handshake = true;
                } else {
                    //let mask = [1, 2, 3, 4];
                    let encoded = &buf[..size];
                    println!("encoded_Hex: {encoded:x?}");
                    println!("encoded: {encoded:?}");
                    for byte in encoded {
                        print!("{:b} ", byte);
                    }
                    println!();
                    let mut offset = 6;
                    let mut byte_idx = 0;
                    let fin: u8 = encoded[byte_idx] & 0x80;
                    let rsv: u8 = encoded[byte_idx] & 0x70;
                    let opcode: u8 = encoded[byte_idx] & 0x0F;
                    byte_idx += 1;
                    let mask_set: u8 = encoded[byte_idx] & 0x80;
                    let mut payload_size: u64 = (encoded[byte_idx] & 0x7F).into();
                    byte_idx += 1;
                    println!("{}, {:b}", opcode, 0b1);
                    println!("enc1:{}, {:b}", encoded[1], 0b1);
                    println!("{}, {:b}", mask_set, 0b01);
                    match payload_size {
                        126 => {
                            //payload_size = encoded[2] & 0x00F0 + encoded[3] & 0x000F;
                            //payload_size = ((encoded[2] as u64) << 8) | encoded[3] as u64;
                            payload_size = get_payload_size(encoded, byte_idx, 2);
                            offset += 2;
                        }
                        127 => {
                            //let byte_count = 8;
                            //let total_bits = byte_count * 8;
                            //let mut new_size = 0u64;
                            //for i in 0..byte_count {
                            //    new_size |= (encoded[i + byte_idx] as u64) << (total_bits - 8 * i);
                            //}
                            //payload_size = new_size;
                            payload_size = get_payload_size(encoded, byte_idx, 8);
                            offset += 8;
                        }
                        _ => (),
                    }
                    println!("payload_size: {}, {:b}", payload_size, 0b10);
                    let mask: &[u8] = &encoded[offset - 4..offset];
                    println!("{:?}, {:b}", mask, 0b10);

                    let decoded: Vec<u8> = encoded
                        .iter()
                        .skip(offset)
                        .enumerate()
                        .map(|(i, x)| {
                            return x ^ mask[i % 4];
                        })
                        .collect();
                    println!("decoded: {decoded:?}");
                    println!("decoded as text: {:?}", String::from_utf8_lossy(&decoded));
                    let received_text = String::from_utf8_lossy(&decoded);
                    if decoded[0] - 48 == PacketType::Message.to_u8() {
                        let command = Command::parse(received_text.to_string());
                        let test = match command {
                            Command::Search(dirs) => {
                                eprintln!("{dirs:?}");
                                std::process::Command::new("../dir_diff/target/release/dir_diff")
                                    //.current_dir("../dir_diff/target/release/")
                                    .arg(dirs.dir1)
                                    //.arg(dirs.dir2)
                                    .spawn()
                                //.expect("ls command failed to start");
                            }
                            Command::Prev | Command::Next => todo!(),
                        };
                        match test {
                            Ok(mut test) => {
                                test.wait().expect("Command wasn't running");
                                let dups_json = fs::read_to_string("./duplicates.json").unwrap();
                                let duplicates: Vec<Duplicate> =
                                    serde_json::from_str(&dups_json).unwrap();

                                let first_match =
                                    fs::read_to_string(&duplicates[0].first_dir_match).unwrap();
                                let second_match =
                                    fs::read_to_string(&duplicates[0].second_dir_match[0]).unwrap();
                                let test = format!("4{}", first_match);
                                let mut test1: Vec<u8> = Vec::new();
                                test1.extend([129, test.len() as u8]); // len might be larger than
                                                                       // u8
                                test1.extend(test.as_bytes());
                                stream.write(&test1);

                                println!("runnin");
                            }
                            Err(e) => eprintln!("{e}"),
                        }
                        println!("workin");
                    }
                    //let result_base64 = general_purpose::STANDARD.decode(decoded).unwrap();
                    //println!("result_base64: {result_base64:?}");
                    //println!("result_base64 as text: {:?}", String::from_utf8_lossy(&result_base64));
                    //buf[..size].iter().map()
                    //stream.write(b"4alo");
                    //println!("{res:?}");

                    println!("received data u8: {buf:?}");
                    let buf_text = String::from_utf8_lossy(&buf);
                    println!("received data text: {buf_text:?}");

                    //std::str::from_utf8(&buf[..size]).unwrap()
                }
                ()
            }
        };
        //eprintln!("\n{payload}");
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    println!("Hello, world!");
    //let ip_addr = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1);
    //let listener = TcpListener::bind(SocketAddrV6::new(ip_addr, 8000, 0, 0))?;
    let listener = TcpListener::bind("127.0.0.1:8000")?;
    //match listener.accept() {
    //    Ok((stream, addr)) => {eprintln!("ACCEPTED {stream:?} {addr}");},
    //    Err(e) => {eprintln!("{e}");}
    //}
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
