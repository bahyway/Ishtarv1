//! Jupyter wire protocol implementation for the DubSar kernel.
//! Implements the ZMQ messaging spec: https://jupyter-client.readthedocs.io/en/latest/messaging.html

use anyhow::Result;
use chrono::Utc;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::Sha256;
use bytes::Bytes;
use std::convert::TryInto;
use tracing::{debug, info, warn};
use uuid::Uuid;
use zeromq::{PubSocket, RepSocket, RouterSocket, Socket, SocketRecv, SocketSend, ZmqMessage};

use crate::magics::{MagicHandler, MagicResult};

// ─── Connection config ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ConnectionConfig {
    pub transport: String,
    pub ip: String,
    pub stdin_port: u16,
    pub control_port: u16,
    pub hb_port: u16,
    pub shell_port: u16,
    pub iopub_port: u16,
    pub signature_scheme: String,
    pub key: String,
}

impl ConnectionConfig {
    pub fn load(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }

    fn endpoint(&self, port: u16) -> String {
        format!("{}://{}:{}", self.transport, self.ip, port)
    }
}

// ─── Message types ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
struct Header {
    msg_id: String,
    session: String,
    username: String,
    date: String,
    msg_type: String,
    version: String,
}

impl Header {
    fn new(msg_type: &str, session: &str) -> Self {
        Self {
            msg_id: Uuid::new_v4().to_string(),
            session: session.to_string(),
            username: "dubsar".to_string(),
            date: Utc::now().to_rfc3339(),
            msg_type: msg_type.to_string(),
            version: "5.3".to_string(),
        }
    }
}

// ─── Kernel ───────────────────────────────────────────────────────────────────

pub struct DubSarKernel {
    config: ConnectionConfig,
    session: String,
    execution_count: u32,
    magic: MagicHandler,
}

impl DubSarKernel {
    pub async fn new(config: ConnectionConfig) -> Result<Self> {
        Ok(Self {
            config,
            session: Uuid::new_v4().to_string(),
            execution_count: 0,
            magic: MagicHandler::new(),
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("DubSar kernel connecting…");

        // Shell socket (ROUTER) — receives execute_request, kernel_info_request
        let mut shell = RouterSocket::new();
        shell.bind(&self.config.endpoint(self.config.shell_port)).await?;

        // IOPub socket (PUB) — sends stream, display_data, status messages
        let mut iopub = PubSocket::new();
        iopub.bind(&self.config.endpoint(self.config.iopub_port)).await?;

        // Control socket (ROUTER) — shutdown / interrupt
        let mut control = RouterSocket::new();
        control.bind(&self.config.endpoint(self.config.control_port)).await?;

        // Heartbeat (REP) — echoes back to show kernel is alive
        let hb_endpoint = self.config.endpoint(self.config.hb_port);
        tokio::spawn(async move {
            let mut hb = RepSocket::new();
            if hb.bind(&hb_endpoint).await.is_err() { return; }
            loop {
                if let Ok(msg) = hb.recv().await {
                    let _ = hb.send(msg).await;
                }
            }
        });

        info!("DubSar kernel ready on shell:{} iopub:{}", self.config.shell_port, self.config.iopub_port);

        // Publish initial idle status
        self.publish_status(&mut iopub, "starting").await?;
        self.publish_status(&mut iopub, "idle").await?;

        loop {
            tokio::select! {
                Ok(msg) = shell.recv() => {
                    self.handle_shell(msg, &mut shell, &mut iopub).await?;
                }
                Ok(msg) = control.recv() => {
                    let should_exit = self.handle_control(msg, &mut control).await?;
                    if should_exit { break; }
                }
            }
        }

        info!("DubSar kernel shutting down.");
        Ok(())
    }

    // ─── Shell handler ────────────────────────────────────────────────────────

    async fn handle_shell(
        &mut self,
        raw: ZmqMessage,
        shell: &mut RouterSocket,
        iopub: &mut PubSocket,
    ) -> Result<()> {
        let frames: Vec<Bytes> = raw.into_vec();
        if frames.len() < 6 { return Ok(()); }

        // ZMQ router adds identity frames before the Jupyter delimiter "<IDS|MSG>"
        let delim_pos = frames.iter().position(|f| f.as_ref() == b"<IDS|MSG>").unwrap_or(0);
        let ids = &frames[..delim_pos];

        let header: Header = serde_json::from_slice(&frames[delim_pos + 2])?;
        let content: Value = serde_json::from_slice(&frames[delim_pos + 5]).unwrap_or(json!({}));

        debug!("shell msg_type={}", header.msg_type);

        match header.msg_type.as_str() {
            "kernel_info_request" => {
                self.publish_status(iopub, "busy").await?;
                let reply = self.kernel_info_reply(&header);
                self.send_reply(shell, ids, &header, "kernel_info_reply", reply).await?;
                self.publish_status(iopub, "idle").await?;
            }

            "execute_request" => {
                self.publish_status(iopub, "busy").await?;
                self.execution_count += 1;
                let count = self.execution_count;
                let code = content["code"].as_str().unwrap_or("").to_string();

                // Publish execute_input
                self.publish_iopub(iopub, "execute_input", json!({
                    "code": code,
                    "execution_count": count,
                })).await?;

                let (reply_content, display) = self.execute_code(&code);

                // Publish display output if any
                if let Some(display_data) = display {
                    self.publish_iopub(iopub, &display_data.0, display_data.1).await?;
                }

                self.send_reply(shell, ids, &header, "execute_reply", reply_content).await?;
                self.publish_status(iopub, "idle").await?;
            }

            "is_complete_request" => {
                let code = content["code"].as_str().unwrap_or("");
                let reply = json!({ "status": if code.trim().is_empty() { "complete" } else { "complete" } });
                self.send_reply(shell, ids, &header, "is_complete_reply", reply).await?;
            }

            "comm_info_request" => {
                self.send_reply(shell, ids, &header, "comm_info_reply", json!({ "comms": {} })).await?;
            }

            other => {
                warn!("Unhandled shell message: {}", other);
            }
        }

        Ok(())
    }

    // ─── Code execution ───────────────────────────────────────────────────────

    fn execute_code(&mut self, code: &str) -> (Value, Option<(String, Value)>) {
        let trimmed = code.trim();

        if trimmed.is_empty() {
            return (self.ok_reply(), None);
        }

        match self.magic.handle(trimmed) {
            MagicResult::Text(text) => {
                let display = Some((
                    "stream".to_string(),
                    json!({ "name": "stdout", "text": text }),
                ));
                (self.ok_reply(), display)
            }

            MagicResult::Json(val) => {
                let text = serde_json::to_string_pretty(&val).unwrap_or_default();
                let display = Some((
                    "display_data".to_string(),
                    json!({
                        "data": { "text/plain": text, "application/json": val },
                        "metadata": {},
                        "transient": {}
                    }),
                ));
                (self.ok_reply(), display)
            }

            MagicResult::Image(b64) => {
                let display = Some((
                    "display_data".to_string(),
                    json!({
                        "data": { "image/png": b64 },
                        "metadata": { "image/png": { "width": 800, "height": 600 } },
                        "transient": {}
                    }),
                ));
                (self.ok_reply(), display)
            }

            MagicResult::NotAMagic => {
                // Plain Rust code cells — explain the evcxr limitation
                let text = format!(
                    "DubSar kernel received Rust code ({} chars).\n\
                     Interactive Rust evaluation requires evcxr_jupyter.\n\
                     Use %%dubsar magics for simulation control:\n\
                       %%dubsar init-storage 100000\n\
                       %%dubsar step 10\n\
                       %%dubsar render\n\
                       %%dubsar stats",
                    trimmed.len()
                );
                let display = Some((
                    "stream".to_string(),
                    json!({ "name": "stdout", "text": text }),
                ));
                (self.ok_reply(), display)
            }
        }
    }

    fn ok_reply(&self) -> Value {
        json!({
            "status": "ok",
            "execution_count": self.execution_count,
            "user_expressions": {},
            "payload": []
        })
    }

    fn kernel_info_reply(&self, _parent: &Header) -> Value {
        json!({
            "status": "ok",
            "protocol_version": "5.3",
            "implementation": "dubsar",
            "implementation_version": "0.1.0",
            "language_info": {
                "name": "rust",
                "version": "1.78",
                "mimetype": "text/x-rustsrc",
                "file_extension": ".rs",
                "pygments_lexer": "rust",
                "codemirror_mode": "rust"
            },
            "banner": "DubSar IDE Kernel — 7D Particle Simulation + Storage Fragmentation\nBuild: Rust + Vulkan (wgpu software backend on Hyper-V)",
            "help_links": []
        })
    }

    // ─── Control handler ──────────────────────────────────────────────────────

    async fn handle_control(&self, raw: ZmqMessage, control: &mut RouterSocket) -> Result<bool> {
        let frames: Vec<Bytes> = raw.into_vec();
        if frames.len() < 6 { return Ok(false); }

        let delim_pos = frames.iter().position(|f| f.as_ref() == b"<IDS|MSG>").unwrap_or(0);
        let ids = &frames[..delim_pos];
        let header: Header = serde_json::from_slice(&frames[delim_pos + 2])?;

        if header.msg_type == "shutdown_request" {
            info!("Shutdown requested");
            let reply = json!({ "status": "ok", "restart": false });
            self.send_reply(control, ids, &header, "shutdown_reply", reply).await?;
            return Ok(true);
        }

        Ok(false)
    }

    // ─── ZMQ helpers ─────────────────────────────────────────────────────────

    async fn send_reply(
        &self,
        socket: &mut RouterSocket,
        ids: &[Bytes],
        parent: &Header,
        msg_type: &str,
        content: Value,
    ) -> Result<()> {
        let reply_header = Header::new(msg_type, &self.session);
        let signature = self.sign(&parent_header_bytes(parent), &content);

        let mut frames: Vec<Bytes> = ids.to_vec();
        frames.push(Bytes::from_static(b"<IDS|MSG>"));
        frames.push(Bytes::from(signature.into_bytes()));
        frames.push(Bytes::from(serde_json::to_vec(&reply_header)?));
        frames.push(Bytes::from(serde_json::to_vec(parent)?));
        frames.push(Bytes::from_static(b"{}"));
        frames.push(Bytes::from(serde_json::to_vec(&content)?));

        let msg: ZmqMessage = frames.try_into()
            .map_err(|_| anyhow::anyhow!("Failed to build ZmqMessage (empty frame list)"))?;
        socket.send(msg).await?;
        Ok(())
    }

    async fn publish_iopub(&self, iopub: &mut PubSocket, msg_type: &str, content: Value) -> Result<()> {
        let header = Header::new(msg_type, &self.session);
        let signature = self.sign(&serde_json::to_vec(&header)?, &content);

        let topic = format!("dubsar.{}", msg_type);
        let frames: Vec<Bytes> = vec![
            Bytes::from(topic.into_bytes()),
            Bytes::from_static(b"<IDS|MSG>"),
            Bytes::from(signature.into_bytes()),
            Bytes::from(serde_json::to_vec(&header)?),
            Bytes::from_static(b"{}"),
            Bytes::from_static(b"{}"),
            Bytes::from(serde_json::to_vec(&content)?),
        ];

        let msg: ZmqMessage = frames.try_into()
            .map_err(|_| anyhow::anyhow!("Failed to build ZmqMessage for iopub"))?;
        iopub.send(msg).await?;
        Ok(())
    }

    async fn publish_status(&self, iopub: &mut PubSocket, state: &str) -> Result<()> {
        self.publish_iopub(iopub, "status", json!({ "execution_state": state })).await
    }

    fn sign(&self, header_bytes: &[u8], content: &Value) -> String {
        if self.config.key.is_empty() { return String::new(); }
        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(self.config.key.as_bytes()).unwrap();
        mac.update(header_bytes);
        mac.update(serde_json::to_vec(content).unwrap_or_default().as_slice());
        hex::encode(mac.finalize().into_bytes())
    }
}

fn parent_header_bytes(h: &Header) -> Vec<u8> {
    serde_json::to_vec(h).unwrap_or_default()
}
