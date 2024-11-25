use std::time::Duration;

use rodio::Sink;

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

pub struct AudioVec {
    pub audio_data: Vec<f32>,
    pub trim_start: f64,
    pub trim_end: f64,
    pub sample_rate: f64, //88200.00
    pub sink: Sink,
    pub current_position: f64,
}

impl AudioVec {
    pub fn get_duration(&self) -> f64 {
        self.audio_data.len() as f64 / self.sample_rate
    }

    pub fn get_trim_duration(&self) -> f64 {
        self.get_audio_slice().len() as f64 / self.sample_rate
    }

    fn get_index_from_second(&self, second: f64) -> usize {
        let index = (second * self.sample_rate) as usize;
        if index % 2 == 0 {
            return index;
        }
        index - 1
    }

    pub fn play_audio(&self) -> () {
        self.sink.stop();
        let source = MySource::new(Vec::from(self.get_audio_slice()));
        self.sink.append(source);
    }

    pub fn get_play_position(&self) -> f64 {
        self.sink.get_pos().as_secs_f64()
    }

    pub fn stop_audio(&self) -> () {
        self.sink.stop();
    }

    pub fn get_audio_slice(&self) -> &[f32] {
        let start_index = self.get_index_from_second(self.trim_start);
        let end_index = self.get_index_from_second(self.trim_end);
        &self.audio_data[start_index as usize..end_index as usize]
    }

    pub fn get_trim_start(&self) -> f64 {
        self.trim_start
    }

    pub fn get_trim_end(&self) -> f64 {
        self.trim_end
    }

    pub fn set_trim_start(&mut self, seconds: f64) {
        self.trim_start = seconds;
    }

    pub fn set_trim_end(&mut self, seconds: f64) {
        self.trim_end = seconds;
    }
}
