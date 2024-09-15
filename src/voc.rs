use std::f64::consts::PI;
use std::ops::Range;

use crate::glottis::Glottis;
use crate::tract::Tract;

pub struct Voc {
    glottis: Glottis,
    tract: Tract,
    buf: Vec<f64>,
    pub sr: f64,
    chunk: usize,
    vocal_output_scaler: f64,
    pub counter: usize,
}

impl Voc {
    pub fn new(
        samplerate: f64,
        chunk: usize,
        vocal_output_scaler: f64,
        default_freq: f64,
        default_tenseness: f64,
        n: usize,
        nose_length: usize,
        nose_start: usize,
        tip_start: usize,
        blade_start: usize,
        epiglottis_start: usize,
        lip_start: usize,
    ) -> Self {
        let glottis = Glottis::new(samplerate, default_freq, default_tenseness);
        let tract = Tract::new(
            samplerate,
            n,
            nose_length,
            nose_start,
            tip_start,
            blade_start,
            epiglottis_start,
            lip_start,
        );
        let buf = vec![0.0; chunk];

        Voc {
            glottis,
            tract,
            buf,
            sr: samplerate,
            chunk,
            vocal_output_scaler,
            counter: 0,
        }
    }

    pub fn frequency(&self) -> f64 {
        self.glottis.freq
    }

    pub fn set_frequency(&mut self, f: f64) {
        self.glottis.freq = f;
    }
    pub fn tract_diameters(&self) -> &[f64] {
        &self.tract.target_diameter
    }

    pub fn current_tract_diameters(&self) -> &[f64] {
        &self.tract.diameter
    }

    pub fn tract_size(&self) -> usize {
        self.tract.n
    }

    pub fn nose_diameters(&self) -> &[f64] {
        &self.tract.nose_diameter
    }

    pub fn nose_size(&self) -> usize {
        self.tract.nose_length
    }

    pub fn tenseness(&self) -> f64 {
        self.glottis.tenseness
    }

    pub fn set_tenseness(&mut self, t: f64) {
        self.glottis.tenseness = t;
    }

    pub fn velum(&self) -> f64 {
        self.tract.velum_target
    }

    pub fn set_velum(&mut self, t: f64) {
        self.tract.velum_target = t;
    }

    pub fn step(&mut self) -> &[f64] {
        self.tract.reshape();
        self.tract.calculate_reflections();

        for i in 0..self.chunk {
            let mut vocal_output = 0.0;
            let lambda1 = i as f64 / self.chunk as f64;
            let lambda2 = (i as f64 + 0.5) / self.chunk as f64;

            let glot = self.glottis.compute(lambda1);

            self.tract.compute(glot, lambda1);
            vocal_output += self.tract.lip_output + self.tract.nose_output;

            self.tract.compute(glot, lambda2);
            vocal_output += self.tract.lip_output + self.tract.nose_output;

            self.buf[i] = vocal_output * self.vocal_output_scaler;
        }

        &self.buf
    }

    pub fn compute(&mut self) -> f64 {
        if self.counter == 0 {
            self.step();
        }
        let out = self.buf[self.counter];
        self.counter = (self.counter + 1) % self.chunk;
        out
    }

    pub fn set_diameters(
        &mut self,
        blade_start: usize,
        lip_start: usize,
        tip_start: usize,
        tongue_index: f64,
        tongue_diameter: f64,
    ) {
        for i in blade_start..lip_start {
            let t = 1.1 * PI * (tongue_index - i as f64) / (tip_start - blade_start) as f64;
            let fixed_tongue_diameter = 2.0 + (tongue_diameter - 2.0) / 1.5;
            let mut curve = (1.5 - fixed_tongue_diameter) * t.cos();

            if i == blade_start.saturating_sub(2) || i == lip_start.saturating_sub(1) {
                curve *= 0.8;
            }
            if i == blade_start || i == lip_start.saturating_sub(2) {
                curve *= 0.94;
            }

            self.tract.target_diameter[i] = 1.5 - curve;
        }
    }

    pub fn tongue_shape(&mut self, tongue_index: f64, tongue_diameter: f64) {
        self.set_diameters(
            self.tract.blade_start,
            self.tract.lip_start,
            self.tract.tip_start,
            tongue_index,
            tongue_diameter,
        );
    }

    pub fn set_tract_parameters(
        &mut self,
        trachea: f64,
        epiglottis: f64,
        velum: f64,
        tongue_index: f64,
        tongue_diameter: f64,
        lips: f64,
    ) {
        self.tract.set_trachea(trachea);
        self.tract.set_epiglottis(epiglottis);
        self.set_velum(velum);
        self.tongue_shape(tongue_index, tongue_diameter);
        self.tract.set_lips(lips);
    }

    pub fn set_tract_diameters(&mut self, range: Range<usize>, diameters: Vec<f64>) {
        for (i, &diameter) in range.zip(diameters.iter()) {
            if i < self.tract.target_diameter.len() {
                self.tract.target_diameter[i] = diameter;
            }
        }
    }

    pub fn play_chunk(&mut self) -> &[f64] {
        self.step()
    }
}

pub enum Mode {
    None,
    Tongue,
}

pub struct VocDemoD {
    pub sr: f64,
    pub chunk: usize,
    pub voc: Voc,
    pub gain: f64,
    pub mode: Mode,
    pub tongue_pos: f64,
    pub tongue_diam: f64,
}

impl VocDemoD {
    pub fn new(sr: f64, chunk: usize) -> Self {
        let voc = Voc::new(sr, chunk, 0.125, 400.0, 0.6, 44, 28, 17, 32, 12, 6, 39);

        VocDemoD {
            sr,
            chunk,
            voc,
            gain: 1.0,
            mode: Mode::None,
            tongue_pos: 0.0,
            tongue_diam: 0.0,
        }
    }
}
