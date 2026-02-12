//! Terminal styling: colored when feature "colored" is on, plain otherwise.

use std::fmt::Display;

#[cfg(feature = "colored")]
pub fn yellow(s: impl AsRef<str>) -> impl Display {
    use colored::Colorize;
    s.as_ref().yellow()
}

#[cfg(feature = "colored")]
pub fn blue(s: impl AsRef<str>) -> impl Display {
    use colored::Colorize;
    s.as_ref().blue()
}

#[cfg(feature = "colored")]
pub fn red(s: impl AsRef<str>) -> impl Display {
    use colored::Colorize;
    s.as_ref().red()
}

#[cfg(feature = "colored")]
pub fn green(s: impl AsRef<str>) -> impl Display {
    use colored::Colorize;
    s.as_ref().green()
}

#[cfg(feature = "colored")]
pub fn purple(s: impl AsRef<str>) -> impl Display {
    use colored::Colorize;
    s.as_ref().purple()
}

#[cfg(not(feature = "colored"))]
pub fn yellow(s: impl AsRef<str>) -> impl Display {
    s.as_ref().to_string()
}

#[cfg(not(feature = "colored"))]
pub fn blue(s: impl AsRef<str>) -> impl Display {
    s.as_ref().to_string()
}

#[cfg(not(feature = "colored"))]
pub fn red(s: impl AsRef<str>) -> impl Display {
    s.as_ref().to_string()
}

#[cfg(not(feature = "colored"))]
pub fn green(s: impl AsRef<str>) -> impl Display {
    s.as_ref().to_string()
}

#[cfg(not(feature = "colored"))]
pub fn purple(s: impl AsRef<str>) -> impl Display {
    s.as_ref().to_string()
}

#[cfg(feature = "colored")]
pub fn dimmed(s: impl AsRef<str>) -> impl Display {
    use colored::Colorize;
    s.as_ref().dimmed()
}

#[cfg(feature = "colored")]
pub fn yellow_dimmed(s: impl AsRef<str>) -> impl Display {
    use colored::Colorize;
    s.as_ref().yellow().dimmed()
}

#[cfg(not(feature = "colored"))]
pub fn dimmed(s: impl AsRef<str>) -> impl Display {
    s.as_ref().to_string()
}

#[cfg(not(feature = "colored"))]
pub fn yellow_dimmed(s: impl AsRef<str>) -> impl Display {
    s.as_ref().to_string()
}
