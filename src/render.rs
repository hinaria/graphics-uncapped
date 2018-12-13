use {
    gfx_hal::Adapter,
    gfx_hal::Backbuffer,
    gfx_hal::command::ClearColor,
    gfx_hal::command::ClearValue,
    gfx_hal::device::Device,
    gfx_hal::format::ChannelType,
    gfx_hal::format::Format,
    gfx_hal::format::Swizzle,
    gfx_hal::FrameSync,
    gfx_hal::Graphics,
    gfx_hal::image::Access,
    gfx_hal::image::Layout,
    gfx_hal::image::SubresourceRange,
    gfx_hal::image::ViewKind,
    gfx_hal::Instance,
    gfx_hal::pass::Attachment,
    gfx_hal::pass::AttachmentLoadOp,
    gfx_hal::pass::AttachmentOps,
    gfx_hal::pass::AttachmentStoreOp,
    gfx_hal::pass::SubpassDependency,
    gfx_hal::pass::SubpassDesc,
    gfx_hal::pass::SubpassRef,
    gfx_hal::pool::CommandPool,
    gfx_hal::pool::CommandPoolCreateFlags,
    gfx_hal::pso::PipelineStage,
    gfx_hal::pso::Rect,
    gfx_hal::pso::Viewport,
    gfx_hal::queue::CommandQueue,
    gfx_hal::queue::Submission,
    gfx_hal::Swapchain,
    gfx_hal::SwapchainConfig,
    gfx_hal::window::Extent2D,
    gfx_hal::window::PresentMode,
    gfx_hal::window::Surface,

    crate::clock::FpsClock,
    crate::wheel::Wheel,
};



const SCREEN_BACKGROUND_SAUTRATION:  f32 = 0.7;
const SCREEN_BACKGROUND_LIGHTNESS:   f32 = 0.5;
const SCREEN_BACKGROUND_COLOR_RANGE: SubresourceRange = SubresourceRange {
    levels:  0..1,
    layers:  0..1,
    aspects: gfx_hal::format::Aspects::COLOR,
};



pub struct HueScreen {
    wheel:         Wheel,
    clock:         FpsClock,

    viewport:      Viewport,
    data:          Option<BoundData>,
    adapter:       Adapter<gfx_backend::Backend>,
    device:        gfx_backend::Device,
    surface:       gfx_backend::Surface,
    semaphore:     gfx_backend::Semaphore,
    command_pool:  CommandPool<gfx_backend::Backend, gfx_hal::queue::capability::Graphics>,
    command_queue: CommandQueue<gfx_backend::Backend, gfx_hal::queue::capability::Graphics>,
}

impl HueScreen {
    pub fn new(
        instance:       gfx_backend::Instance,
        surface:        gfx_backend::Surface,
        clock:          FpsClock,
        initial_width:  usize,
        initial_height: usize,
    ) -> HueScreen {

        let adapter = instance.enumerate_adapters().pop()
            .expect("couldn't get graphics adapter - none are available.");

        let (device, mut queues) = adapter
            .open_with::<_, Graphics>(1, |family| surface.supports_queue_family(family))
            .expect("couldn't create graphics device.");

        let command_pool = device
            .create_command_pool_typed(&queues, CommandPoolCreateFlags::empty(), 16)
            .expect("couldn't create graphics device command pool.");

        let command_queue = queues.queues.pop()
            .expect("invariant: pool has no queues, but we requested one with at least one queue.");

        let semaphore = device.create_semaphore().expect("couldn't create semaphore.");


        HueScreen {
            data:     None,
            wheel:    Wheel::new(SCREEN_BACKGROUND_SAUTRATION, SCREEN_BACKGROUND_LIGHTNESS),
            viewport: Viewport {
                rect:  Rect { x: 0, y: 0, w: initial_width as i16, h: initial_height as i16 },
                depth: 0.0..1.0,
            },

            clock:         clock,
            adapter:       adapter,
            device:        device,
            surface:       surface,
            semaphore:     semaphore,
            command_pool:  command_pool,
            command_queue: command_queue,
        }
    }

    pub fn update(&mut self) {
        self.wheel.update(0.1);
    }

    pub fn draw(&mut self) {
        if self.data.is_none() {
            self.data = Some(
                BoundData::new(
                    &self.adapter,
                    &mut self.surface,
                    &mut self.device,
                    self.viewport.rect.w as u32,
                    self.viewport.rect.h as u32));
        }


        if let Some(data) = &mut self.data {
            self.command_pool.reset();


            let BoundData { swapchain, framebuffers, pass } = data;

            if let Ok(frame_index) = swapchain.acquire_image(-1i64 as u64, FrameSync::Semaphore(&mut self.semaphore)) {
                let submission = {
                    let background   = self.wheel.rgba();
                    let viewport     = self.viewport.clone();
                    let area         = viewport.rect;
                    let mut commands = self.command_pool.acquire_command_buffer(false);

                    commands.set_scissors(0, &[area]);
                    commands.set_viewports(0, &[viewport]);

                    let buffer = &framebuffers[frame_index as usize];
                    let value  = &[ClearValue::Color(ClearColor::Float(background))];

                    commands.begin_render_pass_inline(pass, buffer, area, value).draw(0..6, 0..1);
                    commands.finish()
                };

                let submission = Submission::new()
                    .wait_on(&[(&self.semaphore, PipelineStage::BOTTOM_OF_PIPE)])
                    .submit(Some(submission));

                self.clock.update();
                self.command_queue.submit(submission, None);



                if swapchain.present(&mut self.command_queue, frame_index, &[]).is_err() {
                    BoundData::dispose(&mut self.data, &mut self.device);
                }
            } else {
                BoundData::dispose(&mut self.data, &mut self.device);
            }
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.viewport = Viewport {
            rect:  Rect { x: 0, y: 0, w: width as i16, h: height as i16 },
            depth: self.viewport.depth.clone(),
        };

        // is this necessary? check when we test on opengl.
        //
        //     #[cfg(feature = "gl")]
        //     self.surface.get_window().resize(width, height);
    }
}

impl Drop for HueScreen {
    fn drop(&mut self) {
        self.device.wait_idle().expect("couldn't wait for device idle.");
    }
}



struct BoundData {
    swapchain:    gfx_backend::Swapchain,
    framebuffers: Vec<gfx_backend::Framebuffer>,
    pass:         gfx_backend::RenderPass,
}

impl BoundData {
    fn new(
        adapter: &Adapter<gfx_backend::Backend>,
        surface: &mut gfx_backend::Surface,
        device:  &mut gfx_backend::Device,
        width:   u32,
        height:  u32,
    ) -> BoundData {

        let (capabilities, format) = match surface.compatibility(&adapter.physical_device) {
            (capabilities, None, _) => {
                (capabilities, Format::Rgba8Srgb)
            },

            (capabilities, Some(formats), _) => {
                let selected = match formats.iter().find(|x| x.base_format().1 == ChannelType::Srgb) {
                    Some(x) => *x,
                    None    => formats[0],
                };

                (capabilities, selected)
            },
        };



        let mut config = SwapchainConfig::from_caps(&capabilities, format, Extent2D { width, height });

        // [2018-12-12]: gfx-rs uses v-sync by default. `presentmode::immediate` will turn it off, but it is only
        // honoured by the vulkan backend today. we have workarounds for other backends.
        config.present_mode = PresentMode::Immediate;



        let extent = config.extent.to_extent();

        let (swapchain, backbuffer) = device
            .create_swapchain(surface, config, None)
            .expect("Can't create swapchain");



        let pass = {
            let attachment = Attachment {
                format:      Some(format),
                samples:     1,
                ops:         AttachmentOps::new(AttachmentLoadOp::Clear, AttachmentStoreOp::DontCare),
                stencil_ops: AttachmentOps::DONT_CARE,
                layouts:     Layout::Undefined..Layout::Present,
            };

            let subpass = SubpassDesc {
                colors:        &[(0, Layout::ColorAttachmentOptimal)],
                depth_stencil: None,
                inputs:        &[],
                resolves:      &[],
                preserves:     &[],
            };

            let dependency = SubpassDependency {
                passes:   SubpassRef::External..SubpassRef::Pass(0),
                stages:   PipelineStage::COLOR_ATTACHMENT_OUTPUT..PipelineStage::COLOR_ATTACHMENT_OUTPUT,
                accesses: Access::empty()..(Access::COLOR_ATTACHMENT_READ | Access::COLOR_ATTACHMENT_WRITE),
            };

            device.create_render_pass(&[attachment], &[subpass], &[dependency]).expect("couldn't create render pass.")
        };



        let framebuffers = match backbuffer {
            Backbuffer::Framebuffer(frame) => vec![frame],
            Backbuffer::Images(images) => {
                // todo: do we need to retain the `image-view` and destroy it?
                //
                // this is definitely not necessary with dx backends because they are com pointers that get
                // automatically destroyed; investigate vulkan and opengl backends.
                //
                // if not automatically dropped then we will create a lot of garbage unless we clean up after ourselves
                // (one per resize).

                images
                    .into_iter()
                    .map(|x| device.create_image_view(&x, ViewKind::D2, format, Swizzle::NO, SCREEN_BACKGROUND_COLOR_RANGE.clone()))
                    .map(|x| x.expect("couldn't create image view."))
                    .map(|x| device.create_framebuffer(&pass, Some(x), extent))
                    .map(|x| x.expect("couldn't create framebuffer."))
                    .collect(): Vec<_>
            },
        };

        BoundData { swapchain, framebuffers, pass }
    }

    fn dispose(data: &mut Option<BoundData>, device: &mut gfx_backend::Device) {
        if let Some(data) = data.take() {
            for x in data.framebuffers {
                device.destroy_framebuffer(x);
            }
        }
    }
}