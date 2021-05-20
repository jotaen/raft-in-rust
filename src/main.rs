use crate::Message::{Acknowledge, Reject, AppendLog, RequestLog};

fn main() {
    println!("Hello, world!");
}

struct Node {
    id: u32,
    leader_id: u32,
    term_id: u64,
}

#[derive(PartialEq)]
#[derive(Debug)]
enum Message {
    AppendLog { sender_id: u32, term_id: u64 },
    Acknowledge { sender_id: u32 },
    Reject,
    RequestLog { sender_id: u32 },
}

impl Node {
    fn receive(&mut self, m: Message) -> Message {
        return match m {
            AppendLog { sender_id, term_id } => {
                if sender_id == self.leader_id && term_id == self.term_id {
                    Acknowledge { sender_id: self.id }
                } else if term_id > self.term_id {
                    // TODO buffer all new append logs
                    self.leader_id = sender_id;
                    self.term_id = term_id;
                    RequestLog { sender_id: self.id }
                } else {
                    Reject
                }
            }
            _ => {
                Reject
            }
        };
    }
}

fn new_follower(id: u32, leader_id: u32, term_id: u64) -> Node {
    return Node { id, leader_id, term_id };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Message::{AppendLog, Reject, RequestLog};

    struct MessageBuilder {
        id: u32,
        term_id: u64,
    }

    impl MessageBuilder {
        fn append_log(&self) -> Message {
            AppendLog { sender_id: self.id, term_id: self.term_id }
        }
    }

    fn default_message_builder() -> MessageBuilder {
        return MessageBuilder { id: 827349, term_id: 7812635 };
    }

    fn message_builder(id: u32, term_id: u64) -> MessageBuilder {
        return MessageBuilder { id, term_id };
    }

    #[test]
    fn test_acknowledges_append_log_from_leader_of_same_term() {
        let leader = default_message_builder();
        let mut follower = new_follower(15, leader.id, leader.term_id);

        let result = follower.receive(leader.append_log());

        assert_eq!(result, Acknowledge { sender_id: follower.id });

        let mut other_follower = new_follower(16, leader.id, leader.term_id);
        let result = other_follower.receive(AppendLog { sender_id: leader.id, term_id: leader.term_id });
        assert_eq!(result, Acknowledge { sender_id: other_follower.id });
    }

    #[test]
    fn test_rejects_append_log_from_non_leader() {
        let leader = default_message_builder();
        let non_leader = message_builder(3, 1);

        let mut follower = new_follower(15, leader.id, leader.term_id);
        let result = follower.receive(non_leader.append_log());
        assert_eq!(result, Reject)
    }

    #[test]
    fn test_rejects_append_log_from_previous_term() {
        let leader = default_message_builder();
        let leader_in_old_term = message_builder(leader.id, leader.term_id - 1);
        let mut follower = new_follower(15, leader.id, leader.term_id);
        let result = follower.receive(AppendLog { sender_id: leader.id, term_id: leader_in_old_term.term_id });
        assert_eq!(result, Reject)
    }

    #[test]
    fn test_sends_request_log_if_term_is_behind() {
        let leader = default_message_builder();
        let leader_in_new_term = message_builder(leader.id, leader.term_id + 1);
        let mut follower = new_follower(15, leader.id, leader.term_id);
        let result = follower.receive(leader_in_new_term.append_log());
        assert_eq!(result, RequestLog { sender_id: follower.id })
    }

    #[test]
    fn test_updates_state_if_term_is_behind() {
        let leader = default_message_builder();
        let new_leader_in_new_term = message_builder(leader.id + 1, leader.term_id + 1);
        let mut follower = new_follower(15, leader.id, leader.term_id);

        let _result = follower.receive(new_leader_in_new_term.append_log());
        assert_eq!(follower.leader_id, new_leader_in_new_term.id);
        assert_eq!(follower.term_id, new_leader_in_new_term.term_id);
    }

    #[test]
    fn test_rejects_unexpected_message() {
        let leader_id = 1;
        let current_term = 1;
        let mut follower = new_follower(15, leader_id, current_term);
        let result = follower.receive(Acknowledge { sender_id: leader_id });
        assert_eq!(result, Reject)
    }
}
