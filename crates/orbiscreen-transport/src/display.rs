use tokio::sync::{mpsc, oneshot};

use super::{H264Packet, IncomingInput};

#[derive(Debug, Clone)]
pub struct DisplayInfo {
    pub id: String,
    pub name: String,
    pub connector: String,
    pub width: u32,
    pub height: u32,
    pub encoder: String,
}

#[allow(missing_debug_implementations)]
pub struct AttachedDisplay {
    pub info: DisplayInfo,
    pub video: tokio::sync::broadcast::Receiver<H264Packet>,
}

#[allow(missing_debug_implementations)]
pub enum DisplayCommand {
    Acquire {
        name: String,
        key: Option<String>,
        width: u32,
        height: u32,

        bitrate_kbps: Option<u32>,
        reply: oneshot::Sender<Result<DisplayInfo, String>>,
    },
    Release {
        id: String,
    },
    Attach {
        id: Option<String>,
        key: Option<String>,
        reply: oneshot::Sender<Result<AttachedDisplay, String>>,
    },
    Detach {
        id: String,
    },
    Idr {
        id: String,
    },
    Resize {
        id: String,
        width: u32,
        height: u32,
        refresh_hz: Option<u32>,
        reply: oneshot::Sender<Result<DisplayInfo, String>>,
    },
    Lookup {
        id: Option<String>,
        reply: oneshot::Sender<Option<DisplayInfo>>,
    },
    Input {
        id: Option<String>,
        event: IncomingInput,
    },
    SetDefaults {
        width: u32,
        height: u32,
        refresh_hz: u32,
    },
    Shutdown {
        reply: oneshot::Sender<()>,
    },
}

#[derive(Clone, Debug)]
pub struct DisplayCtl {
    tx: mpsc::Sender<DisplayCommand>,
}

impl DisplayCtl {
    pub fn new(tx: mpsc::Sender<DisplayCommand>) -> Self {
        Self { tx }
    }

    pub async fn set_defaults(&self, width: u32, height: u32, refresh_hz: u32) {
        let _ = self
            .tx
            .send(DisplayCommand::SetDefaults {
                width,
                height,
                refresh_hz,
            })
            .await;
    }

    pub async fn acquire(
        &self,
        name: String,
        key: Option<String>,
        width: u32,
        height: u32,
    ) -> Result<DisplayInfo, String> {
        self.acquire_with_bitrate(name, key, width, height, None)
            .await
    }

    pub async fn acquire_with_bitrate(
        &self,
        name: String,
        key: Option<String>,
        width: u32,
        height: u32,
        bitrate_kbps: Option<u32>,
    ) -> Result<DisplayInfo, String> {
        let (reply, rx) = oneshot::channel();
        self.tx
            .send(DisplayCommand::Acquire {
                name,
                key,
                width,
                height,
                bitrate_kbps,
                reply,
            })
            .await
            .map_err(|_| "display hub stopped".to_string())?;
        rx.await.map_err(|_| "display hub stopped".to_string())?
    }

    pub async fn release(&self, id: &str) {
        let _ = self
            .tx
            .send(DisplayCommand::Release { id: id.to_string() })
            .await;
    }

    pub async fn attach(
        &self,
        id: Option<String>,
        key: Option<String>,
    ) -> Result<AttachedDisplay, String> {
        let (reply, rx) = oneshot::channel();
        self.tx
            .send(DisplayCommand::Attach { id, key, reply })
            .await
            .map_err(|_| "display hub stopped".to_string())?;
        rx.await.map_err(|_| "display hub stopped".to_string())?
    }

    pub async fn detach(&self, id: &str) {
        let _ = self
            .tx
            .send(DisplayCommand::Detach { id: id.to_string() })
            .await;
    }

    pub async fn idr(&self, id: &str) {
        let _ = self
            .tx
            .send(DisplayCommand::Idr { id: id.to_string() })
            .await;
    }

    pub async fn resize(
        &self,
        id: &str,
        width: u32,
        height: u32,
        refresh_hz: Option<u32>,
    ) -> Result<DisplayInfo, String> {
        let (reply, rx) = oneshot::channel();
        self.tx
            .send(DisplayCommand::Resize {
                id: id.to_string(),
                width,
                height,
                refresh_hz,
                reply,
            })
            .await
            .map_err(|_| "display hub stopped".to_string())?;
        rx.await.map_err(|_| "display hub stopped".to_string())?
    }

    pub async fn lookup(&self, id: Option<String>) -> Option<DisplayInfo> {
        let (reply, rx) = oneshot::channel();
        self.tx
            .send(DisplayCommand::Lookup { id, reply })
            .await
            .ok()?;
        rx.await.ok().flatten()
    }

    pub async fn input(&self, id: Option<String>, event: IncomingInput) {
        let _ = self.tx.send(DisplayCommand::Input { id, event }).await;
    }

    pub async fn shutdown(&self) {
        let (reply, rx) = oneshot::channel();
        if self
            .tx
            .send(DisplayCommand::Shutdown { reply })
            .await
            .is_ok()
        {
            let _ = rx.await;
        }
    }
}
