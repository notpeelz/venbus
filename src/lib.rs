#![deny(unsafe_op_in_unsafe_fn)]

use std::sync::{atomic::AtomicBool, Arc};

use napi::threadsafe_function::ThreadsafeFunction;
use napi_derive::napi;

const PATH: &str = "/dev/vencord";

#[derive(Default)]
#[napi(js_name = "Venbus")]
pub struct JsVenbus {
    bus: Option<zbus::Connection>,
    callback_toggle_mute: Option<ThreadsafeFunction<(), (), (), false>>,
    callback_toggle_deafen: Option<ThreadsafeFunction<(), (), (), false>>,
    muted: Arc<AtomicBool>,
    deafened: Arc<AtomicBool>,
}

#[napi]
impl JsVenbus {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    #[napi(setter)]
    pub fn callback_toggle_mute(&mut self, cb: ThreadsafeFunction<(), (), (), false>) {
        if self.bus.is_some() {
            return;
        }

        self.callback_toggle_mute = Some(cb);
    }

    #[napi(setter)]
    pub fn callback_toggle_deafen(&mut self, cb: ThreadsafeFunction<(), (), (), false>) {
        if self.bus.is_some() {
            return;
        }

        self.callback_toggle_deafen = Some(cb);
    }

    #[napi]
    pub async unsafe fn set_deafened(&mut self, state: bool) {
        let old = self
            .deafened
            .swap(state, std::sync::atomic::Ordering::SeqCst);

        if old != state {
            let venbus_ref = self
                .bus
                .as_ref()
                .unwrap()
                .object_server()
                .interface::<_, Venbus>(PATH)
                .await
                .unwrap();
            let venbus = venbus_ref.get_mut().await;
            let _ = venbus.deafened_changed(venbus_ref.signal_emitter()).await;
        }
    }

    #[napi]
    pub async unsafe fn set_muted(&mut self, state: bool) {
        let old = self.muted.swap(state, std::sync::atomic::Ordering::SeqCst);

        if old != state {
            let venbus_ref = self
                .bus
                .as_ref()
                .unwrap()
                .object_server()
                .interface::<_, Venbus>(PATH)
                .await
                .unwrap();
            let venbus = venbus_ref.get_mut().await;
            let _ = venbus.muted_changed(venbus_ref.signal_emitter()).await;
        }
    }

    #[napi]
    pub async unsafe fn start(&mut self) -> napi::Result<()> {
        if self.bus.is_some() {
            return Err(napi::Error::from_reason("venbus already initialized"));
        }

        self.bus = Some(
            init_dbus(self)
                .await
                .map_err(|err| napi::Error::from_reason(err.to_string()))?,
        );

        Ok(())
    }
}

pub struct Venbus {
    callback_toggle_mute: Option<ThreadsafeFunction<(), (), (), false>>,
    callback_toggle_deafen: Option<ThreadsafeFunction<(), (), (), false>>,
    muted: Arc<AtomicBool>,
    deafened: Arc<AtomicBool>,
}

#[zbus::interface(name = "dev.vencord")]
impl Venbus {
    async fn toggle_mute(&mut self) -> zbus::fdo::Result<()> {
        if let Some(cb) = &self.callback_toggle_mute {
            cb.call_async(())
                .await
                .map_err(|err| zbus::fdo::Error::Failed(format!("js error: {err}")))
        } else {
            Ok(())
        }
    }

    async fn toggle_deafen(&mut self) -> zbus::fdo::Result<()> {
        if let Some(cb) = &self.callback_toggle_deafen {
            cb.call_async(())
                .await
                .map_err(|err| zbus::fdo::Error::Failed(format!("js error: {err}")))
        } else {
            Ok(())
        }
    }

    #[zbus(property)]
    async fn muted(&self) -> bool {
        self.muted.load(std::sync::atomic::Ordering::SeqCst)
    }

    #[zbus(property)]
    async fn deafened(&self) -> bool {
        self.deafened.load(std::sync::atomic::Ordering::SeqCst)
    }
}

async fn init_dbus(js_venbus: &mut JsVenbus) -> zbus::Result<zbus::Connection> {
    let callback_toggle_mute = js_venbus.callback_toggle_mute.take();
    let callback_toggle_deafen = js_venbus.callback_toggle_deafen.take();
    let deafened = Arc::clone(&js_venbus.deafened);
    let muted = Arc::clone(&js_venbus.muted);

    let conn = zbus::connection::Builder::session()?
        .serve_at(
            PATH,
            Venbus {
                callback_toggle_mute,
                callback_toggle_deafen,
                deafened,
                muted,
            },
        )?
        .name("dev.vencord")?
        .build()
        .await?;

    Ok(conn)
}
