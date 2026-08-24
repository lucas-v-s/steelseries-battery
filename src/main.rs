#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod battery;

use std::thread;
use std::time::Duration;

use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{TrayIcon, TrayIconBuilder};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::window::WindowId;

const POLL_INTERVAL: Duration = Duration::from_secs(60);

enum AppEvent {
    Battery(battery::Snapshot),
    Menu(MenuEvent),
}

struct App {
    tray_icon: Option<TrayIcon>,
    refresh_id: MenuId,
    quit_id: MenuId,
    snapshot: battery::Snapshot,
    proxy: EventLoopProxy<AppEvent>,
}

impl App {
    fn spawn_poll(&self) {
        let proxy = self.proxy.clone();
        thread::spawn(move || {
            let snapshot = battery::poll();
            let _ = proxy.send_event(AppEvent::Battery(snapshot));
        });
    }
}

impl ApplicationHandler<AppEvent> for App {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        if self.tray_icon.is_some() {
            return;
        }

        let menu = Menu::new();
        let refresh = MenuItem::with_id(self.refresh_id.clone(), "Refresh now", true, None);
        let quit = MenuItem::with_id(self.quit_id.clone(), "Quit", true, None);
        menu.append(&refresh).expect("menu item");
        menu.append(&PredefinedMenuItem::separator())
            .expect("menu separator");
        menu.append(&quit).expect("menu item");

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip(battery::tooltip_text(&self.snapshot))
            .with_icon(battery::make_icon(&self.snapshot))
            .build()
            .expect("failed to create tray icon");
        self.tray_icon = Some(tray);
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        _event: WindowEvent,
    ) {
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: AppEvent) {
        match event {
            AppEvent::Battery(snapshot) => {
                self.snapshot = snapshot;
                if let Some(tray) = &self.tray_icon {
                    let _ = tray.set_tooltip(Some(battery::tooltip_text(&self.snapshot)));
                    let _ = tray.set_icon(Some(battery::make_icon(&self.snapshot)));
                }
            }
            AppEvent::Menu(menu_event) => {
                if menu_event.id == self.quit_id {
                    event_loop.exit();
                } else if menu_event.id == self.refresh_id {
                    self.spawn_poll();
                }
            }
        }
    }

}

fn main() {
    let event_loop = EventLoop::<AppEvent>::with_user_event()
        .build()
        .expect("failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Wait);

    let proxy = event_loop.create_proxy();

    let menu_proxy = proxy.clone();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = menu_proxy.send_event(AppEvent::Menu(event));
    }));

    let poll_proxy = proxy.clone();
    thread::spawn(move || loop {
        let snapshot = battery::poll();
        let _ = poll_proxy.send_event(AppEvent::Battery(snapshot));
        thread::sleep(POLL_INTERVAL);
    });

    let mut app = App {
        tray_icon: None,
        refresh_id: MenuId::new("refresh"),
        quit_id: MenuId::new("quit"),
        snapshot: battery::Snapshot::default(),
        proxy,
    };

    event_loop.run_app(&mut app).expect("event loop error");
}
