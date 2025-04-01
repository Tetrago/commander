use adw::prelude::ButtonExt;
use gtk::{gio, glib};

mod imp {
    use super::*;
    use adw::subclass::prelude::*;
    use glib::subclass::InitializingObject;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(file = "ui/window.blp")]
    pub struct Window {
        #[template_child]
        pub overlay_split_view: TemplateChild<adw::OverlaySplitView>,

        #[template_child]
        pub content_stack: TemplateChild<adw::ViewStack>,

        #[template_child]
        pub content_title: TemplateChild<adw::WindowTitle>,

        #[template_child]
        pub toggle_sidebar_button: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "CommanderWindow";
        type Type = super::Window;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[gtk::template_callbacks]
    impl Window {
        #[template_callback]
        fn on_audio_activated(stack: &adw::ViewStack) {
            stack.set_visible_child_name("audio");
        }

        #[template_callback]
        fn on_display_activated(stack: &adw::ViewStack) {
            stack.set_visible_child_name("display");
        }

        #[template_callback]
        fn toggle_sidebar(view: &adw::OverlaySplitView) {
            view.set_show_sidebar(!view.shows_sidebar());
        }
    }

    impl ObjectImpl for Window {
        fn constructed(&self) {
            self.parent_constructed();

            self.content_stack
                .connect_visible_child_notify(glib::clone!(
                    #[weak(rename_to = title)]
                    self.content_title,
                    move |stack| {
                        stack
                            .visible_child()
                            .and_then(|child| stack.page(&child).title())
                            .map(|page_title| title.set_title(&page_title));
                    }
                ));

            self.overlay_split_view
                .connect_show_sidebar_notify(glib::clone!(
                    #[weak(rename_to = button)]
                    self.toggle_sidebar_button,
                    move |split_view| {
                        button.set_icon_name(if split_view.shows_sidebar() {
                            "sidebar-hide-symbolic"
                        } else {
                            "sidebar-show-symbolic"
                        });
                    }
                ));

            self.overlay_split_view.set_show_sidebar(true);
            self.content_stack.set_visible_child_name("audio");
            self.toggle_sidebar_button
                .set_icon_name("sidebar-hide-symbolic");
        }
    }

    impl WidgetImpl for Window {}
    impl WindowImpl for Window {}
    impl ApplicationWindowImpl for Window {}
    impl AdwApplicationWindowImpl for Window {}
}

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new(app: &adw::Application) -> Self {
        glib::Object::builder().property("application", app).build()
    }
}
