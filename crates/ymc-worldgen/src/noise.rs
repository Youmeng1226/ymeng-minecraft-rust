/// Classic 2D Perlin noise with a seeded permutation table.
#[derive(Clone)]
pub struct Perlin {
    perm: [u8; 512],
}

impl Perlin {
    pub fn new(seed: u64) -> Self {
        let mut table: [u8; 256] = std::array::from_fn(|i| i as u8);
        let mut rng = SplitMix64(seed);
        for i in (1..256).rev() {
            let j = (rng.next() % (i as u64 + 1)) as usize;
            table.swap(i, j);
        }
        let perm = std::array::from_fn(|i| table[i & 255]);
        Self { perm }
    }

    /// Noise value in roughly `[-1, 1]`.
    pub fn get(&self, x: f64, y: f64) -> f64 {
        let xi = x.floor() as i64 & 255;
        let yi = y.floor() as i64 & 255;
        let xf = x - x.floor();
        let yf = y - y.floor();
        let u = fade(xf);
        let v = fade(yf);

        let p = &self.perm;
        let (xi, yi) = (xi as usize, yi as usize);
        let aa = p[p[xi] as usize + yi];
        let ab = p[p[xi] as usize + yi + 1];
        let ba = p[p[xi + 1] as usize + yi];
        let bb = p[p[xi + 1] as usize + yi + 1];

        let x1 = lerp(grad(aa, xf, yf), grad(ba, xf - 1.0, yf), u);
        let x2 = lerp(grad(ab, xf, yf - 1.0), grad(bb, xf - 1.0, yf - 1.0), u);
        lerp(x1, x2, v)
    }

    /// Fractal Brownian motion: several octaves of noise summed together.
    pub fn fbm(&self, x: f64, y: f64, octaves: u32, lacunarity: f64, gain: f64) -> f64 {
        let mut sum = 0.0;
        let mut amp = 1.0;
        let mut freq = 1.0;
        let mut norm = 0.0;
        for _ in 0..octaves {
            sum += self.get(x * freq, y * freq) * amp;
            norm += amp;
            amp *= gain;
            freq *= lacunarity;
        }
        sum / norm
    }
}

fn fade(t: f64) -> f64 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + t * (b - a)
}

fn grad(hash: u8, x: f64, y: f64) -> f64 {
    match hash & 7 {
        0 => x + y,
        1 => -x + y,
        2 => x - y,
        3 => -x - y,
        4 => x,
        5 => -x,
        6 => y,
        _ => -y,
    }
}

/// Small deterministic RNG used for seeding; avoids an external dependency.
pub(crate) struct SplitMix64(pub u64);

impl SplitMix64 {
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_for_same_seed() {
        let a = Perlin::new(42);
        let b = Perlin::new(42);
        assert_eq!(a.get(1.3, 7.9), b.get(1.3, 7.9));
    }

    #[test]
    fn stays_in_range() {
        let noise = Perlin::new(7);
        for i in 0..1000 {
            let v = noise.fbm(i as f64 * 0.137, i as f64 * 0.071, 4, 2.0, 0.5);
            assert!((-1.0..=1.0).contains(&v), "{v}");
        }
    }

    #[test]
    fn zero_at_lattice_points() {
        let noise = Perlin::new(1);
        assert_eq!(noise.get(3.0, 5.0), 0.0);
    }
}
