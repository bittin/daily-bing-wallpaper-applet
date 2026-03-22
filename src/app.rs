// SPDX-License-Identifier: MPL-2.0

use cosmic::iced::{window::Id, Limits, Subscription};
use cosmic::iced_winit::commands::popup::{destroy_popup, get_popup};
use cosmic::prelude::*;
use cosmic::widget;
use futures_util::SinkExt;

use std::path::PathBuf;

use std::fs;
use std::env;

use chrono::Local;
use chrono::NaiveDate;
use chrono::Timelike;
use chrono::Datelike;

use cosmic::iced::Length;
use cosmic::iced::Alignment;

use cosmic::iced::widget::image;

use std::time::Duration;

#[derive(Debug, Clone)]
struct Wallpaper {
    url: String,
    title: String,
    urlbase: String,
    copyright: String,
    date: String,
}

#[derive(Default)]
pub struct AppModel {
    core: cosmic::Core,
    popup: Option<Id>,

    wallpaper: Option<Wallpaper>,
    loading: bool,
    hovered: Option<&'static str>,
    last_run_day: Option<u32>,
}

#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),

    FetchWallpaper,
    WallpaperLoaded(Result<Wallpaper, String>),

    SetWallpaper,
    WallpaperSet,

    SubscriptionChannel,

    HoverItem(&'static str),
    UnhoverItem,
    Tick,
}

async fn fetch_bing() -> Result<Wallpaper, String> {
    let resp = reqwest::get(
        "https://www.bing.com/HPImageArchive.aspx?format=js&idx=0&n=1",
    )
    .await
    .map_err(|e| e.to_string())?;

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

    let img = &json["images"][0];

    let urlbase = img["urlbase"].as_str().unwrap_or("").to_string();

    let copyright = img["copyright"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let url = format!(
        "https://www.bing.com{}_UHD.jpg",
        urlbase
    );

    let date = img["startdate"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let title = img["title"]
        .as_str()
        .unwrap_or("Bing Wallpaper")
        .to_string();

    Ok(Wallpaper { url, urlbase, title, copyright, date })
}

fn format_date_pretty(date: &str) -> String {
    if let Ok(parsed) = NaiveDate::parse_from_str(date, "%Y%m%d") {
        parsed.format("%B %-d, %Y").to_string()
    } else {
        date.to_string()
    }
}

fn filename_from_urlbase(urlbase: &str) -> String {
    // find "id=" and take everything after it
    let mut id_part = urlbase
        .split("id=")
        .nth(1)
        .unwrap_or("bing_wallpaper");
    let after_dot = id_part
        .split(".")
        .nth(1)
        .unwrap_or("bing_wallpaper");

    format!("{}-{}.jpg", Local::now().format("%Y%m%d"), after_dot)
}

async fn download_image(url: String, path: PathBuf) -> Result<(), String> {
    let bytes = reqwest::get(&url)
        .await
        .map_err(|e| e.to_string())?
        .bytes()
        .await
        .map_err(|e| e.to_string())?;

    std::fs::write(&path, &bytes).map_err(|e| e.to_string())?;

    Ok(())
}

fn set_wallpaper(path: &PathBuf) -> Result<(), String> {
    let home = env::var("HOME").map_err(|e| e.to_string())?;

    let config_path = format!(
        "{}/.config/cosmic/com.system76.CosmicBackground/v1/all",
        home
    );

    let content = format!(
        r#"(
    output: "all",
    source: Path("{}"),
    filter_by_theme: true,
    rotation_frequency: 7200,
    filter_method: Lanczos,
    scaling_mode: Zoom,
    sampling_method: Alphanumeric,
)"#,
        path.display()
    );

    fs::create_dir_all(
        format!(
            "{}/.config/cosmic/com.system76.CosmicBackground/v1",
            home
        )
    ).map_err(|e| e.to_string())?;

    fs::write(config_path, content).map_err(|e| e.to_string())?;

    Ok(())
}

fn get_wallpaper_path(urlbase: &str) -> Result<PathBuf, String> {
    let home = std::env::var("HOME").map_err(|e| e.to_string())?;

    let dir = format!("{}/Pictures/BingWallpaper", home);

    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let filename = filename_from_urlbase(urlbase);

    Ok(PathBuf::from(dir).join(filename))
}

fn divider<'a>() -> Element<'a, Message> {
    widget::container(widget::text(""))
        .height(1)
        .width(Length::Fill)
        .style(|theme: &cosmic::Theme| {
            let mut style = widget::container::Style::default();
            style.background = Some(
                cosmic::iced::Background::Color(
                    theme.cosmic().bg_divider().into()
                )
            );
            style
        })
        .into()
}

fn menu_item<'a>(
    core: &cosmic::Core,
    id: &'static str,
    label: &'static str,
    on_press: Message,
    hovered: Option<&'static str>,
) -> Element<'a, Message> {
    let is_hovered = hovered == Some(id);

    let icon_name = match id {
        "set" => "preferences-desktop-wallpaper-symbolic",
        "refresh" => "view-refresh-symbolic",
        _ => "image-x-generic-symbolic",
    };

    let icon = core
        .applet
        .icon_button::<Message>(icon_name)
        .on_press_maybe(None);

    widget::mouse_area(
        widget::container(
            widget::row()
                .spacing(8)
                .align_y(Alignment::Center)

                // 👇 ICON AREA (fixed width like your commands app)
                .push(
                    widget::container(icon)
                        .width(34 + 20)
                        .height(30)
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center)
                        .padding([0, 0, 0, 20])
                )

                // 👇 TEXT AREA (fills remaining space)
                .push(
                    widget::container(
                        widget::text(label)
                    )
                    .width(Length::Fill)
                    .padding([0, 10]) // ✅ padding ONLY here
                )

                .width(Length::Fill)
        )
        .width(Length::Fill)
        .height(34)
        .align_y(Alignment::Center)
        .style(move |theme: &cosmic::Theme| {
            let mut style = widget::container::Style::default();

            if is_hovered {
                let base = theme.cosmic().bg_component_color();

                style.background = Some(
                    cosmic::iced::Background::Color(base.into())
                );
            }

            style
        })
    )
    .on_enter(Message::HoverItem(id))
    .on_exit(Message::UnhoverItem)
    .on_press(on_press)
    .into()
}

impl cosmic::Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "com.github.vinesnts.bing-wallpaper-applet";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        let app = AppModel {
            core,
            wallpaper: None,
            loading: true,
            ..Default::default()
        };

        (
            app,
            Task::perform(fetch_bing(), Message::WallpaperLoaded)
                .map(cosmic::Action::App),
        )
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn view(&self) -> Element<'_, Self::Message> {
        self.core
            .applet
            .icon_button::<Message>("preferences-desktop-wallpaper-symbolic")
            .on_press(Message::TogglePopup)
            .into()
    }

    fn view_window(&self, _id: Id) -> Element<'_, Message> {

        let mut col = widget::column()
            .width(Length::Fill)
            .spacing(8);

        col = col.push(
            widget::container(widget::text(""))
                .height(8)
        );

        if self.loading {
            col = col.push(widget::text("Loading wallpaper..."));
        } else if let Some(w) = &self.wallpaper {

            col = col.push(
                widget::container(
                    widget::row()
                        .align_y(Alignment::Center)
                        .push(
                            widget::text(format!(
                                "Bing Wallpaper of the Day for {}",
                                format_date_pretty(&w.date)
                            ))
                            .size(12)
                        )
                )
                .padding([2, 20])
            );

            let img = image::Image::new(image::Handle::from_path(
                get_wallpaper_path(&w.urlbase)
                    .unwrap_or_else(|_| PathBuf::from("/tmp/bing-wallpaper.jpg"))
            ))
            .width(Length::Fill);

            col = col.push(img);

            col = col.push(
                widget::container(
                    widget::row()
                        .align_y(Alignment::Center)
                        .push(
                            widget::text(&w.title)
                                .width(Length::Fill)
                        )
                )
                .padding([2, 20])
            );


            col = col.push(
                widget::container(
                    widget::text(&w.copyright)
                        .size(10)
                )
                .padding([8, 20, 8, 20])
                .style(|theme: &cosmic::Theme| {
                    let mut style = widget::container::Style::default();
                    style.text_color = Some(
                        theme.cosmic().on_bg_color().into()
                    );
                    style
                })
            );

            col = col.push(divider());


            // Actions
            col = col.push(
                menu_item(
                    &self.core,
                    "set",
                    "Set wallpaper",
                    Message::SetWallpaper,
                    self.hovered,
                )
            );

            col = col.push(
                menu_item(
                    &self.core,
                     "refresh",
                    "Refresh",
                    Message::FetchWallpaper,
                    self.hovered,
                )
            );
        } else {
            col = col.push(
                widget::button::standard("Load wallpaper")
                .on_press(Message::FetchWallpaper)
            );
        }
        col = col.push(
            widget::container(widget::text(""))
                .height(8)
        );

        self.core.applet.popup_container(
            widget::container(col).width(Length::Fill)
        ).into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        cosmic::iced::time::every(Duration::from_secs(60))
            .map(|_| Message::Tick)
    }

    fn update(
        &mut self,
        message: Self::Message,
    ) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    let new_id = Id::unique();
                    self.popup.replace(new_id);

                    let mut popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );

                    popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(300.0)
                        .min_width(250.0)
                        .min_height(100.0);

                    get_popup(popup_settings)
                };
            }

            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }

            Message::FetchWallpaper => {
                self.loading = true;
                return Task::perform(fetch_bing(), Message::WallpaperLoaded)
                    .map(cosmic::Action::App);
            }

            Message::WallpaperLoaded(result) => {
                self.loading = false;

                if let Ok(w) = result {
                    let path = get_wallpaper_path(&w.urlbase).unwrap();

                    // spawn download in background
                    let url = w.url.clone();
                    Task::perform(async move {
                        let _ = download_image(url, path).await;
                    }, |_| Message::WallpaperSet);

                    let path = get_wallpaper_path(&w.urlbase)
                        .unwrap_or_else(|_| PathBuf::from("/tmp/bing-wallpaper.jpg"));

                    let url = w.url.clone();

                    self.wallpaper = Some(w);

                    return Task::perform(
                        async move {
                            if !path.exists() {
                                let _ = download_image(url, path.clone()).await;
                            }

                            let _ = set_wallpaper(&path);
                        },
                        |_| Message::WallpaperSet,
                    ).map(cosmic::Action::App);
                }
            }

            Message::SetWallpaper => {
                if let Some(w) = &self.wallpaper {
                    let path = get_wallpaper_path(&w.urlbase)
                        .unwrap_or_else(|_| PathBuf::from("/tmp/bing-wallpaper.jpg"));

                    let url = w.url.clone();

                    return Task::perform(
                        async move {
                            if !path.exists() {
                                let _ = download_image(url, path.clone()).await;
                            }

                            let _ = set_wallpaper(&path);
                        },
                        |_| Message::WallpaperSet,
                    )
                    .map(cosmic::Action::App);
                }

                Task::<cosmic::Action<Message>>::none();
            }

            Message::HoverItem(id) => {
                self.hovered = Some(id);
            }

            Message::UnhoverItem => {
                self.hovered = None;
            }

            Message::WallpaperSet => {}

            Message::SubscriptionChannel => {}

            Message::Tick => {
                use chrono::Local;
                let now = Local::now();

                let hour = now.hour();
                let minute = now.minute();
                let day = now.day();

                if hour == 0 && minute == 5 {
                    if self.last_run_day != Some(day) {
                        self.last_run_day = Some(day);

                        return Task::perform(fetch_bing(), Message::WallpaperLoaded)
                            .map(cosmic::Action::App);
                    }
                }
            }
        }

        Task::none()
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}