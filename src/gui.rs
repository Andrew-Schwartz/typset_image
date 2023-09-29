use std::{env, fs, io};
use std::collections::hash_map::DefaultHasher;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::time::Duration;

use iced::{Alignment, Application, Command, ContentFit, Event, font, keyboard, Subscription, Theme, widget};
use iced::alignment::{Horizontal, Vertical};
use iced::keyboard::{KeyCode, Modifiers};
use iced::Length::{Fill, FillPortion};
use iced::widget::{button, container, horizontal_rule, image, pick_list, scrollable, svg, text, text_input};
use iced::widget::svg::Handle;
use iced::widget::text_input::Id;
use keyboard::Event::KeyPressed;
use rfd::{AsyncFileDialog, FileHandle};

use types::*;

use crate::{col, easing, GuiError, ICON_FONT, ICON_FONT_BYTES, row};
use crate::icons::Icon;
use crate::latex::{gen_png, gen_svg, get_dir, set_color};

#[allow(dead_code)]
pub mod types {
    use super::Message;

    type Renderer = iced::Renderer<iced::Theme>;

    pub type Element<'a> = iced::Element<'a, Message, Renderer>;
    pub type Container<'a> = iced::widget::Container<'a, Message, Renderer>;
    pub type Text<'a> = iced::widget::Text<'a, Renderer>;
    pub type Row<'a> = iced::widget::Row<'a, Message, Renderer>;
    pub type Column<'a> = iced::widget::Column<'a, Message, Renderer>;
    pub type Button<'a> = iced::widget::Button<'a, Message, Renderer>;
    pub type Tooltip<'a> = iced::widget::Tooltip<'a, Message, Renderer>;
    pub type Scrollable<'a> = iced::widget::Scrollable<'a, Message, Renderer>;
    pub type TextInput<'a> = iced::widget::TextInput<'a, Message, Renderer>;
    pub type Checkbox<'a> = iced::widget::Checkbox<'a, Message, Renderer>;
    pub type PickList<'a, T> = iced::widget::PickList<'a, T, Message, Renderer>;
    pub type Slider<'a, T> = iced::widget::Slider<'a, T, Message, Renderer>;
    pub type Rule = iced::widget::Rule<Renderer>;
    pub type ProgressBar = iced::widget::ProgressBar<Renderer>;
    pub type Circular<'a> = crate::circular::Circular<'a, iced::Theme>;
    // pub type NumberInput<'a, T> = iced_aw::native::number_input::NumberInput<'a, T, Message, Renderer>;
}

#[derive(Default, Debug, PartialEq, Eq, Copy, Clone)]
pub enum ImageFormat {
    #[default]
    Svg,
    Png,
}

impl ImageFormat {
    pub const ALL: [Self; 2] = [
        Self::Svg,
        Self::Png,
    ];

    pub const fn default_file_name(self) -> &'static str {
        match self {
            Self::Svg => "eq.svg",
            Self::Png => "eq.png",
        }
    }
}

impl Display for ImageFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Svg => "svg",
            Self::Png => "png",
        })
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    FontLoaded,
    Latex(String),
    Name(String),
    Color(String),
    Compile,
    SvgGenerated(Result<Dir, GuiError>),
    PngGenerated(Result<Dir, GuiError>),
    FocusNext,
    FocusPrevious,
    Format(ImageFormat),
    SetDpi(String),
    OutDir(String),
    OpenExplorer,
    PickedDir(Option<PathBuf>),
}

pub type Dir = PathBuf;

#[derive(Debug, Clone)]
pub enum State {
    Compiling,
    Svg(Dir),
    Png(Dir),
    Errored(GuiError),
}

impl Default for State {
    fn default() -> Self {
        Self::Errored(GuiError::NoLatex)
    }
}

pub struct Gui {
    latex: String,
    name: Option<String>,
    color: Option<String>,
    compiled_color: String,
    format: ImageFormat,
    dpi: usize,
    out_dir: PathBuf,
    state: State,
    folder_icon: Icon,
}

impl Gui {
    fn latex_hash(&self) -> u64 {
        let mut hash = DefaultHasher::default();
        self.latex.hash(&mut hash);
        hash.finish()
    }

    fn color(&self) -> &str {
        self.color.as_deref().unwrap_or(DEFAULT_COLOR)
    }

    fn copy_to_dest(&self) -> io::Result<()> {
        let dir = get_dir(self.latex_hash());
        fs::copy(
            dir.join(format!(
                "{}_eq.{}",
                self.compiled_color,
                self.format,
            )),
            self.out_dir.join(self.name.as_deref().unwrap_or(self.format.default_file_name())),
        ).map(|_| ())
    }
}

fn not_empty(s: &String) -> bool {
    !s.is_empty()
}

const DEFAULT_COLOR: &str = "white";

fn latex_id() -> Id {
    Id::new("latex")
}

fn color_id() -> Id {
    Id::new("color")
}

fn file_id() -> Id {
    Id::new("file")
}

fn out_dir_id() -> Id {
    Id::new("out_dir")
}

impl Application for Gui {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new((): ()) -> (Self, Command<Message>) {
        let out_dir = env::current_dir().unwrap();
        (
            Self {
                latex: String::new(),
                name: None,
                color: None,
                compiled_color: DEFAULT_COLOR.to_string(),
                format: ImageFormat::default(),
                dpi: 600,
                out_dir,
                state: Default::default(),
                folder_icon: Icon::Folder,
            },
            Command::batch([
                text_input::focus(latex_id()),
                font::load(ICON_FONT_BYTES)
                    .map(|_| Message::FontLoaded)
            ])
        )
    }

    fn title(&self) -> String {
        "Equation Maker".into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::Latex(latex) => {
                self.latex = latex;
                Command::none()
            }
            Message::Name(name) => {
                self.name = Some(name).filter(not_empty);
                Command::none()
            }
            Message::Color(color) => {
                self.color = Some(color).filter(not_empty);
                Command::none()
            }
            Message::Compile => {
                if self.latex.is_empty() {
                    self.state = State::Errored(GuiError::NoLatex);
                    return Command::none();
                }
                self.state = State::Compiling;
                let hash = self.latex_hash();
                let color = self.color().to_string();
                self.compiled_color = color.clone();
                let dir = get_dir(hash);
                if dir.exists() {
                    let img = dir.join(format!(
                        "{color}_eq.{}",
                        self.format,
                    ));
                    // don't recompile latex for already existing svg's, do rerun dvisvgm in case
                    // dpi has changed
                    if img.exists() && self.format == ImageFormat::Svg {
                        self.update(Message::SvgGenerated(Ok(dir)))
                    } else {
                        Command::perform(
                            set_color(hash, color),
                            move |e: Result<(), _>| Message::SvgGenerated(e.map(|_| dir)),
                        )
                    }
                } else {
                    Command::perform(
                        gen_svg(
                            self.latex.clone(),
                            hash,
                            color,
                        ),
                        Message::SvgGenerated,
                    )
                }
            }
            Message::SvgGenerated(dir) => {
                match dir {
                    Ok(dir) => {
                        match self.format {
                            ImageFormat::Svg => {
                                self.state = State::Svg(dir);
                                self.copy_to_dest().unwrap();
                                Command::none()
                            }
                            ImageFormat::Png => Command::perform(
                                gen_png(
                                    dir,
                                    self.color().to_string(),
                                    self.dpi,
                                ),
                                Message::PngGenerated,
                            )
                        }
                    }
                    Err(e) => {
                        self.state = State::Errored(e);
                        Command::none()
                    }
                }
            }
            Message::PngGenerated(dir) => {
                match dir {
                    Ok(dir) => {
                        self.state = State::Png(dir);
                        self.copy_to_dest().unwrap();
                    }
                    Err(e) => self.state = State::Errored(e),
                }
                Command::none()
            }
            Message::FocusNext => widget::focus_next(),
            Message::FocusPrevious => widget::focus_previous(),
            Message::Format(f) => {
                self.format = f;
                self.update(Message::Compile)
            }
            Message::SetDpi(dpi) => {
                if dpi.is_empty() {
                    self.dpi = 0;
                } else if let Ok(dpi) = dpi.parse() {
                    self.dpi = dpi;
                }
                self.update(Message::Compile)
            }
            Message::OutDir(dir) => {
                // println!("dir = {:?}", dir);
                self.out_dir = dir.into();
                // don't copy the file eagerly, wait for user to request re-compile cuz otherwise it
                //  will try to copy to each non-existent directory as they type the full thing in
                //  and will successfully copy to each subdirectory which is no good
                Command::none()
            }
            Message::OpenExplorer => {
                self.folder_icon = Icon::Folder2Open;
                Command::perform(
                    AsyncFileDialog::new().pick_folder(),
                    |fh: Option<FileHandle>| Message::PickedDir(fh.map(|fh| fh.path().to_path_buf())),
                )
            }
            Message::PickedDir(dir) => {
                self.folder_icon = Icon::Folder2;
                if let Some(dir) = dir {
                    self.out_dir = dir;
                }
                Command::none()
            }
            Message::FontLoaded => {
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_> {
        let png_density = if self.format == ImageFormat::Png {
            row![
                6,
                text("dpi: "),
                text_input(
                    "dpi",
                    &self.dpi.to_string()
                ).width(100.0)
                 .on_input(Message::SetDpi),
            ]
        } else {
            row!()
        };
        let input_col = col![
            text_input(
                "latex",
                &self.latex,
            ).on_input(Message::Latex)
             .on_submit(Message::Compile)
             .id(latex_id()),
            6,
            row![
                text("Color: "),
                text_input(
                    DEFAULT_COLOR,
                    self.color.as_deref().unwrap_or_default(),
                ).on_input(Message::Color)
                 .on_submit(Message::Compile)
                 .id(color_id()),
                Fill,
                text("File: "),
                text_input(
                    self.format.default_file_name(),
                    self.name.as_deref().unwrap_or_default()
                ).on_input(Message::Name)
                 .on_submit(Message::Compile)
                 .id(file_id()),
            ].align_items(Alignment::Center),
            6,
            row![
                text("Format: "),
                pick_list(
                    &ImageFormat::ALL[..],
                    Some(self.format),
                    Message::Format,
                ),
                png_density,
                Fill,
                text("Directory: "),
                text_input(
                    ".",
                    &self.out_dir.to_string_lossy()
                ).on_input(Message::OutDir)
                 .on_submit(Message::Compile)
                 .id(out_dir_id()),
                button(
                    text(Icon::Folder2)
                        .font(ICON_FONT)
                ).on_press(Message::OpenExplorer),
            ].align_items(Alignment::Center),
            horizontal_rule(20),
        ].width(FillPortion(3));
        let row = row![
            Fill,
            input_col,
            Fill
        ];
        let content = match &self.state {
            State::Compiling => {
                let spinner = Circular::new()
                    .size(200.0)
                    .bar_height(20.0)
                    .easing(&easing::EMPHASIZED_DECELERATE)
                    .cycle_duration(Duration::from_secs_f32(2.0));
                container(spinner)
            }
            State::Svg(dir) => {
                // have to read the svg manually because otherwise it won't update the image
                //  if the same path is used
                // println!("dir = {:?}", dir);
                let file_name = format!(
                    "{}_eq.svg",
                    self.compiled_color,
                );
                // println!("file_name = {:?}", file_name);
                let data = fs::read(dir.join(file_name)).unwrap();
                let svg = svg(Handle::from_memory(data))
                    .height(Fill)
                    .content_fit(ContentFit::Contain);
                container(svg)
                    .padding(8)
            }
            State::Png(dir) => {
                let file_name = format!(
                    "{}_eq.png",
                    self.compiled_color,
                );
                // let data = fs::read(dir.join(file_name)).unwrap();
                let png = image(dir.join(file_name))
                    .height(Fill)
                    .content_fit(ContentFit::Contain);
                container(png)
                    .padding(8)
            }
            State::Errored(e) => container(scrollable(
                text(e).size(40)
            )),
        }.align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .height(Fill)
            .width(Fill);
        // let content = if self.compiling {
        //     let spinner = Circular::new()
        //         .size(200.0)
        //         .bar_height(20.0)
        //         .easing(&easing::EMPHASIZED_DECELERATE)
        //         .cycle_duration(Duration::from_secs_f32(2.0));
        //     Container::new(spinner)
        // } else {
        //     match &self.dir {
        //         Ok(path) => {
        //             // have to read the svg manually because otherwise it won't update the image
        //             //  if the same path is used
        //             let data = fs::read(path).unwrap();
        //             let svg = svg(Handle::from_memory(data))
        //                 .height(Fill)
        //                 .content_fit(ContentFit::Contain);
        //             Container::new(svg)
        //                 .padding(8)
        //         }
        //         Err(e) => Container::new(scrollable(
        //             text(e).size(40)
        //         )),
        //     }
        // }.align_x(Horizontal::Center)
        //     .align_y(Vertical::Center)
        //     .height(Fill)
        //     .width(Fill);
        container(col![row, content])
            .align_x(Horizontal::Center)
            .align_y(Vertical::Top)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        const NONE: Modifiers = Modifiers::empty();

        iced::subscription::events_with(|event, _status| match event {
            Event::Keyboard(KeyPressed { key_code: KeyCode::Tab, modifiers: NONE }) => Some(Message::FocusNext),
            Event::Keyboard(KeyPressed { key_code: KeyCode::Tab, modifiers: Modifiers::SHIFT }) => Some(Message::FocusPrevious),
            _ => None,
        })
    }
}