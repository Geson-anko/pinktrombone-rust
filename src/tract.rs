use crate::consts::BASE_N;
use crate::transient::TransientPool;

fn move_towards(current: f64, target: f64, amt_up: f64, amt_down: f64) -> f64 {
    if current < target {
        (current + amt_up).min(target)
    } else {
        (current - amt_down).max(target)
    }
}

pub struct Tract {
    pub sr: f64,
    pub n: usize,
    pub blade_start: usize,
    pub lip_start: usize,
    pub epiglottis_start: usize,

    pub diameter: Vec<f64>,
    rest_diameter: Vec<f64>,
    pub target_diameter: Vec<f64>,
    new_diameter: Vec<f64>,
    r: Vec<f64>,
    l: Vec<f64>,
    reflection: Vec<f64>,
    new_reflection: Vec<f64>,
    junction_outl: Vec<f64>,
    junction_outr: Vec<f64>,
    a: Vec<f64>,

    pub nose_length: usize,
    pub nose_start: usize,
    pub tip_start: usize,
    nosel: Vec<f64>,
    noser: Vec<f64>,
    nose_junc_outl: Vec<f64>,
    nose_junc_outr: Vec<f64>,
    nose_reflection: Vec<f64>,
    pub nose_diameter: Vec<f64>,
    nose_a: Vec<f64>,

    reflection_left: f64,
    reflection_right: f64,
    reflection_nose: f64,

    new_reflection_left: f64,
    new_reflection_right: f64,
    new_reflection_nose: f64,

    pub velum_target: f64,

    glottal_reflection: f64,
    lip_reflection: f64,
    last_obstruction: i32,
    pub fade: f64,
    movement_speed: f64,
    pub lip_output: f64,
    pub nose_output: f64,
    block_time: f64,

    tpool: TransientPool,
    t: f64,
}

impl Tract {
    pub fn new(
        samplerate: f64,
        n: usize,
        nose_length: usize,
        nose_start: usize,
        tip_start: usize,
        blade_start: usize,
        epiglottis_start: usize,
        lip_start: usize,
    ) -> Self {
        let mut tract = Tract {
            sr: samplerate,
            n,
            blade_start,
            lip_start,
            epiglottis_start,
            diameter: vec![0.0; n],
            rest_diameter: vec![0.0; n],
            target_diameter: vec![0.0; n],
            new_diameter: vec![0.0; n],
            r: vec![0.0; n],
            l: vec![0.0; n],
            reflection: vec![0.0; n + 1],
            new_reflection: vec![0.0; n + 1],
            junction_outl: vec![0.0; n + 1],
            junction_outr: vec![0.0; n + 1],
            a: vec![0.0; n],
            nose_length,
            nose_start,
            tip_start,
            nosel: vec![0.0; nose_length],
            noser: vec![0.0; nose_length],
            nose_junc_outl: vec![0.0; nose_length + 1],
            nose_junc_outr: vec![0.0; nose_length + 1],
            nose_reflection: vec![0.0; nose_length + 1],
            nose_diameter: vec![0.0; nose_length],
            nose_a: vec![0.0; nose_length],
            reflection_left: 0.0,
            reflection_right: 0.0,
            reflection_nose: 0.0,
            new_reflection_left: 0.0,
            new_reflection_right: 0.0,
            new_reflection_nose: 0.0,
            velum_target: 0.01,
            glottal_reflection: 0.75,
            lip_reflection: -0.85,
            last_obstruction: -1,
            fade: 0.0,
            movement_speed: 15.0,
            lip_output: 0.0,
            nose_output: 0.0,
            block_time: 512.0 / samplerate,
            tpool: TransientPool::new(),
            t: 1.0 / samplerate,
        };

        tract.calculate_diameters();
        tract.calculate_nose_diameter();
        tract.calculate_reflections();
        tract.calculate_nose_reflections();
        tract.nose_diameter[0] = tract.velum_target;

        tract
    }

    fn calculate_diameters(&mut self) {
        for i in 0..self.n {
            let diameter = if i
                < ((1 + self.epiglottis_start) as f64 * self.n as f64 / BASE_N as f64 - 0.5)
                    as usize
            {
                0.6
            } else if i < (self.blade_start as f64 * self.n as f64 / BASE_N as f64) as usize {
                1.1
            } else {
                1.5
            };

            self.diameter[i] = diameter;
            self.rest_diameter[i] = diameter;
            self.target_diameter[i] = diameter;
            self.new_diameter[i] = diameter;
        }
    }

    fn calculate_nose_diameter(&mut self) {
        for i in 0..self.nose_length {
            let d = 2.0 * (i as f64 / self.nose_length as f64);
            let diameter = if d < 1.0 {
                0.4 + 1.6 * d
            } else {
                0.5 + 1.5 * (2.0 - d)
            };
            self.nose_diameter[i] = diameter.min(1.9);
        }
    }

    pub fn calculate_reflections(&mut self) {
        for i in 0..self.n {
            self.a[i] = self.diameter[i].powi(2);
        }

        for i in 1..self.n {
            self.reflection[i] = self.new_reflection[i];
            self.new_reflection[i] = if self.a[i] == 0.0 {
                0.999
            } else {
                (self.a[i - 1] - self.a[i]) / (self.a[i - 1] + self.a[i])
            };
        }

        self.reflection_left = self.new_reflection_left;
        self.reflection_right = self.new_reflection_right;
        self.reflection_nose = self.new_reflection_nose;

        let sum = self.a[self.nose_start] + self.a[self.nose_start + 1] + self.nose_a[0];
        self.new_reflection_left = (2.0 * self.a[self.nose_start] - sum) / sum;
        self.new_reflection_right = (2.0 * self.a[self.nose_start + 1] - sum) / sum;
        self.new_reflection_nose = (2.0 * self.nose_a[0] - sum) / sum;
    }

    fn calculate_nose_reflections(&mut self) {
        for i in 0..self.nose_length {
            self.nose_a[i] = self.nose_diameter[i].powi(2);
        }

        for i in 1..self.nose_length {
            self.nose_reflection[i] =
                (self.nose_a[i - 1] - self.nose_a[i]) / (self.nose_a[i - 1] + self.nose_a[i]);
        }
    }

    pub fn compute(&mut self, input: f64, lambda: f64) {
        let mut transients_to_remove: Vec<usize> = Vec::new();

        {
            let transients = self.tpool.get_valid_transients();
            for n in transients {
                let amp = n.strength * 2.0f64.powf(-1.0 * n.exponent * n.time_alive);
                self.l[n.position] += amp * 0.5;
                self.r[n.position] += amp * 0.5;
                n.time_alive += self.t * 0.5;
                if n.time_alive > n.lifetime {
                    transients_to_remove.push(n.id)
                }
            }
        }

        for id in transients_to_remove {
            self.tpool.remove(id);
        }

        self.junction_outr[0] = self.l[0] * self.glottal_reflection + input;
        self.junction_outl[self.n] = self.r[self.n - 1] * self.lip_reflection;

        self.calculate_junctions(lambda);

        let i = self.nose_start;
        let r = self.new_reflection_left * (1.0 - lambda) + self.reflection_left * lambda;
        self.junction_outl[i] = r * self.r[i - 1] + (1.0 + r) * (self.nosel[0] + self.l[i]);
        let r = self.new_reflection_right * (1.0 - lambda) + self.reflection_right * lambda;
        self.junction_outr[i] = r * self.l[i] + (1.0 + r) * (self.r[i - 1] + self.nosel[0]);
        let r = self.new_reflection_nose * (1.0 - lambda) + self.reflection_nose * lambda;
        self.nose_junc_outr[0] = r * self.nosel[0] + (1.0 + r) * (self.l[i] + self.r[i - 1]);

        self.calculate_lip_output();

        self.nose_junc_outl[self.nose_length] =
            self.noser[self.nose_length - 1] * self.lip_reflection;

        self.calculate_nose_junc_out();

        self.calculate_nose();
        self.nose_output = self.noser[self.nose_length - 1];
    }

    fn calculate_nose(&mut self) {
        self.noser[..self.nose_length].copy_from_slice(&self.nose_junc_outr[..self.nose_length]);
        self.nosel[..self.nose_length].copy_from_slice(&self.nose_junc_outl[1..=self.nose_length]);
    }

    fn calculate_nose_junc_out(&mut self) {
        for i in 1..self.nose_length {
            let w = self.nose_reflection[i] * (self.noser[i - 1] + self.nosel[i]);
            self.nose_junc_outr[i] = self.noser[i - 1] - w;
            self.nose_junc_outl[i] = self.nosel[i] + w;
        }
    }

    fn calculate_lip_output(&mut self) {
        for i in 0..self.n {
            self.r[i] = self.junction_outr[i] * 0.999;
            self.l[i] = self.junction_outl[i + 1] * 0.999;
        }
        self.lip_output = self.r[self.n - 1];
    }

    fn calculate_junctions(&mut self, lambda: f64) {
        for i in 1..self.n {
            let r = self.reflection[i] * (1.0 - lambda) + self.new_reflection[i] * lambda;
            let w = r * (self.r[i - 1] + self.l[i]);
            self.junction_outr[i] = self.r[i - 1] - w;
            self.junction_outl[i] = self.l[i] + w;
        }
    }

    pub fn reshape(&mut self) {
        let mut current_obstruction = -1;
        let amount = self.block_time * self.movement_speed;

        for i in 0..self.n {
            let slow_return = if i < self.nose_start {
                0.6
            } else if i >= self.tip_start {
                1.0
            } else {
                0.6 + 0.4 * (i as f64 - self.nose_start as f64)
                    / (self.tip_start as f64 - self.nose_start as f64)
            };

            let diameter = self.diameter[i];
            let target_diameter = self.target_diameter[i];

            if diameter < 0.001 {
                current_obstruction = i as i32;
            }

            self.diameter[i] = move_towards(
                diameter,
                target_diameter,
                slow_return * amount,
                2.0 * amount,
            );
        }

        if self.last_obstruction > -1 && current_obstruction == -1 && self.nose_a[0] < 0.05 {
            self.tpool.append(self.last_obstruction as usize);
        }
        self.last_obstruction = current_obstruction;

        self.nose_diameter[0] = move_towards(
            self.nose_diameter[0],
            self.velum_target,
            amount * 0.25,
            amount * 0.1,
        );
        self.nose_a[0] = self.nose_diameter[0].powi(2);
    }

    // Getter methods
    pub fn lip_start(&self) -> usize {
        self.lip_start
    }

    pub fn blade_start(&self) -> usize {
        self.blade_start
    }

    pub fn epiglottis_start(&self) -> usize {
        self.epiglottis_start
    }

    // Getter and setter methods for lips, epiglottis, and trachea
    pub fn lips(&self) -> f64 {
        self.target_diameter[self.lip_start..].iter().sum::<f64>()
            / (self.n - self.lip_start) as f64
    }

    pub fn set_lips(&mut self, d: f64) {
        for i in self.lip_start..self.n {
            self.target_diameter[i] = d;
        }
    }

    pub fn epiglottis(&self) -> f64 {
        self.target_diameter[self.epiglottis_start..self.blade_start]
            .iter()
            .sum::<f64>()
            / (self.blade_start - self.epiglottis_start) as f64
    }

    pub fn set_epiglottis(&mut self, d: f64) {
        for i in self.epiglottis_start..self.blade_start {
            self.target_diameter[i] = d;
        }
    }

    pub fn trachea(&self) -> f64 {
        self.target_diameter[..self.epiglottis_start]
            .iter()
            .sum::<f64>()
            / self.epiglottis_start as f64
    }

    pub fn set_trachea(&mut self, d: f64) {
        for i in 0..self.epiglottis_start {
            self.target_diameter[i] = d;
        }
    }
}
