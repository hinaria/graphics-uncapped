#![feature(
    integer_atomics,
    type_ascription,
)]



extern crate gfx_backend_dx11 as gfx_backend;



mod clock;
mod options;
mod render;
mod wheel;
mod window;



use {
    std::sync::Arc,
    std::time::Duration,

    winit::dpi::LogicalSize,
    winit::EventsLoop,
    winit::Window,
    winit::WindowBuilder,

    crate::clock::FpsClock,
    crate::clock::FpsClockClient,
    crate::render::HueScreen,
    crate::window::Command,
    crate::window::WindowEventLoop,
};



fn main() {
    env_logger::init();


    println!();
    println!("graphics-unbounded.");
    println!();


    let options  = options::read();
    let clock    = FpsClock::new();
    let events   = EventsLoop::new();
    let window   = WindowBuilder::new()
        .with_title("graphics-unbounded")
        .with_dimensions(LogicalSize::new(options.dimensions.width as _, options.dimensions.height as _))
        .build(&events)
        .expect("couldn't build window.");

    let instance = gfx_backend::Instance::create("graphics-unbounded", 1);
    let surface  = instance.create_surface(&window);
    let window   = Arc::new(window);



    {
        let clock   = clock.client();
        let window  = window.clone();
        let backend = options.backend;

        std::thread::spawn(move || run_status_thread(backend, window, clock));
    }



    let mut screen   = HueScreen::new(instance, surface, clock, options.dimensions.width, options.dimensions.height);
    let mut commands = WindowEventLoop::new(events, window);

    loop {
        for command in commands.read() {
            match command {
                Command::Quit                  => std::process::exit(0),
                Command::Resize(width, height) => screen.resize(width, height),
            }
        }

        screen.update();
        screen.draw();
    }
}



fn run_status_thread(backend: &'static str, window: Arc<Window>, clock: FpsClockClient) {
    let delay = Duration::from_millis(1000);

    loop {
        let fps   = clock.fps();
        let title = format!("{} - {} fps - graphics-unbounded", backend, fps);

        window.set_title(&title);
        std::thread::sleep(delay);
    }
}
