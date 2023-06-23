use std::collections::HashMap;
use std::process::ExitStatus;
use systemctl;
use tao::system_tray::SystemTray;
use tao::platform::linux::SystemTrayBuilderExtLinux;
use tao::{
    event::Event,
    event_loop::{ControlFlow, EventLoop},
    menu::{ContextMenu as Menu, MenuItemAttributes, MenuType},
    system_tray::SystemTrayBuilder,
    TrayId,
};


struct Service {
    name: String
}

impl Service  {
    fn new() -> Self  {
        Service {
            name: String::from("kerio-kvc")
        }
    }

    fn is_active(&self) -> bool {
        systemctl::is_active(&self.name).unwrap()
    }

    fn stop(&self) -> ExitStatus {
        systemctl::stop(&self.name).unwrap()
    }

    fn restart(&self) -> ExitStatus {
        systemctl::restart(&self.name).unwrap()
    }
}


struct Icon {
    started: tao::system_tray::Icon,
    stopped: tao::system_tray::Icon,
}

impl Icon {
    fn new() -> Self  {
        Icon {
            started: Icon::load_icon("kerio_started_green_32x32.png"),
            stopped: Icon::load_icon("kerio_stopped_gray_32x32.png"),
        }
    }

    fn actual(&self, status: bool) -> tao::system_tray::Icon {
        match status {
            true => self.started.clone(),
            false => self.stopped.clone()
        }
    }

    fn load_icon(file_name: &str) -> tao::system_tray::Icon {
        let path = format!("{}/resources/{}", env!("CARGO_MANIFEST_DIR"), file_name);
        let path = std::path::Path::new(&path);

        let (icon_rgba, icon_width, icon_height) = {
            let image = image::open(path)
                .expect("Failed to open icon path")
                .into_rgba8();
            let (width, height) = image.dimensions();
            let rgba = image.into_raw();
            (rgba, width, height)
        };

        tao::system_tray::Icon::from_rgba(icon_rgba, icon_width, icon_height)
            .expect("Failed to open icon")
    }
}

struct MenuItem {
    active_title: &'static str,
    inactive_title: &'static str
}

impl MenuItem {
    fn new(active_title: &'static str, inactive_title: &'static str) -> Self {
        Self {
            active_title,
            inactive_title
        }
    }

    fn get_actual_title(&self, is_active: bool) -> &str {
        match is_active {
            true => &self.active_title,
            false => &self.inactive_title
        }
    }
}


struct MenuItemCollection {
    items: HashMap<&'static str, MenuItem>
}

impl MenuItemCollection {
    fn new() -> Self {
        Self {
            items: HashMap::new()
        }
    }

    fn add_item(&mut self, key: &'static str, menu_item: MenuItem) {
        self.items.insert(key, menu_item);
    }

    fn get_actual_title(&self, key: &'static str, is_active: bool) -> &str {
        let menu_item = self.items.get(key).expect("Menu item not found");
        menu_item.get_actual_title(is_active)
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let mut tray_menu = Menu::new();
    let service = Service::new();
    let icon = Icon::new();
    let is_active = service.is_active();

    let mut menu_items = MenuItemCollection::new();
    menu_items.add_item("status", MenuItem::new("Status: Started", "Status: Stopped"));
    menu_items.add_item("action", MenuItem::new("Stop kerio-kvc service", "Start kerio-kvc service"));
    menu_items.add_item("quit", MenuItem::new("Quit", "Quit"));

    let mut status_menu_item = tray_menu
        .add_item(MenuItemAttributes::new(
            menu_items.get_actual_title("status", is_active)
        ));

    let mut action_menu_item = tray_menu
        .add_item(MenuItemAttributes::new(
            menu_items.get_actual_title("action", is_active)
        ));

    let quit_menu_item = tray_menu
        .add_item(MenuItemAttributes::new(
            menu_items.get_actual_title("quit", is_active)
        ));

    let mut system_tray = SystemTrayBuilder::new(icon.actual(is_active), Some(tray_menu))
        .with_id(TrayId::new("main-tray"))
        .with_temp_icon_dir(std::path::Path::new("/tmp/kerio-kvc-indicator"))
        .build(&event_loop)
        .unwrap();

    event_loop.run(move |event, _event_loop, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::MenuEvent {
                menu_id,
                // specify only context menu's
                origin: MenuType::ContextMenu,
                ..
            } => {
                if menu_id == quit_menu_item.clone().id() {
                    *control_flow = ControlFlow::Exit;
                } else if menu_id == action_menu_item.clone().id() {
                    let is_active = service.is_active();
                    let status;

                    if is_active {
                        status = service.stop();
                    } else {
                        status = service.restart();
                    }

                    if status.success() {
                        status_menu_item.set_title(
                            menu_items.get_actual_title("status", !is_active)
                        );

                        action_menu_item.set_title(
                            menu_items.get_actual_title("action", !is_active)
                        );

                        SystemTray::set_icon(&mut system_tray, icon.actual(!is_active));
                    }

                } else if menu_id == status_menu_item.clone().id() {
                    let is_active = service.is_active();

                    status_menu_item.set_title(
                        menu_items.get_actual_title("status", is_active)
                    );

                    action_menu_item.set_title(
                        menu_items.get_actual_title("action", is_active)
                    );

                    SystemTray::set_icon(&mut system_tray, icon.actual(is_active));
                }
            },
            _ => (),
        }
    });
}
