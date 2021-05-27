use crate::message::{Message, Message::*, NodeId, TermId};

trait Outbox {
    fn deliver_message(&mut self, m: Message);
}

struct MockOutbox {
    buffer: Vec<Message>,
}

impl Outbox for MockOutbox {
    fn deliver_message(&mut self, m: Message) {
        self.buffer.push(m);
    }
}

struct Leader<'a> {
    id: NodeId,
    term_id: TermId,
    outbox: &'a mut dyn Outbox,
}

impl Leader<'_> {
    fn receive(&mut self, m: Message) -> Message {
        return Reject;
    }

    fn broadcast_heartbeat(&mut self) {
        self.outbox.deliver_message(Message::Heartbeat);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_broadcasts_heartbeat_when_triggered() {
        let mut o = MockOutbox { buffer: vec![] };
        let mut leader = Leader { id: 1, term_id: 2, outbox: &mut o };
        leader.broadcast_heartbeat();
        assert_eq!(o.buffer.len(), 1);
        assert_eq!(o.buffer[0], Heartbeat);
    }
}
