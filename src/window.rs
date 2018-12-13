use {
    std::sync::Arc,

    winit::Event,
    winit::EventsLoop,
    winit::Window,
    winit::WindowEvent,
};



#[derive(Debug)]
pub enum Command {
    Quit,
    Resize(u32, u32),
}



pub struct WindowEventLoop {
    window: Arc<Window>,
    loopie: Option<EventsLoop>,
}

impl WindowEventLoop {
    pub fn new(loopie: EventsLoop, window: Arc<Window>) -> WindowEventLoop {
        WindowEventLoop {
            window: window,
            loopie: Some(loopie),
        }
    }

    pub fn read(&mut self) -> Vec<Command> {
        let mut loopie   = self.loopie.take().unwrap();
        let mut commands = vec![];

        loopie.poll_events(|event| {
            if let Event::WindowEvent { event, window_id: _ } = event {
                if let Some(x) = self.on_window_event(event) {
                    commands.push(x);
                }
            }
        });

        self.loopie = Some(loopie);
        commands
    }

    fn on_window_event(&self, event: WindowEvent) -> Option<Command> {
        match event {
            WindowEvent::CloseRequested => {
                Some(Command::Quit)
            },

            WindowEvent::Resized(size) => {
                let factor          = self.window.get_hidpi_factor();
                let (width, height) = size.to_physical(factor).into();

                Some(Command::Resize(width, height))
            }

            _ => {
                None
            },
        }
    }
}