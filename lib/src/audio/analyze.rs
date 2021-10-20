use super::ringbuf::{self, Consumer, Producer};
use super::{Buffer, FFT, FFT_IMSIZE, FFT_SIZE, FFT_BUFFERS, BUFFER_SIZE, NYQ};

use rustfft::num_complex::Complex32;
use rustfft::num_traits::Zero as _;

use rustfft::FftPlanner;

pub fn analyze(mut rx: Consumer<Buffer>, mut fft_tx: Producer<FFT>) {
    // Set up buffers for the input, complex FFT I/O, and result
    let mut buffers = vec![[0.0; BUFFER_SIZE]; FFT_BUFFERS];
   
    let mut complex = vec![Complex32::zero(); FFT_IMSIZE];
    let zeros = complex.clone();
    let mut result = [0.0; FFT_SIZE];

    // Set up the FFT
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(FFT_IMSIZE);
    let mut scratch = vec![Complex32::zero(); fft.get_inplace_scratch_len()];

    // Set up the window and calculate the factor we need to scale the FFT result by
    let window: Vec<_> = apodize::hanning_iter(FFT_SIZE).map(|v| v as f32).collect();
    let window_factor = window.iter().map(|x| *x as f32).sum::<f32>();

    // This *shouldn't* have any allocations
    loop {
        buffers.iter_mut().for_each(|buffer| ringbuf::receive(&mut rx, buffer));

        // Copy the samples into the real parts of the complex buffer and apply
        // the window function. The imaginary parts are already zero from when
        // we zeroed the entire buffer in the previous loop.
        buffers.iter().flatten()
            .zip(window.iter())
            .zip(complex.iter_mut())
            .for_each(|((sample, w), c)| c.re = *sample * *w);

        fft.process_with_scratch(&mut complex, &mut scratch);

        // Copy the abs of each complex result scaled by the window factor into the result buffer
        complex
            .iter()
            .take(FFT_SIZE)
            .zip(result.iter_mut())
            .for_each(|(c, v)| {
                *v = c.norm_sqr().sqrt() / window_factor;
            });

        // Zero the complex buffer to clear out the in-place modifications.
        // 
        // These aren't overwritten when we copy the samples in the next loop,
        // since we only write the samples to the first half of the complex input.
        //
        // TODO: Maybe we could use an out of place FFT to avoid this copy, but
        // it's probably a negligible optimization.
        complex.copy_from_slice(&zeros);

        // Send off the FFT data
        ringbuf::transmit(&mut fft_tx, &result);

        /*
        Do stuff later, for example:

        let energy_time = samples.iter().map(|y| y.powi(2)).sum::<f32>() * (1.0 / NYQ);
        let energy_freq = bins.iter().map(|y| (y / NYQ).abs().powi(2)).sum::<f32>() * (NYQ / FFT_FSIZE);

        let rms_time = energy_time.sqrt();
        let rms_freq = energy_freq.sqrt();

        let dbfs = 20.0 * (rms_freq * 2.0f32.sqrt()).log10();
        */
    }
}

pub fn freq(bin: usize) -> f32 {
    bin as f32 * (NYQ / FFT_SIZE as f32 / 2.0)
}

pub fn bin(freq: f32) -> usize {
    (freq / (NYQ / FFT_SIZE as f32 / 2.0)).floor() as usize
}

pub fn rms(bins: &[f32]) -> f32 {
    let sum: f32 = bins.iter().map(|s| s.abs().powi(2)).sum();
    (sum / bins.len() as f32).sqrt()
}

pub fn peak(samples: &[f32]) -> f32 {
    samples
        .iter()
        .map(|s| s.abs())
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Less))
        .unwrap()
}

pub fn dbfs(v: f32) -> f32 {
    20.0 * (v + 0.0001).log10()
}

pub mod prelude {
    pub use super::{freq, bin, rms, peak, dbfs};
}