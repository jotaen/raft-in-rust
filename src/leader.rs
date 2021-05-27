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
    followers: Vec<NodeId>,
    expected_acks: Vec<NodeId>,
}

fn new_leader(id: NodeId, term_id: TermId, outbox: &mut dyn Outbox, followers: Vec<NodeId>) -> Leader {
    return Leader { id, term_id, outbox, followers, expected_acks: vec![] };
}

impl Leader<'_> {
    fn receive(&mut self, m: Message) {
        match m {
            Acknowledge { sender_id } => {
                let index = self.expected_acks.iter().position(|n| *n == sender_id);
                index.map(|i| self.expected_acks.remove(i));
            }
            _ => {}
        }
    }

    fn trigger_retries(&mut self) {
        for e in self.expected_acks.as_slice() {
            self.outbox.deliver_message(Message::AppendLog { term_id: self.term_id, sender_id: self.id, receiver_id: *e });
        }
    }

    fn trigger_heartbeat(&mut self) {
        for f in self.followers.as_slice() {
            self.outbox.deliver_message(Message::AppendLog { term_id: self.term_id, sender_id: self.id, receiver_id: *f });
            self.expected_acks.push(*f);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LEADER_ID: NodeId = 1;
    const TERM_ID: TermId = 2;

    #[test]
    fn test_broadcasts_append_log_message_when_heartbeat_triggered() {
        let mut o = MockOutbox { buffer: vec![] };
        let mut leader = new_leader(LEADER_ID, TERM_ID, &mut o, vec![3672, 951287]);
        leader.trigger_heartbeat();
        assert_eq!(o.buffer, vec![
            AppendLog { term_id: TERM_ID, sender_id: LEADER_ID, receiver_id: 3672 },
            AppendLog { term_id: TERM_ID, sender_id: LEADER_ID, receiver_id: 951287 },
        ]);
    }

    #[test]
    fn test_broadcast_retries_followers_without_acknowledgement() {
        let mut o = MockOutbox { buffer: vec![] };
        let mut leader = new_leader(LEADER_ID, TERM_ID, &mut o, vec![3672, 951287]);
        leader.trigger_heartbeat();
        leader.receive(Acknowledge { sender_id: 951287 });

        let mut o2 = MockOutbox { buffer: vec![] };
        leader.outbox = &mut o2;
        leader.trigger_retries();
        assert_eq!(o2.buffer, vec![
            AppendLog { term_id: TERM_ID, sender_id: LEADER_ID, receiver_id: 3672 },
        ]);
    }

    #[test]
    fn test_receive_ignores_acknowledgements_from_other_nodes() {
        let mut o = MockOutbox { buffer: vec![] };
        let mut leader = new_leader(LEADER_ID, TERM_ID, &mut o, vec![3672, 951287]);
        leader.trigger_heartbeat();
        leader.receive(Acknowledge { sender_id: 8796324 });
    }
}
