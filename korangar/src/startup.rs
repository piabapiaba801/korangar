use std::iter;

use korangar_video::ivf::Ivf;
use wgpu::{Queue, TextureFormat};

use crate::graphics::{Color, ScreenClip, ScreenPosition, ScreenSize};
use crate::loaders::TextureLoader;
use crate::renderer::{GameInterfaceRenderer, SpriteRenderer};
use crate::world::{Video, VideoFrame};

const INTRO_VIDEO_PATH: &str = "client/branding/intro.ivf";

/// The login and character selection screens do not need a Ragnarok map.
pub struct StartupScene {
    video: Option<Video>,
    video_size: Option<ScreenSize>,
}

impl StartupScene {
    pub fn new(texture_loader: &TextureLoader) -> Self {
        let video_data = std::fs::read(INTRO_VIDEO_PATH).ok();
        let (video, video_size) = video_data
            .as_deref()
            .and_then(|data| Self::load_video(texture_loader, data))
            .map_or((None, None), |(video, size)| (Some(video), Some(size)));

        Self { video, video_size }
    }

    fn load_video(texture_loader: &TextureLoader, data: &[u8]) -> Option<(Video, ScreenSize)> {
        let mut ivf = Ivf::new(data).ok()?;
        if !matches!(ivf.four_cc(), [b'A' | b'a', b'V' | b'v', b'0', b'1']) {
            return None;
        }

        let width = u32::from(ivf.width());
        let height = u32::from(ivf.height());
        if width == 0 || height == 0 || width > 3840 || height > 2160 || ivf.header().timebase_denominator == 0 {
            return None;
        }

        let timescale = ivf.header().timebase_numerator as f64 / ivf.header().timebase_denominator as f64;
        if !timescale.is_finite() || timescale <= 0.0 {
            return None;
        }

        let frames = Vec::from_iter(iter::from_fn(|| {
            ivf.read_frame().ok().flatten().map(|frame| VideoFrame {
                timestamp: frame.timestamp as i64,
                packet: frame.packet.into(),
            })
        }));
        if frames.is_empty() {
            return None;
        }

        let texture = texture_loader.create_raw("freokro startup video", width, height, 1, TextureFormat::Rgba8UnormSrgb, false);
        let video = Video::new(width, height, timescale, frames, texture);
        let size = ScreenSize {
            width: width as f32,
            height: height as f32,
        };
        Some((video, size))
    }

    pub fn advance(&mut self, queue: &Queue, delta_time: f64) {
        if let Some(video) = &mut self.video {
            if video.should_show_next_frame(delta_time) {
                video.update_texture(queue);
            }
            video.check_for_next_frame();
        }
    }

    pub fn render(&self, renderer: &GameInterfaceRenderer, screen: ScreenSize) {
        renderer.render_rectangle(ScreenPosition::default(), screen, Color::rgb_u8(8, 16, 13));

        if let (Some(video), Some(video_size)) = (&self.video, self.video_size) {
            let scale = (screen.width / video_size.width).min(screen.height / video_size.height);
            let size = ScreenSize {
                width: video_size.width * scale,
                height: video_size.height * scale,
            };
            let position = ScreenPosition {
                left: (screen.width - size.width) * 0.5,
                top: (screen.height - size.height) * 0.5,
            };
            renderer.render_sprite(
                video.get_texture().clone(),
                position,
                size,
                ScreenClip::default(),
                Color::WHITE,
                true,
            );
        }

    }
}
