use std::{fs::File, io::Write, num::NonZero};

use vorbis_rs::{VorbisDecoder, VorbisEncoderBuilder};

use crate::audio_data::Audio;
pub trait AudioContainer {
    fn encode(&self, audio: &Audio) -> Audio;
    fn decode(&self, file: &mut File) -> Audio;
    fn save(&self, audio: &Audio, file_path: &str);
}

pub struct OggContainer;

impl AudioContainer for OggContainer {
    fn encode(&self, audio: &Audio) -> Audio {
        let Audio(audio) = audio;

        Audio(audio.clone())
    }

    fn decode(&self, file: &mut File) -> Audio {
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

        Audio(ogg_data)
    }

    fn save(&self, audio: &Audio, file_path: &str) {
        let Audio(audio_data) = audio;
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
}
