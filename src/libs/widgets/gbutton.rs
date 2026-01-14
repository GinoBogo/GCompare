//! Custom GButton widget implementation with theming support.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

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
    Action1,
    Action2,
    Action3,
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
    ///
    /// # Returns
    ///
    /// New GButton instance
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
        self.remove_css_class("gbutton-action_1");
        self.remove_css_class("gbutton-action_2");
        self.remove_css_class("gbutton-action_3");

        match theme {
            ButtonTheme::Default => self.add_css_class("gbutton-default"),
            ButtonTheme::Primary => self.add_css_class("gbutton-primary"),
            ButtonTheme::Secondary => self.add_css_class("gbutton-secondary"),
            ButtonTheme::Highlight => self.add_css_class("gbutton-highlight"),
            ButtonTheme::Action1 => self.add_css_class("gbutton-action_1"),
            ButtonTheme::Action2 => self.add_css_class("gbutton-action_2"),
            ButtonTheme::Action3 => self.add_css_class("gbutton-action_3"),
        }
    }

    /// Set custom background and foreground colors.
    ///
    /// # Arguments
    ///
    /// * `bg_color` - Background color in hex format (e.g., "#ffcccc").
    /// * `fg_color` - Foreground color in hex format (e.g., "#000000").
    pub fn set_custom_colors(&self, bg_color: &str, fg_color: &str) {
        // Remove existing theme classes
        self.remove_css_class("gbutton-default");
        self.remove_css_class("gbutton-primary");
        self.remove_css_class("gbutton-secondary");
        self.remove_css_class("gbutton-highlight");
        self.remove_css_class("gbutton-action_1");
        self.remove_css_class("gbutton-action_2");
        self.remove_css_class("gbutton-action_3");

        // Add a custom class for targeting
        self.add_css_class("gbutton-custom");

        // Create CSS provider for custom colors
        let css_provider = gtk::CssProvider::new();
        let css = format!(
            ".gbutton-custom {{ background: {}; color: {}; border: 1px solid #cccccc; }}",
            bg_color, fg_color
        );

        css_provider.load_from_data(&css);

        // Apply the style context
        let style_context = self.style_context();
        style_context.add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
    }
}
