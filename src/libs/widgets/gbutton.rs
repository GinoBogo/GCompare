//! Custom GButton widget implementation with theming support.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

#![allow(dead_code)]

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{Button, glib};

/// Button theme variants for styling.
#[derive(Debug, Clone, Copy)]
pub enum ButtonTheme {
    Default,
    Primary,
    Secondary,
    Highlight,
    LightBlue,
    LightGreen,
}

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct GButton;

    #[glib::object_subclass]
    impl ObjectSubclass for GButton {
        const NAME: &'static str = "GButton";
        type Type = super::GButton;
        type ParentType = Button;
    }

    impl ObjectImpl for GButton {
        /// Initialize widget when constructed.
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.set_width_request(100);
            obj.set_height_request(32);
            obj.set_cursor_from_name(Some("pointer"));

            obj.add_css_class("gbutton");
            obj.add_css_class("gbutton-default");
        }
    }

    impl WidgetImpl for GButton {}
    impl ButtonImpl for GButton {}
}

glib::wrapper! {
    pub struct GButton(ObjectSubclass<imp::GButton>)
        @extends Button, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl GButton {
    /// Create a new GButton widget.
    ///
    /// # Arguments
    ///
    /// * `label` - Button label text.
    pub fn new(label: &str) -> Self {
        glib::Object::builder().property("label", label).build()
    }

    /// Apply a specific theme to the button.
    ///
    /// # Arguments
    ///
    /// * `theme` - ButtonTheme enum value.
    pub fn set_theme(&self, theme: ButtonTheme) {
        self.remove_css_class("gbutton-default");
        self.remove_css_class("gbutton-primary");
        self.remove_css_class("gbutton-secondary");
        self.remove_css_class("gbutton-highlight");
        self.remove_css_class("gbutton-lightblue");
        self.remove_css_class("gbutton-lightgreen");

        match theme {
            ButtonTheme::Default => self.add_css_class("gbutton-default"),
            ButtonTheme::Primary => self.add_css_class("gbutton-primary"),
            ButtonTheme::Secondary => self.add_css_class("gbutton-secondary"),
            ButtonTheme::Highlight => self.add_css_class("gbutton-highlight"),
            ButtonTheme::LightBlue => self.add_css_class("gbutton-lightblue"),
            ButtonTheme::LightGreen => self.add_css_class("gbutton-lightgreen"),
        }
    }
}
