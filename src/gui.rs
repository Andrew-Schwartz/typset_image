use std::{env, fs, io};
use std::collections::hash_map::DefaultHasher;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::Duration;

use iced::{Alignment, Application, Command, ContentFit, Element, Event, font, keyboard, Renderer, Subscription, Theme, widget};
use iced::alignment::{Horizontal, Vertical};
use iced::keyboard::{Key, key::Named};
use iced::Length::{Fill, FillPortion};
use iced::widget::{button, container, Container, horizontal_rule, image, pick_list, scrollable, svg, text, text_input};
use iced::widget::svg::Handle;
use iced::widget::text_input::Id;
use once_cell::sync::Lazy;
use rfd::{AsyncFileDialog, FileHandle};
use tempdir::TempDir;

use crate::{col, easing, GuiError, ICON_FONT, ICON_FONT_BYTES, latex, row, typst};
use crate::backends::Backend;
use crate::circular::Circular;
use crate::icons::Icon;

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
    EditEquation(String),
    Name(String),
    Color(String),
    Compile,
    SvgGenerated(Result<(), GuiError>),
    PngGenerated(Result<(), GuiError>),
    FocusNext,
    FocusPrevious,
    Format(ImageFormat),
    SetDpi(String),
    OutDir(String),
    OpenExplorer,
    PickedDir(Option<PathBuf>),
    SetBackend(Backend),
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
        Self::Errored(GuiError::NoEquation(Backend::default().stylized()))
    }
}

pub struct Gui {
    latex_eq: String,
    typst_eq: String,
    name: Option<String>,
    color: Option<String>,
    compiled_color: String,
    format: ImageFormat,
    dpi: usize,
    out_dir: PathBuf,
    state: State,
    folder_icon: Icon,
    backend: Backend,
    typst_dir: TempDir,
}

impl Gui {
    fn eq(&self) -> &str {
        match self.backend {
            Backend::LaTeX => &self.latex_eq,
            Backend::Typst => &self.typst_eq,
        }
    }

    fn eq_mut(&mut self) -> &mut String {
        match self.backend {
            Backend::LaTeX => &mut self.latex_eq,
            Backend::Typst => &mut self.typst_eq,
        }
    }

    fn equation_hash(&self) -> u64 {
        let mut hash = DefaultHasher::default();
        self.latex_eq.hash(&mut hash);
        hash.finish()
    }

    fn color(&self) -> &str {
        self.color.as_deref().unwrap_or(DEFAULT_COLOR)
    }

    fn cache_dir(&self) -> Dir {
        match self.backend {
            Backend::LaTeX => get_dir(self.equation_hash()),
            Backend::Typst => self.typst_dir.path().to_owned(),
        }
    }

    fn copy_to_dest(&self) -> io::Result<()> {
        let dir = self.cache_dir();
        let from_name = format!(
            "{}_eq.{}",
            self.compiled_color,
            self.format,
        );
        let to_name = self.name
            .as_ref()
            .map_or_else(
                || self.format.default_file_name().into(),
                |s| {
                    let p: &Path = s.as_ref();
                    p.with_extension(self.format.to_string())
                },
            );
        fs::copy(
            dir.join(from_name),
            self.out_dir.join(to_name),
        ).map(|_| ())
    }
}

fn not_empty(s: &String) -> bool {
    !s.is_empty()
}

const DEFAULT_COLOR: &str = "white";

fn eq_editor_id() -> Id {
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
        (
            Self {
                latex_eq: String::new(),
                typst_eq: String::new(),
                name: None,
                color: None,
                compiled_color: DEFAULT_COLOR.to_string(),
                format: ImageFormat::default(),
                dpi: 1000,
                out_dir: env::current_dir().unwrap(),
                state: Default::default(),
                folder_icon: Icon::Folder,
                backend: Default::default(),
                typst_dir: TempDir::new("typst_").unwrap(),
            },
            Command::batch([
                text_input::focus(eq_editor_id()),
                font::load(ICON_FONT_BYTES)
                    .map(|_| Message::FontLoaded),
            ])
        )
    }

    fn title(&self) -> String {
        "Equation Maker".into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::EditEquation(equation) => {
                *self.eq_mut() = equation;
                if self.backend == Backend::Typst {
                    self.update(Message::Compile)
                } else {
                    Command::none()
                }
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
                if self.eq().is_empty() {
                    self.state = State::Errored(GuiError::NoEquation(self.backend.stylized()));
                    return Command::none();
                }
                self.state = State::Compiling;
                let color = self.color().to_string();
                self.compiled_color = color.clone();
                match self.backend {
                    Backend::LaTeX => {
                        let hash = self.equation_hash();
                        let dir = get_dir(hash);
                        println!("dir = {dir:?}");
                        if dir.exists() {
                            println!("dir exists!");
                            let img = dir.join(format!(
                                "{color}_eq.{}",
                                self.format,
                            ));
                            // don't recompile latex for already existing svg's, do rerun dvisvgm in case
                            // dpi has changed
                            if img.exists() && self.format == ImageFormat::Svg {
                                self.update(Message::SvgGenerated(Ok(())))
                            } else {
                                Command::perform(
                                    latex::set_color(
                                        dir,
                                        color,
                                    ),
                                    move |e: Result<(), _>| Message::SvgGenerated(e),
                                )
                            }
                        } else {
                            println!("doesn't exist, performing `latex::gen_svg`");
                            Command::perform(
                                latex::gen_svg(
                                    self.latex_eq.clone(),
                                    dir,
                                    color,
                                ),
                                Message::SvgGenerated,
                            )
                        }
                    }
                    Backend::Typst => Command::perform(
                        typst::gen_svg(
                            self.typst_eq.clone(),
                            self.typst_dir.path().to_owned(),
                            color,
                        ),
                        Message::SvgGenerated,
                    ),
                }
            }
            Message::SvgGenerated(dir) => {
                match dir {
                    Ok(()) => {
                        let dir = self.cache_dir();
                        match self.format {
                            ImageFormat::Svg => {
                                self.state = State::Svg(dir);
                                self.copy_to_dest().unwrap();
                                Command::none()
                            }
                            ImageFormat::Png => Command::perform(
                                self.backend.gen_png(
                                    self.eq().to_string(),
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
            Message::PngGenerated(res) => {
                match res {
                    Ok(()) => {
                        let dir = self.cache_dir();
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
            Message::SetBackend(backend) => {
                self.backend = backend;
                self.update(Message::Compile)
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
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
            row![
                text_input(
                    self.backend.name(),
                    self.eq(),
                ).on_input(Message::EditEquation)
                 .on_submit(Message::Compile)
                 .id(eq_editor_id()),
                button(self.backend.letter())
                    .on_press(Message::SetBackend(self.backend.flip())),
            ],
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
        let content: Container<Message, Theme, Renderer> = match &self.state {
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
                let data = fs::read(dir.join(file_name)).unwrap();
                let svg = svg::<Theme>(Handle::from_memory(data))
                    .height(Fill)
                    .content_fit(ContentFit::Contain);
                container(svg)
                    .padding(8)
            }
            State::Png(dir) => {
                // have to read the png manually because otherwise it won't update the image
                //  if the same path is used
                let file_name = format!(
                    "{}_eq.png",
                    self.compiled_color,
                );
                let data = fs::read(dir.join(file_name)).unwrap();
                let png = image(image::Handle::from_memory(data))
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
        container(col![row, content])
            .align_x(Horizontal::Center)
            .align_y(Vertical::Top)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        // const NONE: Modifiers = Modifiers::empty();
        // const CMD_SHIFT: Modifiers = Modifiers::COMMAND | Modifiers::SHIFT;

        iced::event::listen_with(|event, _status| match event {
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                match (modifiers.command(), modifiers.shift(), key.as_ref()) {
                    (true, true, Key::Named(Named::Tab)) => Some(Message::FocusNext),
                    (true, _, Key::Named(Named::Tab)) => Some(Message::FocusNext),
                    (true, _, Key::Character("L")) => Some(Message::SetBackend(Backend::LaTeX)),
                    (true, _, Key::Character("T")) => Some(Message::SetBackend(Backend::Typst)),
                    _ => None,
                }
            }
            _ => None,
        })
    }
}

static CACHE_DIR: Lazy<PathBuf> = Lazy::new(|| {
    let path = dirs::data_local_dir()
        .expect("unsupported os?")
        .join("latex_image");
    std::fs::create_dir_all(&path).unwrap();
    path
});

pub fn get_dir(hash: u64) -> Dir {
    let hash_dir = format!("latex_{hash}");
    CACHE_DIR.join(hash_dir)
}
