use crate::Message::{Acknowledge, Reject, AppendLog};

fn main() {
    println!("Hello, world!");
}

struct Node {
    id: u32,
    leader_id: u32,
}

#[derive(PartialEq)]
#[derive(Debug)]
enum Message {
    AppendLog { sender_id: u32 },
    Acknowledge { sender_id: u32 },
    Reject,
}

impl Node {
    fn receive(&self, m: Message) -> Message {
        return match m {
            AppendLog { sender_id } => {
                if sender_id == 1 {
                    Acknowledge { sender_id: 123 }
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

fn new_follower(id: u32, leader_id: u32) -> Node {
    return Node{ id, leader_id };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Message::{AppendLog, Reject};

    #[test]
    fn test_acknowledges_append_log_from_leader() {
        let leader_id = 1;
        let node = new_follower(15, leader_id);
        let result = node.receive(AppendLog { sender_id: leader_id });
        assert_eq!(result, Acknowledge { sender_id: 123 })
    }

    #[test]
    fn test_rejects_append_log_from_non_leader() {
        let other_follower_id = 2;
        let node = new_follower(15, other_follower_id);
        let result = node.receive(AppendLog { sender_id: other_follower_id });
        assert_eq!(result, Reject)
    }

    #[test]
    fn test_rejects_unexpected_message() {
        let leader_id = 1;
        let node = new_follower(15, leader_id);
        let result = node.receive(Acknowledge { sender_id: leader_id });
        assert_eq!(result, Reject)
    }
}
