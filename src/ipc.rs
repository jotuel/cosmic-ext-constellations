use std::error::Error;
use tokio::sync::mpsc;
use zbus::names::WellKnownName;
use zbus::{interface, proxy, Connection};

pub const DBUS_NAME: &str = "com.system76.Claw";
pub const DBUS_PATH: &str = "/com/system76/Claw";

pub struct IpcInterface {
    tx: mpsc::UnboundedSender<String>,
}

#[interface(name = "com.system76.Claw.Ipc")]
impl IpcInterface {
    async fn handle_callback(&self, uri: String) {
        if !uri.starts_with("com.system76.Claw://callback") {
            tracing::warn!("Received invalid OIDC callback URI: {}", uri);
            return;
        }
        tracing::info!("Received OIDC callback URI via D-Bus");
        let _ = self.tx.send(uri);
    }
}

#[proxy(
    interface = "com.system76.Claw.Ipc",
    default_service = "com.system76.Claw",
    default_path = "/com/system76/Claw"
)]
pub trait Ipc {
    fn handle_callback(&self, uri: String) -> zbus::Result<()>;
}

pub async fn start_server(tx: mpsc::UnboundedSender<String>) -> Result<Connection, Box<dyn Error>> {
    let connection = Connection::session().await?;
    let name = WellKnownName::try_from(DBUS_NAME)?;
    connection.request_name(name).await?;
    connection
        .object_server()
        .at(DBUS_PATH, IpcInterface { tx })
        .await?;
    Ok(connection)
}

pub async fn call_handle_callback(uri: String) -> Result<(), Box<dyn Error>> {
    let connection = Connection::session().await?;
    let proxy = IpcProxy::new(&connection).await?;
    proxy.handle_callback(uri).await?;
    Ok(())
}
