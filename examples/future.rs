use glib::Sender;
use gtk::prelude::{
    BoxExt, ButtonExt, EditableExt, GtkWindowExt, TextBufferExt, TextViewExt, WidgetExt,
};
use relm4::*;
use struct_tracker::Tracker;

struct AppWidgets {
    main: gtk::ApplicationWindow,
    text: gtk::TextView,
}

#[derive(Debug)]
enum AppMsg {
    Request(String),
    Repsonse(String),
}

#[struct_tracker::tracker]
struct AppModel {
    text: String,
    waiting: bool,
}

impl RelmWidgets<AppModel, (), AppMsg> for AppWidgets {
    type Root = gtk::ApplicationWindow;

    fn init_view(_model: &AppModel, _components: &(), sender: Sender<AppMsg>) -> Self {
        let main = gtk::ApplicationWindowBuilder::new()
            .default_width(300)
            .default_height(200)
            .build();
        let main_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .margin_end(5)
            .margin_top(5)
            .margin_start(5)
            .margin_bottom(5)
            .spacing(5)
            .build();

        let url = gtk::Entry::builder()
            .placeholder_text("https://example.com")
            .build();
        let submit = gtk::Button::with_label("Submit");

        let scroller = gtk::ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .build();
        let text = gtk::TextView::new();
        scroller.set_child(Some(&text));

        main_box.append(&url);
        main_box.append(&submit);
        main_box.append(&scroller);

        main.set_child(Some(&main_box));

        submit.connect_clicked(move |_| {
            let text: String = url.text().into();
            sender.send(AppMsg::Request(text)).unwrap();
        });

        AppWidgets { main, text }
    }

    fn root_widget(&self) -> gtk::ApplicationWindow {
        self.main.clone()
    }
}

impl AppUpdate<(), AppMsg> for AppModel {
    type Widgets = AppWidgets;

    fn update(&mut self, msg: AppMsg, _components: &(), sender: Sender<AppMsg>) {
        self.reset();

        match msg {
            AppMsg::Request(url) => {
                self.set_waiting(true);

                let fut = async move {
                    let mut text = "Connection error".to_string();

                    if surf::Url::parse(&url).is_ok() {
                        if let Ok(mut req) = surf::get(url).await {
                            if let Ok(resp) = req.body_string().await {
                                text = resp;
                            }
                        }
                    }
                    sender.send(AppMsg::Repsonse(text)).unwrap();
                };

                spawn_future(fut);
            }
            AppMsg::Repsonse(text) => {
                self.set_text(text);
                self.set_waiting(false);
            }
        }
    }

    fn view(&self, widgets: &mut Self::Widgets, _sender: Sender<AppMsg>) {
        if self.changed(Self::text()) {
            widgets.text.buffer().set_text(&self.text);
        }

        if self.changed(Self::waiting()) {
            widgets.main.set_sensitive(!self.waiting);
        }
    }
}

fn main() {
    gtk::init().unwrap();
    let model = AppModel {
        text: String::new(),
        waiting: false,
        tracker: 0,
    };
    let relm: RelmApp<AppWidgets, AppModel, (), AppMsg> = RelmApp::new(model);
    relm.run();
}
