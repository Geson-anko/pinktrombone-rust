use hound;
use std::error::Error; // WAVファイルの読み書きに使用

use pinktrombone::voc::{Mode, Voc, VocDemoD};

const SAMPLE_RATE: f64 = 44100.0;
const BUFFER_FRAMES: usize = 1024;

fn main() -> Result<(), Box<dyn Error>> {
    throat_and_lips()?;
    tongue_index()?;
    tongue_diameter()?;
    // lips_open_shut()?;
    // throat_open_shut()?;
    // frequency()?;
    // tenseness()?;
    // velum()?;
    // epiglottis()?;
    // lips()?;
    nothing()?;

    Ok(())
}

fn callme(output: &mut [(f32, f32)], vd: &mut VocDemoD) {
    for frame in output.iter_mut() {
        if vd.voc.counter == 0 {
            println!("Counter Zero");
            if matches!(vd.mode, Mode::Tongue) {
                vd.voc.tongue_shape(vd.tongue_pos, vd.tongue_diam);
            }
        }

        let tmp = vd.voc.compute() * vd.gain;
        *frame = (tmp as f32, tmp as f32);
    }
}

fn setup() -> VocDemoD {
    let mut vdd = VocDemoD::new(SAMPLE_RATE, BUFFER_FRAMES);
    vdd.voc.set_frequency(160.0);
    vdd
}

fn play_update<F>(mut update_fn: F, filename: &str) -> Result<(), Box<dyn Error>>
where
    F: FnMut(&mut Voc, f64),
{
    let mut voc = Voc::new(
        SAMPLE_RATE,
        BUFFER_FRAMES,
        0.125,
        400.0,
        0.6,
        44,
        28,
        17,
        32,
        12,
        6,
        39,
    );
    let mut x = 0.0;
    update_fn(&mut voc, x);

    let mut writer = hound::WavWriter::create(
        filename,
        hound::WavSpec {
            channels: 2,
            sample_rate: SAMPLE_RATE as u32,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        },
    )?;

    while (writer.duration() as f64) < SAMPLE_RATE * 5.0 {
        let chunk = voc.play_chunk();
        for &sample in chunk {
            // Convert f64 to f32 and write as stereo
            let sample_f32 = sample as f32;
            writer.write_sample(sample_f32)?;
            writer.write_sample(sample_f32)?;
        }

        x += 1.0;
        update_fn(&mut voc, x);
    }

    Ok(())
}

fn tongue_index() -> Result<(), Box<dyn Error>> {
    play_update(
        |voc, x| {
            let idx = (x * 0.05).sin() * 9.0 + 21.0;
            let diam = 2.75;
            voc.tongue_shape(idx, diam);
            println!("{} {}", idx, diam);
        },
        "data/tongue_index_12-30.wav",
    )
}

fn tongue_diameter() -> Result<(), Box<dyn Error>> {
    play_update(
        |voc, x| {
            let idx = 0.0_f64.sin() * 9.0 + 21.0;
            let diam = (x * 0.05).sin() * 3.5 / 2.0 + 3.5 / 2.0;
            voc.tongue_shape(idx, diam);
            println!("{} {}", idx, diam);
        },
        "data/tongue_diameter_0-3.5.wav",
    )
}

// 他のデモ関数も同様に実装します...

fn nothing() -> Result<(), Box<dyn Error>> {
    play_update(|_, _| {}, "data/nothing.wav")
}

fn throat_and_lips() -> Result<(), Box<dyn Error>> {
    let mut vdd: VocDemoD = setup();
    let mut x: f64 = 0.0;
    let mut y: f64 = 0.0;

    let mut throat = x.sin() * 1.5 / 2.0 + 0.75;
    let n_t = 7;
    vdd.voc.set_tract_diameters(0..n_t, vec![throat; n_t]);

    let mut lips = y.sin() * 1.5 / 2.0 + 0.75;
    let n_l = vdd.voc.tract_size() - 39;
    vdd.voc
        .set_tract_diameters(39..vdd.voc.tract_size(), vec![lips; n_l]);

    let mut writer = hound::WavWriter::create(
        "data/throat_and_lips.wav",
        hound::WavSpec {
            channels: 2,
            sample_rate: vdd.sr as u32,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        },
    )?;

    let mut output = vec![(0.0, 0.0); BUFFER_FRAMES];
    while (writer.duration() as f64) < vdd.sr * 5.0 {
        callme(&mut output, &mut vdd);
        for &(left, right) in &output {
            writer.write_sample(left)?;
            writer.write_sample(right)?;
        }

        x += 0.55;
        lips = x.sin() * 1.5 / 2.0 + 0.75;
        vdd.voc
            .set_tract_diameters(39..vdd.voc.tract_size(), vec![lips; n_l]);

        y += 0.5;
        throat = y.sin() * 1.5 / 2.0 + 0.75;
        vdd.voc.set_tract_diameters(0..n_t, vec![throat; n_t]);

        println!("Duration: {}, Throat: {}", writer.duration(), throat);
    }

    Ok(())
}
