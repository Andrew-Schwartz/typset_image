//! Selected boostrap icons. Machine generated code. Do not change!

/// Icons
#[derive(Copy, Clone, Debug, Hash)]
#[allow(dead_code)]
pub enum Icon {
    /// folder
    Folder,
    /// folder2
    Folder2,
    /// folder2-open
    Folder2Open,
}

/// Converts an icon into a char.
#[must_use]
#[allow(clippy::too_many_lines)]
pub const fn icon_to_char(icon: Icon) -> char {
    match icon {
        Icon::Folder => '\u{61}',
        Icon::Folder2 => '\u{62}',
        Icon::Folder2Open => '\u{63}',
    }
}

impl std::fmt::Display for Icon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;
        f.write_char(icon_to_char(*self))
    }
}
