use std::time::Duration;

use anyhow::Result;

use crate::client::sync::SyncConfig;
use crate::connection::ProtocolMessage;
use crate::inputs::input_buffer::InputBuffer;
use crate::packet::packet::PacketId;
use crate::packet::packet_manager::Payload;
use crate::tick::Tick;
use crate::{
    ChannelKind, ChannelRegistry, PingChannel, Protocol, ReadBuffer, SyncMessage, TickManager,
    TimeManager,
};

use super::ping_manager::PingConfig;
use super::sync::SyncManager;

// TODO: this layer of indirection is annoying, is there a better way?
//  maybe just pass the inner connection to ping_manager? (but harder to test)
pub struct Connection<P: Protocol> {
    pub(crate) base: crate::Connection<P>,

    // pub(crate) ping_manager: PingManager,
    pub(crate) input_buffer: InputBuffer<P::Input>,
    pub(crate) sync_manager: SyncManager,
    // TODO: maybe don't do any replication until connection is synced?
}

impl<P: Protocol> Connection<P> {
    // NOTE: looks like we're using SyncManager on the client, and PingManager on the server
    pub fn new(channel_registry: &ChannelRegistry, sync_config: SyncConfig) -> Self {
        Self {
            base: crate::Connection::new(channel_registry),
            // ping_manager: PingManager::new(ping_config),
            input_buffer: InputBuffer::default(),
            sync_manager: SyncManager::new(sync_config),
        }
    }

    /// Add an input for the given tick
    pub fn add_input(&mut self, input: P::Input, tick: Tick) {
        self.input_buffer.buffer.push(&tick, input);
    }

    pub fn update(&mut self, time_manager: &TimeManager, tick_manager: &TickManager) {
        self.base.update(time_manager, tick_manager);
        self.sync_manager.update(time_manager);
        // TODO: maybe prepare ping?
        // self.ping_manager.update(delta);

        // client send pings to estimate rtt
        if let Some(sync_ping) = self
            .sync_manager
            .maybe_prepare_ping(time_manager, tick_manager)
        {
            let message = ProtocolMessage::Sync(SyncMessage::TimeSyncPing(sync_ping));
            let channel = ChannelKind::of::<PingChannel>();
            self.base
                .message_manager
                .buffer_send(message, channel)
                .unwrap();
        }
    }

    pub fn recv_packet(&mut self, reader: &mut impl ReadBuffer) -> Result<()> {
        let tick = self.base.recv_packet(reader)?;
        if tick > self.sync_manager.latest_received_server_tick {
            self.sync_manager.latest_received_server_tick = tick;
            self.sync_manager.duration_since_latest_received_server_tick = Duration::default();
        }
        Ok(())
    }

    // pub fn buffer_ping(&mut self, time_manager: &TimeManager) -> Result<()> {
    //     if !self.ping_manager.should_send_ping() {
    //         return Ok(());
    //     }
    //
    //     let ping_message = self.ping_manager.prepare_ping(time_manager);
    //
    //     // info!("Sending ping {:?}", ping_message);
    //     trace!("Sending ping {:?}", ping_message);
    //
    //     let message = ProtocolMessage::Sync(SyncMessage::Ping(ping_message));
    //     let channel = ChannelKind::of::<DefaultUnreliableChannel>();
    //     self.base.message_manager.buffer_send(message, channel)
    // }

    // TODO: eventually call handle_ping and handle_pong directly from the connection
    //  without having to send to events

    // send pongs for every ping we received
    // pub fn buffer_pong(&mut self, time_manager: &TimeManager, ping: PingMessage) -> Result<()> {
    //     let pong_message = self.ping_manager.prepare_pong(time_manager, ping);
    //
    //     // info!("Sending ping {:?}", ping_message);
    //     trace!("Sending pong {:?}", pong_message);
    //     let message = ProtocolMessage::Sync(SyncMessage::Pong(pong_message));
    //     let channel = ChannelKind::of::<DefaultUnreliableChannel>();
    //     self.base.message_manager.buffer_send(message, channel)
    // }
}