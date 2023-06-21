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
    started_title: String,
    stopped_title: String
}

impl MenuItem {
    fn actual_title(&self, status: bool) -> &String {
        match status {
            true => &self.started_title,
            false => &self.stopped_title
        }
    }
}

struct MenuItemList {
    status: MenuItem,
    action: MenuItem
}

impl MenuItemList {
    fn new() -> Self {
        MenuItemList {
            status: MenuItem {
                stopped_title: String::from("Status: Stopped"),
                started_title: String::from("Status: Started")
            },
            action: MenuItem {
                stopped_title: String::from("Start kerio-kvc service"),
                started_title: String::from("Stop kerio-kvc service")
            }
        }
    }
}


fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let mut tray_menu = Menu::new();
    let service = Service::new();
    let icon = Icon::new();
    let menu_item_list = MenuItemList::new();
    let is_active = service.is_active();

    let mut status_menu_item = tray_menu
        .add_item(MenuItemAttributes::new(
            menu_item_list
                .status
                .actual_title(is_active)
        ));

    let mut action_menu_item = tray_menu
        .add_item(MenuItemAttributes::new(
            menu_item_list
                .action
                .actual_title(is_active)
        ));

    let quit_menu_item = tray_menu
        .add_item(MenuItemAttributes::new("Quit"));

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
                            menu_item_list
                                .status
                                .actual_title(!is_active)
                        );

                        action_menu_item.set_title(
                            menu_item_list
                                .action
                                .actual_title(!is_active)
                        );

                        SystemTray::set_icon(&mut system_tray, icon.actual(!is_active));
                    }

                } else if menu_id == status_menu_item.clone().id() {
                    let is_active = service.is_active();

                    status_menu_item.set_title(
                        menu_item_list
                            .status
                            .actual_title(is_active)
                    );

                    action_menu_item.set_title(
                        menu_item_list
                            .action
                            .actual_title(is_active)
                    );

                    SystemTray::set_icon(&mut system_tray, icon.actual(is_active));
                }
            },
            _ => (),
        }
    });
}
