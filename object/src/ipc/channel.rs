use alloc::{collections::vec_deque::VecDeque, vec::Vec};
use spin::Mutex;

use crate::{
    Errno, Result,
    object::{Handle, KernelObject, TypedKObject, WeakTypedKObject},
};

pub struct Channel {
    peer: WeakTypedKObject<Channel>,
    recv_queue: Mutex<VecDeque<MessagePacket>>,
}

#[derive(Default)]
pub struct MessagePacket {
    pub data: Vec<u8>,
    pub handles: Vec<Handle>,
}

impl KernelObject for Channel {
    fn peer(&self) -> Result<crate::object::KObject> {
        self.peer
            .upgrade()
            .map(|p| p.into())
            .ok_or(Errno::PeerClosed.no_message())
    }
}

impl Channel {
    pub fn new() -> (TypedKObject<Self>, TypedKObject<Self>) {
        let mut channel0 = TypedKObject::new(Self {
            peer: WeakTypedKObject::default(),
            recv_queue: Mutex::new(VecDeque::new()),
        });
        let channel1 = TypedKObject::new(Self {
            peer: channel0.downgrade(),
            recv_queue: Mutex::new(VecDeque::new()),
        });

        unsafe {
            channel0.get_mut_unchecked().peer = channel1.downgrade();
        }

        (channel0, channel1)
    }
}

impl Channel {
    pub fn peer_closed(&self) -> bool {
        self.peer.upgrade().is_none()
    }
}

impl Channel {
    pub fn read(&self) -> Result<MessagePacket> {
        let mut recv_queue = self.recv_queue.lock();
        if let Some(msg) = recv_queue.pop_front() {
            Ok(msg)
        } else if self.peer_closed() {
            Err(Errno::PeerClosed.no_message())
        } else {
            Err(Errno::ShouldWait.no_message())
        }
    }

    pub fn write(&self, msg: MessagePacket) -> Result<()> {
        let peer = self.peer.upgrade().ok_or(Errno::PeerClosed.no_message())?;
        peer.push(msg);
        Ok(())
    }

    fn push(&self, msg: MessagePacket) {
        let mut recv_queue = self.recv_queue.lock();
        recv_queue.push_back(msg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_new() {
        let (channel0, channel1) = Channel::new();
        assert!(channel0.peer().is_ok());
        assert!(channel1.peer().is_ok());
    }

    #[test]
    fn read_write() {
        let (channel0, channel1) = Channel::new();

        channel0
            .write(MessagePacket {
                data: Vec::from("Hello 1"),
                handles: Vec::new(),
            })
            .unwrap();
        channel1
            .write(MessagePacket {
                data: Vec::from("Hello 0"),
                handles: Vec::new(),
            })
            .unwrap();

        let recv_msg = channel0.read().unwrap();
        assert_eq!(recv_msg.data, Vec::from("Hello 0"));

        let recv_msg = channel1.read().unwrap();
        assert_eq!(recv_msg.data, Vec::from("Hello 1"));
    }
}
