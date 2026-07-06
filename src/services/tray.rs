use stray::message::NotifierItemMessage;
use stray::tokio;
use stray::StatusNotifierWatcher;

pub fn spawn_tray_listener() {
    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new()
            .expect("Failed to create Tokio runtime for tray listener");
        rt.block_on(async {
            let (_cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(32);
            let watcher = match StatusNotifierWatcher::new(cmd_rx).await {
                Ok(w) => w,
                Err(e) => {
                    eprintln!("[tray] Failed to create StatusNotifierWatcher: {e:?}");
                    return;
                }
            };

            let mut host = match watcher.create_notifier_host("shellous-tray").await {
                Ok(h) => h,
                Err(e) => {
                    eprintln!("[tray] Failed to create NotifierHost: {e:?}");
                    return;
                }
            };

            loop {
                match host.recv().await {
                    Ok(msg) => match msg {
                        NotifierItemMessage::Update {
                            address,
                            item,
                            menu: _,
                        } => {
                            println!("[tray] Item added/updated:");
                            println!("  address: {address}");
                            println!("  id: {}", item.id);
                            if let Some(title) = &item.title {
                                println!("  title: {title}");
                            }
                            println!("  category: {:?}", item.category);
                            println!("  status: {:?}", item.status);
                            if let Some(icon) = &item.icon_name {
                                println!("  icon_name: {icon}");
                            }
                            if let Some(menu) = &item.menu {
                                println!("  menu path: {menu}");
                            }
                            println!();
                        }
                        NotifierItemMessage::Remove { address } => {
                            println!("[tray] Item removed: {address}");
                        }
                    },
                    Err(e) => {
                        eprintln!("[tray] Error receiving message: {e}");
                    }
                }
            }
        });
    });
}
