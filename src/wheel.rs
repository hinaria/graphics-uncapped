pub struct Wheel {
    hue:        f32,
    saturation: f32,
    lightness:  f32,
}

impl Wheel {
    pub fn new(saturation: f32, lightness: f32) -> Wheel {
        Wheel { hue: 0.0, saturation, lightness }
    }

    pub fn update(&mut self, increment: f32) {
        self.hue = (self.hue + increment) % 360.0;
    }

    pub fn rgba(&self) -> [f32; 4] {
        let (r, g, b) = as_rgb(self.hue, self.saturation, self.lightness);

        [r, g, b, 1.0]
    }
}




// https://github.com/bthomson/go-color
fn as_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    if s == 0.0 {
        (l, l, l)
    } else {
        let h = h / 360.0;
        let s = s;
        let l = l;

        let q = match l < 0.5 {
            true  => l * (1.0 + s),
            false => l + s - (l * s),
        };

        let p = 2.0 * l - q;

        let r = hue_as_rgb(p, q, h + 1.0 / 3.0);
        let g = hue_as_rgb(p, q, h);
        let b = hue_as_rgb(p, q, h - 1.0 / 3.0);

        (r, g, b)
    }
}

// https://github.com/jariz/vibrant.js
fn hue_as_rgb(p: f32, q: f32, t: f32) -> f32 {
    let t = match t {
        t if t < 0.0 => t + 1.0,
        t if t > 1.0 => t - 1.0,
        t            => t,
    };

    match true {
        _ if t < 1.0 / 6.0 => p + (q - p) * 6.0 * t,
        _ if t < 1.0 / 2.0 => q,
        _ if t < 2.0 / 3.0 => p + (q - p) * (2.0 / 3.0 - t) * 6.0,
        _                  => p,
    }
}
