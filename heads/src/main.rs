use std::fs::File;
use std::path::Path;
use wav::BitDepth;

struct TapeLoop {
    read_head_index: usize,
    write_head_index: usize,
    buffer: Vec<f32>,
}

impl TapeLoop {
    fn get_next_head_position(&mut self, cur_position: usize) -> usize {
        if cur_position < self.buffer.len() - 1 {
            return cur_position + 1;
        }
        return 0;
    }

    fn write_sample(&mut self, write_sample: f32) {
        //move value out into read value
        self.buffer[self.write_head_index] = write_sample;
        self.write_head_index = self.get_next_head_position(self.write_head_index);
    }

    fn read_sample(&mut self) -> f32 {
        let read_sample = self.buffer[self.read_head_index];
        self.read_head_index = self.get_next_head_position(self.read_head_index);
        return read_sample;
    }
}

fn read_wav_data(file_path: &str) -> Result<(wav::Header, wav::BitDepth), std::io::Error> {
    let mut input_file = File::open(Path::new(file_path))?;
    let (header, wav) = match wav::read(&mut input_file) {
        Err(e) => return Err(e),
        Ok(value) => value,
    };
    return Ok((header, wav));
}

fn write_wav_data(header: wav::Header, bit_depth: wav::BitDepth) {
    let mut out_file = File::create(Path::new("assets/output.wav")).expect("write file okay");
    wav::write(header, &bit_depth, &mut out_file).expect("write okay");
}

fn main() {
    const DELAY_LEN_SECONDS: f32 = 0.2;
    const FEEDBACK: f32 = 0.7;
    const WET_MIX: f32 = 0.5;
    let (header, wav_data) =
        read_wav_data("./assets/drum-loop-102-bpm.wav").expect("reading file successful");
    let signal = wav_data.as_thirty_two_float();
    let signal = signal.expect(" wav_data is thirty two bit float");

    let sample_rate = header.sampling_rate as f32;
    let tape_loop_length = (sample_rate * DELAY_LEN_SECONDS) as usize;
    let mut tape_loop = TapeLoop {
        read_head_index: 0,
        write_head_index: 0,
        buffer: vec![0.0; tape_loop_length],
    };

    let mut output_signal:Vec<f32> = vec![0.0; signal.len()];

    for i in 0..signal.len() {
        let sample_from_loop = tape_loop.read_sample();
        let sample_for_loop = sample_from_loop * FEEDBACK + signal[i];
        tape_loop.write_sample(sample_for_loop);
        let output_sample = sample_from_loop * WET_MIX + signal[i];
        output_signal[i] = output_sample;
    }

    let write_bit_depth = BitDepth::ThirtyTwoFloat(output_signal);
    write_wav_data(header, write_bit_depth)
}
