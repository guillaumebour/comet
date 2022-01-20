mod bauds;
mod utils;

use chrono;
use chrono::{DateTime, Utc};
use clap::App;
use clap::{Arg, ArgMatches};
use hex;
use question::{Answer, Question};
use serde::{Serialize, Serializer};
use serialport;
use serialport::{SerialPortBuilder, SerialPortType};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::exit;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{fs, thread};
use termion::{color, style};

struct InputConfig {
    port: SerialPortBuilder,
    port_id: i32,
    next_idx: Arc<Mutex<i32>>,
}

struct OutputConfig {
    session_name: String,
    display_timestamp: bool,
    display_direction: bool,
    with_colour: bool,
}

#[derive(Serialize)]
struct CapturedData {
    idx: i32,
    timestamp: DateTime<Utc>,
    source: i32,
    is_raw: bool,
    #[serde(serialize_with = "encode_to_hex")]
    data: Vec<u8>,
}

fn encode_to_hex<S>(data: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(hex::encode(data).as_str())
}

fn build_app() -> App<'static> {
    App::new("comet")
        .author(clap::crate_authors!("\n"))
        .version(clap::crate_version!())
        .about("A simple program to capture serial communications")
        .args(&[
            Arg::new("port")
                .short('p')
                .long("port")
                .takes_value(true)
                .required_unless_present("list-ports")
                .required_unless_present("common-baudrates")
                .help("Port 1"),
            Arg::new("port2")
                .long("port2")
                .required(false)
                .takes_value(true)
                .help("Port 2"),
            Arg::new("baud")
                .long("baud")
                .default_value("115200")
                .help("Baudrate for port"),
            Arg::new("baud2")
                .long("baud2")
                .default_value("115200")
                .help("Baudrate for port 2"),
            Arg::new("no-timestamp")
                .long("no-timestamp")
                .help("Do not display timestamps"),
            Arg::new("no-direction")
                .long("no-direction")
                .help("Do not display message direction information"),
            Arg::new("no-colour")
                .long("no-colour")
                .help("Do not display colours"),
            Arg::new("session-name")
                .long("session-name")
                .takes_value(true)
                .help("Name of the capture session"),
            Arg::new("list-ports")
                .long("list-ports")
                .help("List available TTY ports"),
            Arg::new("common-baudrates")
                .long("common-baudrates")
                .help("List common baudrates"),
        ])
}

fn main() {
    let app = build_app();
    let app_args = app.get_matches();

    if app_args.is_present("list-ports") {
        list_available_ports();
    } else if app_args.is_present("common-baudrates") {
        bauds::display_common_baudrates();
    } else {
        listen(app_args);
    }
}

fn list_available_ports() {
    let ports = serialport::available_ports().unwrap();
    for p in ports {
        println!("{} ({})", p.port_name, {
            match p.port_type {
                SerialPortType::UsbPort(_) => "USB",
                SerialPortType::PciPort => "PCI",
                SerialPortType::BluetoothPort => "Bluetooth",
                SerialPortType::Unknown => "Unknown",
            }
        });
    }
}

fn listen(args: ArgMatches) {
    if args.is_present("port") {
        let port = args.value_of("port").unwrap();
        if args.is_present("baud") {
            let baud_str = args.value_of("baud").unwrap();
            let baud: u32 = baud_str.parse().expect("failed to parse baud rate");

            let session_name = match args.is_present("session-name") {
                true => String::from(args.value_of("session-name").unwrap()),
                false => {
                    let current_time = Utc::now();
                    current_time.format("comet_%Y%m%d_%H%M%S").to_string()
                }
            };

            let (tx, rx) = mpsc::channel();
            let shared_idx = Arc::new(Mutex::new(0));

            let input_config_1 = InputConfig {
                port: serialport::new(port, baud).timeout(Duration::from_millis(2000)),
                port_id: 1,
                next_idx: Arc::clone(&shared_idx),
            };

            if Path::new(&session_name).exists() {
                let answer = Question::new(format!("[!] The capture directory for session name {} already exists, override? (y/n) ", session_name).as_str()).yes_no().confirm();
                match answer {
                    Answer::RESPONSE(_) => {
                        println!("SHOULD NOT HAPPEN")
                    }
                    Answer::YES => {
                        fs::remove_dir_all(&session_name).unwrap();
                    }
                    Answer::NO => {
                        println!("Exiting...");
                        exit(0);
                    }
                }
            }

            println!("[*] session: {}", session_name);

            let output_config = OutputConfig {
                session_name,
                display_timestamp: !args.is_present("no-timestamp"),
                display_direction: !args.is_present("no-direction"),
                with_colour: !args.is_present("no-colour"),
            };

            println!("[*] port 1: {}", port);
            println!("[*] baudrate for port 1: {}", baud);

            thread::spawn(|| handle_message(output_config, rx));

            if args.is_present("port2") {
                let port2 = args.value_of("port2").unwrap();
                if args.is_present("baud2") {
                    let baud2_str = args.value_of("baud2").unwrap();
                    let baud2: u32 = baud2_str
                        .parse()
                        .expect("failed to parse baud rate for the second port");

                    println!("[*] port 2: {}", port2);
                    println!("[*] baudrate for port 2: {}", baud2);

                    let input_config_2 = InputConfig {
                        port: serialport::new(port2, baud2).timeout(Duration::from_millis(2000)),
                        port_id: 2,
                        next_idx: shared_idx,
                    };

                    let tx2 = tx.clone();
                    thread::spawn(move || {
                        receive_on_port(input_config_2, tx2);
                    });
                }
            }

            let reception_thread_1 = thread::spawn(|| {
                receive_on_port(input_config_1, tx);
            });

            reception_thread_1.join().unwrap();
        }
    }
}

fn receive_on_port(cfg: InputConfig, send_to: Sender<CapturedData>) {
    let port = cfg.port.open().expect("failed to open serial port");
    let mut reader = BufReader::new(port);

    println!("[*] start listening on port {}...", cfg.port_id);
    loop {
        let mut next_line_bytes: Vec<u8> = Vec::new();
        match reader.read_until(b'\n', &mut next_line_bytes) {
            Ok(_) => {
                let mut current_idx_mut = cfg.next_idx.lock().unwrap();

                let parsed_line = String::from_utf8(next_line_bytes.clone());
                let captured_data = CapturedData {
                    idx: *current_idx_mut,
                    timestamp: Utc::now(),
                    source: cfg.port_id,
                    is_raw: parsed_line.is_err(),
                    data: next_line_bytes,
                };

                match send_to.send(captured_data) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("error while sending line to main receiver: {}", e)
                    }
                };

                *current_idx_mut += 1;
            }
            Err(_) => {
                continue;
            }
        }
    }
}

fn handle_message(cfg: OutputConfig, receive_on: Receiver<CapturedData>) {
    let session_path = Path::new(&cfg.session_name);
    fs::create_dir_all(session_path).unwrap();

    let mut out_file = File::create(session_path.join("console_log.txt")).unwrap();
    let out_file_json = File::create(session_path.join("capture.json")).unwrap();

    let mut json_writer = utils::IncrementalJsonWriter::new(out_file_json);

    for msg in receive_on {
        let data_to_display;
        if !msg.is_raw {
            data_to_display = match String::from_utf8(msg.data.clone()) {
                Ok(v) => v,
                Err(e) => {
                    println!("[ERROR] failed to decode from utf-8: {}", e);
                    continue;
                }
            };
        } else {
            data_to_display = hex::encode(msg.data.clone());
        }

        let timestamp = if cfg.display_timestamp {
            String::from(format!("[{}]", msg.timestamp))
        } else {
            String::from("")
        };

        let direction = if cfg.display_direction {
            if cfg.with_colour {
                if msg.source == 1 {
                    String::from(format!("[{}<{}]", color::Fg(color::Blue), style::Reset))
                } else {
                    String::from(format!("[{}>{}]", color::Fg(color::Red), style::Reset))
                }
            } else {
                String::from(format!("[{}]", if msg.source == 1 { "<" } else { ">" },))
            }
        } else {
            String::from("")
        };

        let line_to_display = String::from(format!(
            "{}{} {}",
            direction,
            timestamp,
            data_to_display.trim()
        ));
        println!("{}", line_to_display.trim());

        match out_file.write_all(&msg.data) {
            Ok(_) => {}
            Err(err) => {
                println!("[ERROR] could not write line to file: {}", err)
            }
        };

        match json_writer.write_json(&msg) {
            Ok(_) => {}
            Err(err) => {
                println!("[ERROR] could not write json msg to file: {}", err)
            }
        }
    }
}
