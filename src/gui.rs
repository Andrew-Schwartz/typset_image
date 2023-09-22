use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use iced::{Alignment, Application, Command, ContentFit, Event, keyboard, Subscription, Theme, widget};
use iced::alignment::{Horizontal, Vertical};
use iced::keyboard::{KeyCode, Modifiers};
use iced::Length::{Fill, FillPortion};
use iced::widget::{container, horizontal_rule, scrollable, svg, text, text_input};
use iced::widget::svg::Handle;
use iced::widget::text_input::Id;
use keyboard::Event::KeyPressed;

use types::*;

use crate::{col, easing, gen_svg, GuiError, row};

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

#[derive(Clone, Debug)]
pub enum Message {
    Latex(String),
    Name(String),
    Color(String),
    Compile,
    SvgGenerated(Result<PathBuf, GuiError>),
    FocusNext,
    FocusPrevious,
}

pub struct Gui {
    latex: String,
    name: Option<String>,
    color: Option<String>,
    path: Result<PathBuf, GuiError>,
    compiling: bool,
}

fn not_empty(s: &String) -> bool {
    !s.is_empty()
}

const DEFAULT_COLOR: &'static str = "blue";
const DEFAULT_FILE_NAME: &'static str = "eq.svg";

fn latex_id() -> Id {
    Id::new("latex")
}
fn color_id() -> Id {
    Id::new("color")
}
fn file_id() -> Id {
    Id::new("file")
}

impl Application for Gui {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new((): ()) -> (Self, Command<Message>) {
        (
            Self {
                latex: String::new(),
                name: None,
                color: None,
                path: Err(GuiError::NoLatex),
                compiling: false,
            },
            text_input::focus(latex_id())
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
                self.compiling = true;
                Command::perform(
                    gen_svg(
                        self.latex.clone(),
                        self.color.clone().unwrap_or_else(|| DEFAULT_COLOR.to_string()),
                        self.name.clone().unwrap_or_else(|| DEFAULT_FILE_NAME.to_string()),
                    ),
                    |res| Message::SvgGenerated(res),
                )
            }
            Message::SvgGenerated(path) => {
                self.path = path;
                self.compiling = false;
                Command::none()
            }
            Message::FocusNext => widget::focus_next(),
            Message::FocusPrevious => widget::focus_previous(),
        }
    }

    fn view(&self) -> Element<'_> {
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
                    DEFAULT_FILE_NAME,
                    self.name.as_deref().unwrap_or_default()
                ).on_input(Message::Name)
                 .on_submit(Message::Compile)
                 .id(file_id()),
            ].align_items(Alignment::Center),
            horizontal_rule(20),
        ].width(FillPortion(3));
        let row = row![
            Fill,
            input_col,
            Fill
        ];
        let content = if self.compiling {
            let spinner = Circular::new()
                .size(200.0)
                .bar_height(20.0)
                .easing(&easing::EMPHASIZED_DECELERATE)
                .cycle_duration(Duration::from_secs_f32(2.0));
            Container::new(spinner)
        } else {
            match &self.path {
                Ok(path) => {
                    // have to read the svg manually because otherwise it won't update the image
                    //  if the same path is used
                    let data = fs::read(path).unwrap();
                    let svg = svg(Handle::from_memory(data))
                        .height(Fill)
                        .content_fit(ContentFit::Contain);
                    Container::new(svg)
                }
                Err(e) => Container::new(scrollable(
                    text(e).size(40)
                )),
            }
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
        const NONE: Modifiers = Modifiers::empty();

        iced::subscription::events_with(|event, _status| match event {
            Event::Keyboard(KeyPressed { key_code: KeyCode::Tab, modifiers: NONE }) => Some(Message::FocusNext),
            Event::Keyboard(KeyPressed { key_code: KeyCode::Tab, modifiers: Modifiers::SHIFT }) => Some(Message::FocusPrevious),
            _ => None,
        })
    }
}