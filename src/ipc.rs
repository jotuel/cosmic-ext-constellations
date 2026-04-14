use std::error::Error;
use tokio::sync::mpsc;
use zbus::names::WellKnownName;
use zbus::{interface, proxy, Connection};

pub const DBUS_NAME: &str = "fi.joonastuomi.CosmicExtConstellations";
pub const DBUS_PATH: &str = "/fi/joonastuomi/CosmicExtConstellations";

pub struct IpcInterface {
    tx: mpsc::UnboundedSender<String>,
}

#[interface(name = "fi.joonastuomi.CosmicExtConstellations.Ipc")]
impl IpcInterface {
    async fn handle_callback(&self, uri: String) {
        if !uri.starts_with("fi.joonastuomi.CosmicExtConstellations://callback") {
            tracing::warn!("Received invalid OIDC callback URI");
            return;
        }
        tracing::info!("Received OIDC callback URI via D-Bus");
        let _ = self.tx.send(uri);
    }
}

#[proxy(
    interface = "fi.joonastuomi.CosmicExtConstellations.Ipc",
    default_service = "fi.joonastuomi.CosmicExtConstellations",
    default_path = "/fi/joonastuomi/CosmicExtConstellations"
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

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;
    use tokio::sync::mpsc;

    #[tokio::test]
    #[serial]
    async fn test_start_server_dbus_error() {
        // Save the original DBUS_SESSION_BUS_ADDRESS
        let original_dbus_address = env::var("DBUS_SESSION_BUS_ADDRESS").ok();

        // Mock a DBus error by setting an invalid session bus address
        env::set_var("DBUS_SESSION_BUS_ADDRESS", "unix:path=/nonexistent");

        let (tx, _rx) = mpsc::unbounded_channel();
        let result = start_server(tx).await;

        // Restore the original address to not affect other tests
        if let Some(addr) = original_dbus_address {
            env::set_var("DBUS_SESSION_BUS_ADDRESS", addr);
        } else {
            env::remove_var("DBUS_SESSION_BUS_ADDRESS");
        }

        // The function should return an error since the session bus is unreachable
        assert!(
            result.is_err(),
            "Expected an error when DBUS_SESSION_BUS_ADDRESS is invalid"
        );
    }
}
