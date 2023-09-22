use std::fmt::Display;

use iced::Length;
use iced::widget::{horizontal_space, Space, vertical_space};

use crate::gui::types::*;

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
        vertical_space(length)
    }
}

pub enum RowDir {}

impl Dir for RowDir {
    fn space(length: Length) -> Space {
        horizontal_space(length)
    }
}

pub trait DirectionalElement<'a, Dir> {
    fn into_element(self) -> Element<'a>;
}

macro_rules! impl_directional_element {
    ($(
        $ty:path/*, $(<$lt:lifetime>)?*/
    );+ $(;)?) => {
        $(
            impl<'a, Dir> DirectionalElement<'a, Dir> for $ty {
                fn into_element(self) -> Element<'a> {
                    Element::from(self)
                }
            }
        )+
    };
}

impl_directional_element! {
    TextInput<'a>;
    Container<'a>;
    Text<'a>;
    Button<'a>;
    Row<'a>;
    Column<'a>;
    Tooltip<'a>;
    Scrollable<'a>;
    Checkbox<'a>;
    Rule;
    ProgressBar;
    iced::widget::Space;
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

impl<'a, T, Dir> DirectionalElement<'a, Dir> for PickList<'a, T>
    where T: Clone + Eq + Display + 'static,
          [T]: ToOwned<Owned=Vec<T>>,
{
    fn into_element(self) -> Element<'a> {
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
    fn into_element(self) -> Element<'a> {
        <Space as DirectionalElement<'a, D>>::into_element(D::space(self))
    }
}

impl<'a, D: Dir> DirectionalElement<'a, D> for u16 {
    fn into_element(self) -> Element<'a> {
        <Space as DirectionalElement<'a, D>>::into_element(D::space(self.into()))
    }
}

pub trait SpacingExt {
    fn push_space<L: Into<Length>>(self, length: L) -> Self;
}

impl<'a> SpacingExt for Column<'a> {
    fn push_space<L: Into<Length>>(self, length: L) -> Self {
        self.push(vertical_space(length.into()))
    }
}

impl<'a> SpacingExt for Row<'a> {
    fn push_space<L: Into<Length>>(self, length: L) -> Self {
        self.push(horizontal_space(length.into()))
    }
}