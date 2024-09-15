use rand::Rng;
use std::f64::consts::PI;

pub struct Glottis {
    pub freq: f64,
    pub tenseness: f64,
    rd: f64,
    waveform_length: f64,
    time_in_waveform: f64,
    alpha: f64,
    e0: f64,
    epsilon: f64,
    shift: f64,
    delta: f64,
    te: f64,
    omega: f64,
    t: f64,
    pub sr: f64,
}

impl Glottis {
    pub fn new(sr: f64, default_freq: f64, default_tenseness: f64) -> Self {
        let mut glottis = Glottis {
            freq: default_freq,
            tenseness: default_tenseness,
            rd: 0.0,
            waveform_length: 0.0,
            time_in_waveform: 0.0,
            alpha: 0.0,
            e0: 0.0,
            epsilon: 0.0,
            shift: 0.0,
            delta: 0.0,
            te: 0.0,
            omega: 0.0,
            t: 1.0 / sr,
            sr,
        };
        glottis.setup_waveform(0.0);
        glottis
    }

    pub fn setup_waveform(&mut self, _lambda: f64) {
        self.rd = 3.0 * (1.0 - self.tenseness);
        self.waveform_length = 1.0 / self.freq;

        let mut rd = self.rd;
        if rd < 0.5 {
            rd = 0.5;
        }
        if rd > 2.7 {
            rd = 2.7;
        }

        let ra = -0.01 + 0.048 * rd;
        let rk = 0.224 + 0.118 * rd;
        let rg = (rk / 4.0) * (0.5 + 1.2 * rk) / (0.11 * rd - ra * (0.5 + 1.2 * rk));

        let ta = ra;
        let tp = 1.0 / (2.0 * rg);
        let te = tp + tp * rk;

        self.epsilon = 1.0 / ta;
        self.shift = (-self.epsilon * (1.0 - te)).exp();
        self.delta = 1.0 - self.shift;

        let mut rhs_integral = (1.0 / self.epsilon) * (self.shift - 1.0) + (1.0 - te) * self.shift;
        rhs_integral = rhs_integral / self.delta;
        let lower_integral = -(te - tp) / 2.0 + rhs_integral;
        let upper_integral = -lower_integral;

        self.omega = PI / tp;
        let s = (self.omega * te).sin();

        let y = -PI * s * upper_integral / (tp * 2.0);
        let z = y.ln();
        self.alpha = z / (tp / 2.0 - te);
        self.e0 = -1.0 / (s * (self.alpha * te).exp());

        self.te = te;
    }

    pub fn compute(&mut self, lambda: f64) -> f64 {
        let intensity = 1.0;

        self.time_in_waveform += self.t;

        if self.time_in_waveform > self.waveform_length {
            self.time_in_waveform -= self.waveform_length;
            self.setup_waveform(lambda);
        }

        let t = self.time_in_waveform / self.waveform_length;

        let out = if t > self.te {
            (-(-self.epsilon * (t - self.te)).exp() + self.shift) / self.delta
        } else {
            self.e0 * (self.alpha * t).exp() * (self.omega * t).sin()
        };

        let noise: f64 = rand::thread_rng().gen_range(-1.0..1.0);
        let aspiration = intensity * (1.0 - self.tenseness.sqrt()) * 0.3 * noise;

        out + aspiration * 0.2
    }
}
