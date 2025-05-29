#![windows_subsystem = "windows"]

mod dll;
mod logging;
mod style;

#[cfg(feature = "embed-dll")]
use dll::extract_dll;

use dll::{ExpanMod, load_expan_module};
use iced::{
    Alignment, Font, Subscription, Task, Theme, padding, time,
    widget::{Container, checkbox, column, container, row, slider, text, text_input},
    window::Settings,
};
use logging::log_amp_data;
use std::{
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use style::{build_icon, styled_text};

const INTER_REGULAR: &[u8] = include_bytes!("../fonts/Inter_24pt-Regular.ttf");

fn get_temp_path() -> Result<PathBuf, anyhow::Error> {
    let mut path = match std::env::var_os("LOCALAPPDATA") {
        Some(local_app) => PathBuf::from(local_app),
        None => return Err(anyhow::Error::msg("No LOCALAPPDATA var!")),
    };
    path.push("astral");
    std::fs::create_dir_all(&path).ok();
    Ok(path)
}

fn update_amp_data(expan_mod: &ExpanMod, amp_data: &mut AmperageData) -> Result<(), anyhow::Error> {
    let res = expan_mod.get_amperage_info(amp_data.gpu_index, &mut amp_data.pin_value_buffer)?;
    if res != 0 {
        tracing::warn!(
            "Failed to fetch data: {:?}, {}",
            amp_data.pin_value_buffer,
            res
        );
    }
    amp_data.status_code = res;
    amp_data.timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    Ok(())
}

fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting astral-flow-meter application!");

    #[cfg(feature = "embed-dll")]
    extract_dll()?;

    iced::application("Astral Amp Info", AppState::update, AppState::view)
        .subscription(AppState::subscription)
        .window(Settings {
            size: (400.0, 280.0).into(),
            resizable: false,
            decorations: true,
            icon: Some(build_icon()?),
            ..Default::default()
        })
        .font(INTER_REGULAR)
        .theme(|_| Theme::Dark)
        .run_with(AppState::new)
        .map_err(anyhow::Error::from)
}

#[derive(Debug, Clone, Default)]
pub struct AmperageData {
    pub gpu_index: i32,
    pub timestamp: u64,
    pub pin_value_buffer: [f32; 6],
    pub status_code: i32,
}

#[derive(Debug)]
struct AppState {
    expan_module: Option<ExpanMod>,
    amperage_data: AmperageData,
    polling_rate: Duration,
    slider_value_ms: u16,
    enable_logging: bool,
    log_path: PathBuf,
}

impl AppState {
    fn new() -> (Self, Task<Message>) {
        #[cfg(feature = "embed-dll")]
        let dll_path = {
            let mut dll_path = get_temp_path().expect("Temp path should be available");
            dll_path.push("ExpanModule_temp.dll");
            dll_path
        };

        let paths = [
            #[cfg(feature = "embed-dll")]
            dll_path,
            PathBuf::from("ExpanModule.dll"),
            PathBuf::from(r"C:\Program Files (x86)\ASUS\GPUTweakIII\ExpanModule.dll"),
        ];
        let expan_module = load_expan_module(&paths);

        let mut amperage_data = AmperageData {
            gpu_index: 1,
            ..Default::default()
        };
        if let Some(expan_mod) = expan_module.as_ref() {
            if let Err(e) = update_amp_data(expan_mod, &mut amperage_data) {
                tracing::error!("Failed to fetch amperage data in the beginning: {}", e);
            }
        } else {
            tracing::warn!("ExpanModule.dll not found!");
        }

        let polling_rate = Duration::from_millis(1000);
        let slider_value_ms = polling_rate.as_millis() as u16;

        let path = if let Ok(mut path) = get_temp_path() {
            path.push("astral-amp.log");
            path
        } else {
            PathBuf::new()
        };

        (
            Self {
                expan_module,
                amperage_data,
                polling_rate,
                enable_logging: false,
                log_path: path,
                slider_value_ms,
            },
            Task::none(),
        )
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(self.polling_rate).map(|_| Message::UpdateAmpData)
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::UpdateAmpData => {
                if let Some(expan_module) = &self.expan_module {
                    match update_amp_data(expan_module, &mut self.amperage_data) {
                        Ok(data) => data,
                        Err(e) => {
                            tracing::error!("Failed to update amp data: {}", e);
                        }
                    };

                    if self.enable_logging {
                        if let Err(e) = log_amp_data(&self.log_path, &self.amperage_data) {
                            tracing::error!(
                                "Failed to log data to {}: {}",
                                self.log_path.display(),
                                e
                            );
                        }
                    }
                }
            }
            Message::SliderChanged(new_value) => {
                self.polling_rate = Duration::from_millis(new_value as u64);
                self.slider_value_ms = new_value;
            }
            Message::ToggleLogging(value) => {
                self.enable_logging = value;
            }
            Message::LogPathChanged(path) => {
                let log_path = PathBuf::from(path);
                self.log_path = log_path;
            }
            Message::ExpanModulePathChanged(path) => {
                let expan_module_path = PathBuf::from(path);
                let expan_module = load_expan_module(&[expan_module_path]);
                let mut amperage_data = AmperageData::default();
                if let Some(expan_mod) = expan_module.as_ref() {
                    if let Err(e) =
                        expan_mod.get_amperage_info(1, &mut amperage_data.pin_value_buffer)
                    {
                        tracing::error!("Failed to get amperage data: {}", e);
                    }
                } else {
                    tracing::warn!("ExpanModule.dll not found!");
                }
                self.expan_module = expan_module;
            }
        }
    }

    fn view(&self) -> Container<Message> {
        if self.expan_module.is_none() {
            let text = text("Please provide ExpanModule.dll path:");
            let expan_module_path = container(
                text_input("/path/to/the/ExpanModule.dll", "")
                    .on_input(Message::ExpanModulePathChanged)
                    .width(360),
            )
            .padding(padding::bottom(15));
            return container(column!(text, expan_module_path));
        }

        // Create left column (first 3 pins)
        let left_column = column(
            self.amperage_data
                .pin_value_buffer
                .iter()
                .take(3)
                .enumerate()
                .map(|(i, value)| styled_text(i + 1, *value))
                .collect::<Vec<_>>(),
        )
        .spacing(2)
        .align_x(Alignment::Start);
        // Create right column (last 3 pins)
        let right_column = column(
            self.amperage_data
                .pin_value_buffer
                .iter()
                .skip(3)
                .take(3)
                .enumerate()
                .map(|(i, value)| styled_text(i + 4, *value))
                .collect::<Vec<_>>(),
        )
        .spacing(4)
        .align_x(Alignment::Start);

        // Combine columns into a row
        let pins = row![left_column, right_column]
            .spacing(10)
            .align_y(Alignment::Center);

        let pins = container(pins)
            .padding(padding::top(20).left(20).right(20).bottom(15))
            .align_x(Alignment::Start);

        // Sliders
        let current_slider_value = container(
            text(format!("Polling rate: {} ms", self.slider_value_ms))
                .font(Font::with_name("Inter 24pt")),
        )
        .padding(padding::bottom(5));

        let slider_ = container(
            slider(500u16..=3000u16, self.slider_value_ms, |v| {
                Message::SliderChanged(v)
            })
            .default(1000u16)
            .shift_step(50u16)
            .width(360),
        );

        // Logging
        let logging_path = container(
            text_input("/path/to/the/log.file", &self.log_path.to_string_lossy())
                .on_input(Message::LogPathChanged)
                .width(360),
        )
        .padding(padding::bottom(15));

        let checkbox_ = checkbox("Enable logging", self.enable_logging)
            .on_toggle(Message::ToggleLogging)
            .font(Font::with_name("Inter 24pt"));

        container(column!(
            row![pins],
            row![column!(current_slider_value, slider_)].padding(padding::left(20).top(0)),
            row![column!(logging_path, checkbox_)].padding(padding::left(20).bottom(20).top(15))
        ))
    }
}

#[derive(Debug, Clone)]
enum Message {
    UpdateAmpData,
    LogPathChanged(String),
    ToggleLogging(bool),
    SliderChanged(u16),
    ExpanModulePathChanged(String),
}
