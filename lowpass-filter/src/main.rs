use std::fs::File;
use std::path::Path;

use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use wav::BitDepth;

fn read_wav_data(file_path: &str) -> Result<(wav::Header, wav::BitDepth), std::io::Error> {
    let mut input_file = File::open(Path::new(file_path))?;
    let (header, wav) = match wav::read(&mut input_file) {
        Err(e) => return Err(e),
        Ok(value) => value,
    };
    return Ok((header, wav));
    // header, data
}

fn process_complex_slice(
    slice: &mut [Complex<f32>],
    planner: &mut FftPlanner<f32>,
    window_size: usize,
    cut_off: usize,
) -> Vec<Complex<f32>> {
    let fft = planner.plan_fft_forward(window_size);
    let inverse_fft = planner.plan_fft_inverse(window_size);
    fft.process(slice);
    for i in cut_off..window_size {
        slice[i] = Complex { re: 0.0, im: 0.0 };
    }
    inverse_fft.process(slice);
    return slice.to_vec();
}

//resizing the signal will simplify windowing, if you wanted to be exact you could trim the end off prior
fn resize_signal(signal: &Vec<f32>, window_size: usize) -> Vec<f32> {
    let signal_length = signal.len();
    let padding = signal_length % window_size;
    let mut signal_resized = signal.clone();
    signal_resized.resize(signal_length + padding, 0.0);
    return signal_resized;
}

fn main() {
    const CUT_OFF: usize = 50;
    const WINDOW_SIZE: usize = 2048;
    //load wav file
    let (header, wav_data) =
        read_wav_data("./assets/drum-loop-102-bpm.wav").expect("reading file successful");
    //convert to 16 bit vector and unwrap
    let signal = wav_data.as_thirty_two_float();
    //TODO: if I was doing this for real I should probably be able to parse different encodings
    let signal = signal.expect("wav data is 32 bit float");
    let signal_resized = &resize_signal(signal, WINDOW_SIZE);
    //convert to complex numbers
    let mut signal_complex: Vec<Complex<f32>> = signal_resized
        .into_iter()
        .map(|x| Complex { re: *x, im: 0.0 })
        .collect();
    let buffer: &mut [Complex<f32>] = &mut signal_complex[..];

    //initialise ffts
    let planner: &mut FftPlanner<f32> = &mut FftPlanner::new();

    let mut output_signal: Vec<f32> = vec![];
    //process buffer in chunks
    let n_windows = (signal_resized.len() / WINDOW_SIZE) as i32;
    for window_index in 0..n_windows {
        let window_start = (window_index * WINDOW_SIZE as i32) as usize;
        let window_end = (window_start + WINDOW_SIZE) as usize;
        let window = &mut buffer[window_start..window_end];
        let window_processed = process_complex_slice(window, planner, WINDOW_SIZE, CUT_OFF);
        // convert back to real numbers
        let window_processed: Vec<f32> = window_processed.into_iter().map(|x| x.re).collect();
        //normalise
        let window_processed: Vec<f32> = window_processed
            .into_iter()
            .map(|x| x / (WINDOW_SIZE as f32))
            .collect();
        output_signal.extend(window_processed);
    }
    let write_bit_depth = BitDepth::ThirtyTwoFloat(output_signal);
    let mut out_file = File::create(Path::new("assets/output.wav")).expect("write file okay");
    wav::write(header, &write_bit_depth, &mut out_file).expect("write okay");
}

//SCRAP WORKINGS OUT THAT IM TOO SENTIMENTAL TO DELETE

//get FFT

// let buffer_size = buffer.len();
// let mut planner: FftPlanner<f32> = FftPlanner::new();
// let fft = planner.plan_fft_forward(buffer_size);
// fft.process(buffer);
// //set values to 0 above cut off in frequency domain
// for i in CUT_OFF..buffer_size {
//     println!("{}", i);
//     buffer[i] = Complex { re: 0.0, im: 0.0 };
// }
// //fft back into time domain
// let inverse_fft = planner.plan_fft_inverse(buffer_size);
// inverse_fft.process(buffer);
//convert back to real numbers
// let transformed_signal_complex = buffer[..].to_vec();
// println!("{}", transformed_signal_complex[0].im);
// let transformed_signal: Vec<f32> = transformed_signal_complex
//     .into_iter()
//     .map(|x| x.re)
//     .collect();
// //normalise by 1/buffer_size
// let normalised_transformed_signal: Vec<f32> = transformed_signal
//     .into_iter()
//     .map(|x| x / (buffer_size as f32))
//     .collect();
// //write to wav file
