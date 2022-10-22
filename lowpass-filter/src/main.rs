use std::fs::File;
use std::path::Path;

use apodize::hanning_iter;
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

fn write_wav_data(header: wav::Header, bit_depth: wav::BitDepth) {
    let mut out_file = File::create(Path::new("assets/output.wav")).expect("write file okay");
    wav::write(header, &bit_depth, &mut out_file).expect("write okay");
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

//resizing the signal will simplify windowing, if you wanted to be exact you could trim the end off in post
fn resize_signal(signal: &Vec<f32>, window_size: usize) -> Vec<f32> {
    let signal_length = signal.len();
    let padding = signal_length % window_size;
    let mut signal_resized = signal.clone();
    signal_resized.resize(signal_length + padding, 0.0);
    return signal_resized;
}

fn windowed_lowpass(
    num_windows: usize,
    window_size: usize,
    offset: usize,
    cut_off: usize,
    input_reference: &mut [Complex<f32>],
) ->  Vec<f32> {
    let mut write_signal : Vec<f32> = vec![];
    let planner: &mut FftPlanner<f32> = &mut FftPlanner::new();
    for window_index in 0..num_windows {
        let window_start = (window_index * window_size) + offset;
        let window_end = window_start + window_size;
        let window = &mut input_reference[window_start..window_end];
        let window_processed = process_complex_slice(window, planner, window_size, cut_off);
        // convert back to real numbers
        let window_processed: Vec<f32> = window_processed.into_iter().map(|x| x.re).collect();
        //normalise
        let window_processed: Vec<f32> = window_processed
            .into_iter()
            .map(|x| x / (window_size as f32))
            .collect();
        write_signal.extend(window_processed);
    }
    return write_signal;
}

fn main() {
    const CUT_OFF: usize = 20;
    const WINDOW_SIZE: usize = 1024;
    //load wav file
    let (header, wav_data) =
        read_wav_data("./assets/drum-loop-102-bpm.wav").expect("reading file successful");
    //convert to 16 bit vector and unwrap
    let signal = wav_data.as_thirty_two_float();
    //TODO: if I was doing this for real I should probably be able to parse different encodings
    let signal = signal.expect("wav data is 32 bit float");
    let signal_resized = &resize_signal(signal, WINDOW_SIZE);
    //convert to complex numbers
    let signal_complex: Vec<Complex<f32>> = signal_resized
        .into_iter()
        .map(|x| Complex { re: *x, im: 0.0 })
        .collect();

    
    let n_windows = signal_resized.len() / WINDOW_SIZE;
    let mut output_signal = windowed_lowpass(
        n_windows,
        WINDOW_SIZE,
        0,
        CUT_OFF,
        &mut signal_complex.clone()[..],
    );

    let overlap_offset = ((0.5 * WINDOW_SIZE as f32) - 1.0) as usize;
    let overlap_signal = windowed_lowpass(
        n_windows - 1,
        WINDOW_SIZE,
        overlap_offset,
        CUT_OFF,
        &mut signal_complex.clone()[..],
    );

    let hann_window = hanning_iter(WINDOW_SIZE)
        .map(|x| x as f32)
        .collect::<Vec<f32>>();
    // println!("{}", hann_window.len());
    for i in 0..overlap_signal.len() {
        //get index within current window
        let window_index = i % WINDOW_SIZE;
        //get hanning window coefficient
        let hann_coefficient = hann_window[window_index];
        //add the overlapping signal
        output_signal[i + (WINDOW_SIZE / 2) - 1] = overlap_signal[i] * hann_coefficient
            + output_signal[i + (WINDOW_SIZE / 2) - 1] * (1.0 - hann_coefficient);
        // println!("{}", hann_coefficient);
    }

    let write_bit_depth = BitDepth::ThirtyTwoFloat(output_signal);
    write_wav_data(header, write_bit_depth);
}
