use std::borrow::Borrow;
use std::fmt::Display;

use iced::{Element, Length};
use iced::widget::{Button, Checkbox, Column, Container, PickList, ProgressBar, Row, Rule, Scrollable, Space, Text, TextInput, Tooltip};

use crate::circular::Circular;
use crate::gui::Message;

// use crate::gui::types::*;

// versions that get the spacing easier
#[macro_export]
macro_rules! col {
    () => {
        iced::widget::Column::new()
    };
    ($($x:expr), + $(,)?) => {
        iced::widget::Column::with_children(vec![$($crate::utils::DirectionalElement::<$crate::utils::ColDir>::into_element($x)),+])
    }
}

#[macro_export]
macro_rules! row {
    () => {
        iced::widget::Row::new()
    };
    ($($x:expr),+ $(,)?) => {
        iced::widget::Row::with_children(vec![$($crate::utils::DirectionalElement::<$crate::utils::RowDir>::into_element($x)),+])
    }
}

trait Dir {
    fn space(length: Length) -> Space;
}

pub enum ColDir {}

impl Dir for ColDir {
    fn space(length: Length) -> Space {
        Space::with_height(length)
    }
}

pub enum RowDir {}

impl Dir for RowDir {
    fn space(length: Length) -> Space {
        Space::with_width(length)
    }
}

pub trait DirectionalElement<'a, Dir> {
    fn into_element(self) -> Element<'a, Message>;
}

macro_rules! impl_directional_element {
    ($(
        $ty:path/*, $(<$lt:lifetime>)?*/
    );+ $(;)?) => {
        $(
            impl<'a, Dir> DirectionalElement<'a, Dir> for $ty {
                fn into_element(self) -> Element<'a, Message, iced::Theme, iced::Renderer> {
                    Element::from(self)
                }
            }
        )+
    };
}

impl_directional_element! {
    TextInput<'a, Message>;
    Container<'a, Message>;
    Text<'a>;
    Button<'a, Message>;
    Row<'a, Message>;
    Column<'a, Message>;
    Tooltip<'a, Message>;
    Scrollable<'a, Message>;
    Checkbox<'a, Message>;
    Rule;
    ProgressBar;
    Space;
    Circular<'a>;
}

// impl<'a, T, Dir> DirectionalElement<'a, Dir> for Slider<'a, T, Message, Renderer>
//     where T: Copy + num_traits::cast::FromPrimitive + 'a,
//           f64: From<T>,
// {
//     fn into_element(self) -> Element<'a, Message> {
//         Element::from(self)
//     }
// }

impl<'a, T, L, V, Dir> DirectionalElement<'a, Dir> for PickList<'a, T, L, V, Message>
    where T: Clone + Eq + Display + 'static,
          L: Borrow<[T]>,
          V: Borrow<T>,
{
    fn into_element(self) -> Element<'a, Message> {
        Element::from(self)
    }
}

// impl<'a, T, Dir> DirectionalElement<'a, Dir> for NumberInput<'a, T>
//     where T: num_traits::Num + PartialOrd + Display + FromStr + Copy + AddAssign + SubAssign + MulAssign + DivAssign + RemAssign + 'a
// {
//     fn into_element(self) -> Element<'a> {
//         Element::from(self)
//     }
// }

impl<'a, D: Dir> DirectionalElement<'a, D> for Length {
    fn into_element(self) -> Element<'a, Message> {
        <Space as DirectionalElement<'a, D>>::into_element(D::space(self))
    }
}

impl<'a, D: Dir> DirectionalElement<'a, D> for u16 {
    fn into_element(self) -> Element<'a, Message> {
        <Space as DirectionalElement<'a, D>>::into_element(D::space(self.into()))
    }
}

pub trait SpacingExt {
    fn push_space<L: Into<Length>>(self, length: L) -> Self;
}

impl<'a> SpacingExt for Column<'a, Message> {
    fn push_space<L: Into<Length>>(self, length: L) -> Self {
        self.push(Space::with_height(length))
    }
}

impl<'a> SpacingExt for Row<'a, Message> {
    fn push_space<L: Into<Length>>(self, length: L) -> Self {
        self.push(Space::with_width(length))
    }
}