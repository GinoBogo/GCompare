//! Custom GDiffMap widget implementation.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{Box, DrawingArea, Fixed, Frame, GestureDrag, Overlay, glib};
use once_cell::sync::Lazy;
use std::cell::{Cell, RefCell};

#[derive(Clone, Copy, Debug, Default)]
pub struct CursorSize {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum PanelId {
    #[default]
    A,
    B,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FileTextInfo {
    pub upper_line: usize,
    pub total_lines: usize,
    pub visible_lines: usize,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TextInfo {
    pub a: FileTextInfo,
    pub b: FileTextInfo,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CanvasSize {
    pub w: f64,
    pub h: f64,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DrawInfo {
    pub canvas_size: CanvasSize,
    pub cursor_size: CursorSize,
}

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct GDiffMap {
        pub text_info: Cell<TextInfo>,
        pub draw_info: Cell<DrawInfo>,
        pub last_updated_panel: Cell<PanelId>,
        pub drag_start_y: Cell<f64>,
        pub is_dragging: Cell<bool>,
        pub diff_lines_a: RefCell<Vec<usize>>,
        pub diff_lines_b: RefCell<Vec<usize>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GDiffMap {
        const NAME: &'static str = "GDiffMap";
        type Type = super::GDiffMap;
        type ParentType = Frame;
    }

    impl ObjectImpl for GDiffMap {
        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<glib::subclass::Signal>> = Lazy::new(|| {
                vec![
                    glib::subclass::Signal::builder("scroll-to")
                        // Signal emitted when dragging the cursor
                        .param_types([f64::static_type()])
                        .build(),
                ]
            });
            &SIGNALS
        }

        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            // Configure Frame: No label
            obj.set_label(None);
            obj.set_width_request(40);
            obj.set_hexpand(false);
            obj.set_vexpand(true);

            // Create Overlay
            let overlay = Overlay::new();

            // Create DrawingArea (Background)
            let drawing_area = DrawingArea::new();
            drawing_area.set_hexpand(true);
            drawing_area.set_vexpand(true);
            overlay.set_child(Some(&drawing_area));

            // Create Fixed container (Overlay)
            let fixed = Fixed::new();
            overlay.add_overlay(&fixed);

            // Create Cursor widget
            let cursor = Box::new(gtk::Orientation::Horizontal, 0);
            cursor.add_css_class("map-cursor");
            cursor.set_cursor_from_name(Some("grab"));

            crate::libs::gtheme::init();

            // Setup Drag Gesture
            let drag = GestureDrag::new();

            // Handle drag start
            drag.connect_drag_begin(glib::clone!(@weak obj => move |gesture, start_x, start_y| {
                let imp = obj.imp();
                let cursor_size = imp.draw_info.get().cursor_size;

                // Check if click is inside cursor
                if start_x >= cursor_size.x
                    && start_x <= cursor_size.x + cursor_size.w
                    && start_y >= cursor_size.y
                    && start_y <= cursor_size.y + cursor_size.h
                {
                    imp.is_dragging.set(true);
                    imp.drag_start_y.set(cursor_size.y);
                    obj.set_cursor_from_name(Some("grabbing"));
                } else {
                    gesture.set_state(gtk::EventSequenceState::Denied);
                }
            }));
            drag.connect_drag_update(glib::clone!(@weak obj => move |_, _, offset_y| {
                let imp = obj.imp();
                if imp.is_dragging.get() {
                    obj.handle_drag_update(offset_y);
                }
            }));
            drag.connect_drag_end(glib::clone!(@weak obj => move |_, _, _| {
                let imp = obj.imp();
                if imp.is_dragging.get() {
                    imp.is_dragging.set(false);
                    obj.set_cursor_from_name(None);
                    obj.update_cursor_size();
                }
            }));
            obj.add_controller(drag);

            fixed.put(&cursor, 0.0, 0.0);
            obj.set_child(Some(&overlay));

            // Setup Drawing Function
            let obj_weak = obj.downgrade();
            drawing_area.set_draw_func(move |da, cr, w, h| {
                if let Some(obj) = obj_weak.upgrade() {
                    obj.draw_diff_lines(da, cr, w as f64, h as f64);
                }
            });

            obj.connect_notify_local(Some("width"), move |obj, _| obj.update_cursor_size());
            obj.connect_notify_local(Some("height"), move |obj, _| obj.update_cursor_size());
        }
    }

    impl WidgetImpl for GDiffMap {}
    impl FrameImpl for GDiffMap {}
}

glib::wrapper! {
    pub struct GDiffMap(ObjectSubclass<imp::GDiffMap>)
        @extends Frame, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl GDiffMap {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn update_text_info(
        &self,
        panel: PanelId,
        upper_line: usize,
        total_lines: usize,
        visible_lines: usize,
    ) {
        let imp = self.imp();
        let mut info = imp.text_info.get();

        let file_info = FileTextInfo {
            upper_line,
            total_lines,
            visible_lines,
        };

        match panel {
            PanelId::A => info.a = file_info,
            PanelId::B => info.b = file_info,
        }

        imp.text_info.set(info);
        imp.last_updated_panel.set(panel);

        self.update_cursor_size();
    }

    pub fn set_diff_lines(&self, lines_a: Vec<usize>, lines_b: Vec<usize>) {
        let imp = self.imp();
        *imp.diff_lines_a.borrow_mut() = lines_a;
        *imp.diff_lines_b.borrow_mut() = lines_b;

        if let Some(overlay) = self.child().and_downcast::<Overlay>() {
            if let Some(drawing_area) = overlay.child() {
                drawing_area.queue_draw();
            }
        }
    }

    fn draw_diff_lines(&self, da: &DrawingArea, cr: &gtk::cairo::Context, width: f64, height: f64) {
        let imp = self.imp();
        let info = imp.text_info.get();
        let max_total_lines = info.a.total_lines.max(info.b.total_lines).max(1) as f64;
        let scale_y = height / max_total_lines;

        let style_context = da.style_context();

        // Helper to get color from CSS class
        let get_color = |class_name: &str| {
            style_context.add_class(class_name);
            let color = style_context.color();
            style_context.remove_class(class_name);
            color
        };

        // Draw Background
        let bg_color = get_color("diff-map");
        cr.set_source_rgba(
            bg_color.red() as f64,
            bg_color.green() as f64,
            bg_color.blue() as f64,
            bg_color.alpha() as f64,
        );
        cr.paint().expect("Invalid cairo surface state");

        // Draw separator line
        let separator_color = get_color("diff-map-separator");
        cr.set_source_rgba(
            separator_color.red() as f64,
            separator_color.green() as f64,
            separator_color.blue() as f64,
            separator_color.alpha() as f64,
        );
        cr.set_line_width(1.0);
        let center_x = (width / 2.0).floor() + 0.5;
        cr.move_to(center_x, 0.0);
        cr.line_to(center_x, height);
        let _ = cr.stroke();

        // Draw A lines (Left to Half) - Light Red
        let color_a = get_color("diff-map-remove");
        cr.set_source_rgba(
            color_a.red() as f64,
            color_a.green() as f64,
            color_a.blue() as f64,
            color_a.alpha() as f64,
        );
        for &line in imp.diff_lines_a.borrow().iter() {
            let y = line as f64 * scale_y;
            cr.rectangle(0.0, y, width / 2.0, scale_y.max(1.0));
            let _ = cr.fill();
        }

        // Draw B lines (Half to Right) - Light Green
        let color_b = get_color("diff-map-add");
        cr.set_source_rgba(
            color_b.red() as f64,
            color_b.green() as f64,
            color_b.blue() as f64,
            color_b.alpha() as f64,
        );
        for &line in imp.diff_lines_b.borrow().iter() {
            let y = line as f64 * scale_y;
            cr.rectangle(width / 2.0, y, width / 2.0, scale_y.max(1.0));
            let _ = cr.fill();
        }
    }

    fn update_cursor_size(&self) {
        let imp = self.imp();

        if imp.is_dragging.get() {
            return;
        }

        let info = imp.text_info.get();
        let last_updated = imp.last_updated_panel.get();

        let width = self.width() as f64;
        let height = self.height() as f64;
        let mut cursor_size = CursorSize::default();

        let (upper_line, visible_lines) = match last_updated {
            PanelId::A => (info.a.upper_line, info.a.visible_lines),
            PanelId::B => (info.b.upper_line, info.b.visible_lines),
        };

        let max_total_lines = info.a.total_lines.max(info.b.total_lines).max(1);

        let gap = 5.0;
        let map_width = (width - 2.0 * gap).max(0.0);
        let map_height = (height - 2.0 * gap).max(0.0);

        if (max_total_lines <= 1) || (visible_lines >= max_total_lines) {
            cursor_size.x = gap;
            cursor_size.y = gap;
            cursor_size.w = map_width;
            cursor_size.h = map_height;
        } else {
            // Calculate vertical position ratio
            let ratio_y = upper_line as f64 / max_total_lines as f64;
            let ratio_h = visible_lines as f64 / max_total_lines as f64;

            cursor_size.x = gap;
            cursor_size.y = gap + (map_height * ratio_y);
            cursor_size.w = map_width;
            cursor_size.h = (map_height * ratio_h).max(5.0).min(map_height);
        }

        if let Some(overlay) = self.child().and_downcast::<Overlay>() {
            let mut child = overlay.first_child();
            while let Some(widget) = child {
                if let Some(fixed) = widget.downcast_ref::<Fixed>() {
                    if let Some(cursor) = fixed.first_child() {
                        fixed.move_(&cursor, cursor_size.x, cursor_size.y);
                        cursor.set_size_request(cursor_size.w as i32, cursor_size.h as i32);
                    }
                    break;
                }
                child = widget.next_sibling();
            }
        }

        let draw_info = DrawInfo {
            canvas_size: CanvasSize {
                w: width,
                h: height,
            },
            cursor_size,
        };
        imp.draw_info.set(draw_info);
    }

    fn handle_drag_update(&self, offset_y: f64) {
        let imp = self.imp();
        let start_y = imp.drag_start_y.get();
        let new_y = start_y + offset_y;

        let height = self.height() as f64;
        let gap = 5.0;
        let map_height = (height - 2.0 * gap).max(0.0);
        let cursor_h = imp.draw_info.get().cursor_size.h;

        if map_height > 0.0 {
            // Calculate valid Y range for the cursor top
            let max_y = gap + map_height - cursor_h;
            let clamped_y = new_y.max(gap).min(max_y);

            // Move cursor visually immediately to avoid vibration
            if let Some(overlay) = self.child().and_downcast::<Overlay>() {
                let mut child = overlay.first_child();
                while let Some(widget) = child {
                    if let Some(fixed) = widget.downcast_ref::<Fixed>() {
                        if let Some(cursor) = fixed.first_child() {
                            let current_x = imp.draw_info.get().cursor_size.x;
                            fixed.move_(&cursor, current_x, clamped_y);
                        }
                        break;
                    }
                    child = widget.next_sibling();
                }
            }

            // Calculate ratio (0.0 to 1.0) based on scrollable area
            let scrollable_height = map_height - cursor_h;
            if scrollable_height > 0.0 {
                let ratio = (clamped_y - gap) / scrollable_height;
                self.emit_by_name::<()>("scroll-to", &[&ratio]);
            }
        }
    }
}
