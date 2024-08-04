use ratatui::style::Style;

pub trait IntoRatatuiStyle {
    fn style(&self) -> Style;
}
