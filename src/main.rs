mod audio_container;
mod audio_data;

use audio_container::{AudioContainer, OggContainer};
use audio_data::{Audio, AudioVec};
use rodio::OutputStreamHandle;
use rodio::{source::Source, Decoder, OutputStream, Sink};
use std::time::Duration;
use std::{
    fs::File,
    io::{stdin, Write},
    num::NonZero,
};
use vorbis_rs::VorbisEncoderBuilder;
struct Color(u8, u8, u8);

struct TerminalColor(Color, Color); //FOREGROUND, BACKGROUND

fn main() {
    let file_path = "audio.ogg";
    let mut source_ogg = File::open(&file_path).unwrap();
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let file_extension = get_file_extension_from_path(&file_path.to_string());
    let container = get_supported_container_type(&file_extension);
    let mut audio_vec = audio_vec_from_file(
        source_ogg,
        &stream_handle,
        file_path.to_string(),
        container.unwrap(),
    )
    .unwrap();

    print!("\x1B[2J\x1B[1;1H");
    loop {
        let status = create_audio_selection_status(78, &audio_vec);
        println!("File:{}\nStatus:\n{}", audio_vec.get_path(), status);

        println!("Enter a command (\"command_list\"): ");
        let mut line = String::new();
        stdin().read_line(&mut line).unwrap();

        match line.trim() {
            "play" => {
                audio_vec.play_audio();
            }
            "save" => {
                save_audio_file(&audio_vec);
            }
            "set_start" => {
                let temp_start = get_second_value();
                if temp_start != -1.0 {
                    audio_vec.set_trim_start(temp_start)
                }
            }
            "set_end" => {
                let temp_end = get_second_value();
                if temp_end != -1.0 {
                    audio_vec.set_trim_end(temp_end);
                }
            }
            "quit" => {
                break;
            }
            "stop_audio" => {
                audio_vec.stop_audio();
            }
            "command_list" => {
                command_list();
                continue;
            }
            "set_source" => {
                if let Some(mut source) = set_source(&stream_handle) {
                    audio_vec = source;
                } else {
                    println!("\nSOURCE COULD NOT BE UPDATED");
                    continue;
                }
            }
            _ => {
                print!("\x1B[2J\x1B[1;1H");
                continue;
            }
        }
        print!("\x1B[2J\x1B[1;1H");
    }
}

fn command_list() {
    print!("\x1B[2J\x1B[1;1H");
    let commands = vec![
        "play",
        "save",
        "set_start",
        "set_end",
        "quit",
        "stop_audio",
        "clear_screan",
        "command_list",
        "set_source",
    ];
    for i in 0..commands.len() {
        println!("{}. {}", i + 1, commands[i]);
    }
}

fn set_source(output_stream_handle: &OutputStreamHandle) -> Option<AudioVec> {
    loop {
        println!("Enter a file path: ");
        let mut temp_buf = String::new();
        stdin().read_line(&mut temp_buf).unwrap();
        temp_buf = temp_buf.as_str().trim().to_string();
        if let Ok(mut file) = File::open(&temp_buf) {
            let file_path_parts: Vec<&str> = temp_buf.split('.').collect();
            let file_extension = file_path_parts[file_path_parts.len() - 1];

            if let Some(container) = get_supported_container_type(file_extension) {
                let mut audio =
                    audio_vec_from_file(file, output_stream_handle, temp_buf.clone(), container)
                        .unwrap();

                audio.set_trim_end(audio.get_duration());

                return Some(audio);
            } else {
                return None;
            }
        } else {
            return None;
        }
    }
}

fn get_supported_container_type(extension: &str) -> Option<&dyn AudioContainer> {
    match extension.to_lowercase().as_str() {
        "ogg" => {
            return Some(&OggContainer);
        }
        _ => {
            return None;
        }
    }
}

fn audio_vec_from_file(
    mut file: File,
    stream_handle: &OutputStreamHandle,
    file_path: String,
    container: &dyn AudioContainer,
) -> Option<AudioVec> {
    let Audio(audio_data) = container.decode(&mut file);

    let mut sink = Sink::try_new(stream_handle).unwrap();
    let mut audio = AudioVec {
        audio_data: Audio(audio_data),
        trim_start: 0.00,
        trim_end: 202.00,
        sample_rate: 88200.00,
        sink: sink,
        current_position: 0.0,
        filepath: file_path,
    };

    Some(audio)
}

fn get_second_value() -> f64 {
    loop {
        println!("Enter a value \nValue must be positive\nType \"back\" to go back\nPosition in seconds: ");
        let mut temp_buf = String::new();
        stdin().read_line(&mut temp_buf).unwrap();
        temp_buf = temp_buf.as_str().trim().to_string();
        if let Ok(some_val) = temp_buf.parse::<f64>() {
            if some_val >= 0.0 {
                return some_val;
            }
        } else {
            if temp_buf == "back" {
                return -1.0;
            }
        }
    }
}

fn save_audio_file(audio_vec: &AudioVec) {
    let audio_data = audio_vec.get_audio_slice();
    println!("Enter a file name:");
    let mut file_path = String::new();
    stdin().read_line(&mut file_path).unwrap();
    file_path = file_path.as_str().trim().to_string();
    let file_extension = get_file_extension_from_path(&file_path);

    if let Some(container) = get_supported_container_type(file_extension.as_str()) {
        container.save(&Audio(audio_vec.get_audio_slice().to_vec()), &file_path);
    }
}

fn get_file_extension_from_path(path: &String) -> String {
    let file_path_parts: Vec<&str> = path.split('.').collect();
    file_path_parts[file_path_parts.len() - 1].to_string()
}

fn create_audio_selection_status(width: u64, audio_vec: &AudioVec) -> String {
    let duration = audio_vec.get_duration();
    let start_indicator_position = audio_vec.get_trim_start() / duration;
    let end_indicator_position = audio_vec.get_trim_end() / duration;
    let start_position = width as f64 * start_indicator_position;
    let current_position =
        ((audio_vec.get_play_position() + audio_vec.get_trim_start()) / duration) * (width as f64);
    let end_position = width as f64 * end_indicator_position;
    let mut status = String::new();
    status.push('[');
    for i in 0..width {
        if i as f64 >= start_position && i as f64 <= end_position {
            if current_position.ceil() as u64 == i {
                status.push_str({
                    let foreground = Color(255, 165, 0);
                    let background = Color(255, 165, 0);
                    let color = TerminalColor(foreground, background);
                    &color_terminal("#", &color)
                });
            } else {
                status.push('#');
            }
        } else {
            status.push(' ');
        }
    }
    status.push(']');
    let current_sec = audio_vec.get_play_position() + audio_vec.get_trim_start();
    status.push_str(
        format!(
            "\n{}/{}",
            round_to_decimal(current_sec, 2),
            round_to_decimal(duration, 2)
        )
        .as_str(),
    );
    status
}

fn round_to_decimal(num: f64, place: u32) -> f64 {
    (num * 10u32.pow(place) as f64).floor() / 10u32.pow(place) as f64
}

fn color_terminal(string: &str, color: &TerminalColor) -> String {
    format!(
        "\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m{}\x1b[0m",
        color.0 .0, color.0 .1, color.0 .2, color.1 .0, color.1 .1, color.1 .2, string
    )
}
