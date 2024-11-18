use std::{fs::File, io::{stdin, Write}, num::NonZero};
use rodio::{source::Source, Decoder, OutputStream, Sink};
use std::time::Duration;
use vorbis_rs::{VorbisDecoder, VorbisEncoderBuilder};
struct MySource {
    buf: Vec<f32>,
    cur_idx: usize,
}

impl MySource {
    fn new(buf: Vec<f32>) -> Self {
        Self { buf, cur_idx: 0 }
    }
}

impl Iterator for MySource {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if self.cur_idx < self.buf.len() {
            let val = self.buf[self.cur_idx];
            self.cur_idx += 1;
            Some(val)
        } else {
            None
        }
    }
}

impl rodio::Source for MySource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        2
    }

    fn sample_rate(&self) -> u32 {
        44100
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
fn main() {
    let file_path = "audio.ogg";
    let mut source_ogg = File::open(&file_path).unwrap();
    let audio_data = decode_audio_file(&mut source_ogg);
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let mut sink = Sink::try_new(&stream_handle).unwrap();

    let mut start = 0.0;
    let mut end = get_duration(&audio_data);

    loop {
        let status= create_audio_selection_status(78, &audio_data, start, end);
        println!("File:{}\nStatus:\n{}", &file_path, status);

        println!("Enter a command: ");
        let mut line = String::new();
        stdin().read_line(&mut line).unwrap();
        
        
        match line.trim() {
            "play" => {
                play_audio(get_audio_slice(&audio_data, start, end), &mut sink);
            }
            "save" =>{
                save_audio_file(get_audio_slice(&audio_data, start, end));
            }
            "set_start"=>{
               let temp_start = get_second_value();
               if temp_start != -1.0 {
                start = temp_start;
               }
            }
            "set_end"=>{
                let temp_end = get_second_value();
               if temp_end != -1.0 {
                end = temp_end;
               }
            }
            "quit"=>{
                break;
            }           
            _=> {
                continue;
            }
        }
    }

}

fn get_second_value() -> f64 {
    loop {
        println!("Enter a value \nValue must be positive\nType \"back\" to go back\nPosition in seconds: ");
        let mut temp_buf = String::new();
        stdin().read_line(&mut temp_buf).unwrap();
        temp_buf = temp_buf.as_str().trim().to_string();
        if let Ok(some_val) = temp_buf.parse::<f64>() {
            if some_val >= 0.0{
                return some_val;
            }
        }else{
            if temp_buf == "back" {
                return -1.0;
            }
        }
        

    }
}

fn save_audio_file(audio_data : &[f32]) {
    println!("Enter a file name, (.ogg will be appended)\n:");
    let mut file_path = String::new();
        stdin().read_line(&mut file_path).unwrap();
    file_path = file_path.as_str().trim().to_string();
    file_path.push_str(".ogg");
    
    let mut output_vec = vec![];
    let mut encoder = VorbisEncoderBuilder::new(
        NonZero::new(44100).unwrap(),
        NonZero::new(2).unwrap(),
        &mut output_vec
    ).unwrap().build().unwrap();

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

fn create_audio_selection_status(width : u64, audio_data : &[f32], start : f64, end : f64) -> String {
    let duration = get_duration(audio_data);
    let start_indicator_position = start / duration;
    let end_indicator_position = end / duration;
    let start_position = width as f64 * start_indicator_position ;
    let end_position = width as f64 * end_indicator_position ;
    let mut status = String::new();
    status.push('[');
    for i in 0..width {
        if i as f64 >= start_position &&  i as f64 <= end_position {
            status.push('#');
        }else{
            status.push(' ');
        }
    }
    status.push(']');
    status
}

fn decode_audio_file(file : &mut File) -> Vec<f32> {
    let mut decoder = VorbisDecoder::new( file).unwrap();

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

fn get_audio_slice(audio_data : &[f32], start : f64, end : f64) -> &[f32] {
    
    let start_index = get_index_from_second(start);
    let end_index = get_index_from_second(end);
    &audio_data[start_index as usize..end_index as usize]
}

fn get_index_from_second(second : f64) -> usize {
      let index = (second * 88200.00) as usize;
      if index % 2 == 0 {
        return index;
      }
      index - 1
}

fn get_duration(audio_data : &[f32]) -> f64{
    audio_data.len() as f64 / 88200.00
}

fn play_audio(audio_data : &[f32], sink : &mut Sink) -> () {
    sink.stop();
    let source = MySource::new(Vec::from(audio_data));
    sink.append(source);
}
