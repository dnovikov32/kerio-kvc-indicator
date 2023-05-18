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

const SERVICE_NAME: &str = "kerio-kvc";

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();

    let mut tray_menu = Menu::new();

    let menu_status_title = if is_service_active() { "Status: Started" } else { "Status: Stopped" };
    let mut menu_status_item = tray_menu.add_item(MenuItemAttributes::new(menu_status_title));

    let menu_action_title = if is_service_active() { "Stop kerio-kvc service" } else { "Start kerio-kvc service" };
    let mut menu_action_item = tray_menu.add_item(MenuItemAttributes::new(menu_action_title));

    let menu_quit_item = tray_menu.add_item(MenuItemAttributes::new("Quit"));

    let icon_started = load_icon("kerio_started.png");
    let icon_stopped = load_icon("kerio_stopped.png");
    let icon_actual = if is_service_active() { icon_started.clone() } else { icon_stopped.clone() };

    let mut system_tray = SystemTrayBuilder::new(icon_actual, Some(tray_menu))
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
                if menu_id == menu_quit_item.clone().id() {
                    *control_flow = ControlFlow::Exit;
                } else if menu_id == menu_action_item.clone().id() {
                    if is_service_active() {
                        systemctl::stop(SERVICE_NAME).unwrap();
                        menu_status_item.set_title("Status: Stopped");
                        menu_action_item.set_title("Start kerio-kvc service");
                        SystemTray::set_icon(&mut system_tray, icon_stopped.clone());
                    } else {
                        systemctl::restart(SERVICE_NAME).unwrap();
                        menu_status_item.set_title("Status: Started");
                        menu_action_item.set_title("Stop kerio-kvc service");
                        SystemTray::set_icon(&mut system_tray, icon_started.clone());
                    }
                } else if menu_id == menu_status_item.clone().id() {
                    if is_service_active() {
                        menu_status_item.set_title("Status: Started");
                        menu_action_item.set_title("Stop kerio-kvc service");
                        SystemTray::set_icon(&mut system_tray, icon_started.clone());
                    } else {
                        menu_status_item.set_title("Status: Stopped");
                        menu_action_item.set_title("Start kerio-kvc service");
                        SystemTray::set_icon(&mut system_tray, icon_stopped.clone());
                    }
                }
            },
            _ => (),
        }
    });
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

fn is_service_active() -> bool {
    systemctl::is_active(SERVICE_NAME).unwrap()
}