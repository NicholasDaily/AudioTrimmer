mod audio_data;


use audio_data::AudioVec;
use rodio::{source::Source, Decoder, OutputStream, Sink};
use std::time::Duration;
use std::{
    fs::File,
    io::{stdin, Write},
    num::NonZero,
};
use vorbis_rs::{VorbisDecoder, VorbisEncoderBuilder};

struct Color(u8, u8, u8);

struct TerminalColor(Color, Color);//FOREGROUND, BACKGROUND

fn main() {
    let file_path = "audio.ogg";
    let mut source_ogg = File::open(&file_path).unwrap();
    let audio_data = decode_audio_file(&mut source_ogg);
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let mut sink = Sink::try_new(&stream_handle).unwrap();

    let mut audio_vec = AudioVec{
        audio_data : audio_data,
        trim_start : 0.00,
        trim_end : 202.00,
        sample_rate : 88200.00,
        sink : sink,
        current_position : 0.0
    };

    loop {
        let status = create_audio_selection_status(78, &audio_vec);
        println!("File:{}\nStatus:\n{}", &file_path, status);

        println!("Enter a command: ");
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
            "clear_screen" => {
                println!("");
            }
            _ => {
                print!("\x1B[2J\x1B[1;1H");
                continue;
            }
            
        }
        print!("\x1B[2J\x1B[1;1H");
    }
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
    println!("Enter a file name, (.ogg will be appended)\n:");
    let mut file_path = String::new();
    stdin().read_line(&mut file_path).unwrap();
    file_path = file_path.as_str().trim().to_string();
    file_path.push_str(".ogg");

    let mut output_vec = vec![];
    let mut encoder = VorbisEncoderBuilder::new(
        NonZero::new(44100).unwrap(),
        NonZero::new(2).unwrap(),
        &mut output_vec,
    )
    .unwrap()
    .build()
    .unwrap();

    let mut left = vec![];
    let mut right = vec![];

    for i in 0..audio_data.len() {
        if i % 2 != 0 {
            right.push(audio_data[i]);
            continue;
        }
        left.push(audio_data[i]);
    }

    let both = vec![left, right];

    encoder.encode_audio_block(both).unwrap();
    encoder.finish().unwrap();

    let mut output_file = File::create_new(&file_path).unwrap();

    _ = output_file.write_all(&output_vec).unwrap();
}

fn create_audio_selection_status(width: u64, audio_vec : &AudioVec) -> String {
    let duration = audio_vec.get_duration();
    let start_indicator_position = audio_vec.get_trim_start() / duration;
    let end_indicator_position = audio_vec.get_trim_end() / duration;
    let start_position = width as f64 * start_indicator_position;
    let current_position = ((audio_vec.get_play_position() + audio_vec.get_trim_start() )/ duration) * (width as f64);
    let end_position = width as f64 * end_indicator_position;
    let mut status = String::new();
    status.push('[');
    for i in 0..width {
        if i as f64 >= start_position && i as f64 <= end_position {
            if current_position.ceil() as u64 == i {
                
                status.push_str(
        { 
                    let foreground = Color(255, 165, 0); 
                    let background = Color(255, 165, 0); 
                    let color = TerminalColor(foreground, background);
                    &color_terminal("#", &color)
                });
            }else{
                status.push('#');
            }
        } else {
            status.push(' ');
        }
    }
    status.push(']');
    let current_sec = audio_vec.get_play_position() + audio_vec.get_trim_start();
    status.push_str(format!("\n{}/{}", round_to_decimal(current_sec, 2),round_to_decimal(duration,2)).as_str());
    status
}

fn decode_audio_file(file: &mut File) -> Vec<f32> {
    let mut decoder = VorbisDecoder::new(file).unwrap();

    let mut ogg_data = vec![];
    while let Some(decoded_block) = decoder.decode_audio_block().unwrap() {
        let l_audio = decoded_block.samples();
        let num_samples = l_audio[0].len();
        for i in 0..num_samples {
            for block in decoded_block.samples() {
                ogg_data.push(block[i]);
            }
        }
    }
    ogg_data
}

fn round_to_decimal(num : f64, place : u32) -> f64{
    (num * 10u32.pow(place) as f64).floor() / 10u32.pow(place) as f64
}

fn color_terminal (string : &str, color : &TerminalColor) -> String {
    format!(
        "\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m{}\x1b[0m"
        , color.0.0, color.0.1, color.0.2
        , color.1.0, color.1.1, color.1.2, string)
    
}







