use crate::engine::orchestrator::AppCommand;
use crate::update::UpdateStatus;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tokio::sync::mpsc;

pub fn build_tray(
    app: &mut tauri::App,
    cmd_tx: mpsc::Sender<AppCommand>,
    update_status: UpdateStatus,
) -> tauri::Result<()> {
    let app_handle = app.handle().clone();

    let quit_i = MenuItem::with_id(&app_handle, "quit", "Quit", true, None::<&str>)?;
    let settings_i =
        MenuItem::with_id(&app_handle, "settings", "Settings", true, None::<&str>)?;
    let menu = Menu::with_items(&app_handle, &[&settings_i, &quit_i])?;

    let menu_for_update = menu.clone();
    let update_status_for_check = update_status.clone();
    let app_handle_for_update = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        crate::update::spawn_update_check(
            app_handle_for_update,
            update_status_for_check,
            menu_for_update,
        )
        .await;
    });

    let update_status_for_menu = update_status;
    let mut tray_builder = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(false);

    if let Some(icon) = app_handle.default_window_icon() {
        tray_builder = tray_builder.icon(icon.clone());
    }

    tray_builder
        .on_menu_event(move |app, event| {
            if event.id == quit_i.id() {
                let _ = cmd_tx.blocking_send(AppCommand::Quit);
            } else if event.id == settings_i.id() {
                let _ = cmd_tx.blocking_send(AppCommand::OpenSettings);
            } else if event.id == "update" {
                if let Ok(lock) = update_status_for_menu.0.lock() {
                    if let Some(ref release) = *lock {
                        use tauri_plugin_opener::OpenerExt;
                        let _ = app.opener().open_url(&release.html_url, None::<&str>);
                    }
                }
            }
        })
        .on_tray_icon_event(|_tray, _event: TrayIconEvent| {})
        .build(app)?;

    Ok(())
}
