use {
    clap::App,
    clap::Arg,
};



const DEFAULT_DIMENSIONS: &str = "720p";



pub struct SimpleOptions {
    pub backend:    &'static str,
    pub dimensions: Dimensions,
}

pub struct Dimensions {
    pub width:  usize,
    pub height: usize,
}



pub fn read() -> SimpleOptions {
    let matches = App::new("graphics-unbounded")
        .version("0.0.1")
        .author("annie <a@hinaria.com>")
        .about("graphics unbounded.")
        .arg(
            Arg::with_name("dimensions")
                .help("window dimensions (eg, 3840x2160 or 4k).")
                .long_help("window dimensions as a width x height specification (3840 x 2160), or one of: 720p, 1080p, 4k, 8k.")
                .takes_value(true)
                .required(false)
        ).get_matches();

    let dimensions = matches.value_of("dimensions").unwrap_or(DEFAULT_DIMENSIONS);
    let dimensions = extract_dimensions(dimensions);
    let backend    = "henlo :3";

    SimpleOptions { dimensions, backend }
}

fn extract_dimensions(value: &str) -> Dimensions {
    let (width, height) = match value {
        "1080p"  => (1920, 1080),
        "720p"   => (1280, 720),
        "4k"     => (3840, 2160),
        "8k"     => (7680, 4320),
        explicit => {
            let components = explicit
                .split("x")
                .map(|x| x.trim())
                .map(|x| x.parse().expect("dimension specification must be a number."))
                .collect(): Vec<usize>;

            if components.len() != 2 {
                panic!("an explicit dimension specification must be in the format of `<usize> x <usize>`.");
            }

            (components[0], components[1])
        }
    };

    Dimensions { width, height }
}