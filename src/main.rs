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
use tao::menu::{CustomMenuItem, MenuId};


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
    current_menu_item: CustomMenuItem,
    menu_name: &'static str,
    active_title: &'static str,
    inactive_title: &'static str
}

impl MenuItem {
    fn new(
        current_menu_item: CustomMenuItem,
        menu_name: &'static str,
        active_title: &'static str,
        inactive_title: &'static str
    ) -> Self {
        Self {
            current_menu_item,
            menu_name,
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
    items: HashMap<MenuId, MenuItem>
}

impl MenuItemCollection {
    fn new() -> Self {
        Self {
            items: HashMap::new()
        }
    }

    fn add_item(&mut self, menu_item: MenuItem) {
        let menu_id = menu_item.current_menu_item.clone().id();
        self.items.insert(menu_id, menu_item);
    }

    fn get_name(&self, menu_id: MenuId) -> &str {
        self.items
            .get(&menu_id)
            .expect("Menu item not found")
            .menu_name
    }

    fn switch_titles(&mut self, is_active: bool) {
        for (_, menu_item) in self.items.iter_mut() {
            let title = menu_item.get_actual_title(is_active).to_string();
            menu_item.current_menu_item.set_title(&title);
        }
    }

}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let mut tray_menu = Menu::new();
    let service = Service::new();
    let icon = Icon::new();
    let is_active = service.is_active();

    let mut status_menu_item = tray_menu.add_item(MenuItemAttributes::new("Status"));
    let mut action_menu_item = tray_menu.add_item(MenuItemAttributes::new("Action"));
    let mut quit_menu_item = tray_menu.add_item(MenuItemAttributes::new("Quit"));

    let mut menu_items = MenuItemCollection::new();

    menu_items.add_item(MenuItem::new(status_menu_item, "status", "Status: Started", "Status: Stopped"));
    menu_items.add_item(MenuItem::new(action_menu_item, "action", "Stop kerio-kvc service", "Start kerio-kvc service"));
    menu_items.add_item(MenuItem::new(quit_menu_item, "quit", "Quit", "Quit"));

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
                origin: MenuType::ContextMenu,
                ..
            } => {
                let menu = menu_items.get_name(menu_id);
                dbg!(menu);

                match menu {
                    "quit" => *control_flow = ControlFlow::Exit,
                    "action" => {
                        let is_active = service.is_active();
                        let status;

                        if is_active {
                            status = service.stop();
                        } else {
                            status = service.restart();
                        }

                        if status.success() {
                            menu_items.switch_titles(!is_active);
                            SystemTray::set_icon(&mut system_tray, icon.actual(!is_active));
                        }
                    },
                    "status" => {
                        let is_active = service.is_active();

                        menu_items.switch_titles(is_active);
                        SystemTray::set_icon(&mut system_tray, icon.actual(is_active));
                    }
                    _ => ()
                }

            },
            _ => (),
        }
    });
}
