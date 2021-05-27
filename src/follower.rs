use crate::message::{Message, Message::*, NodeId, TermId};

struct Follower {
    id: NodeId,
    leader_id: NodeId,
    term_id: TermId,
    vote_for_future_term: Option<(TermId, NodeId)>,
}

impl Follower {
    fn receive(&mut self, m: Message) -> Message {
        // TODO break up this gigantic match
        // TODO check that the message is for us (`receiver_id`)
        return match m {
            AppendLog { sender_id, term_id, receiver_id } => {
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
            RequestVote { sender_id, proposed_term_id } => {
                if proposed_term_id < self.term_id {
                    return VoteNo { sender_id: self.id };
                }
                return match self.vote_for_future_term {
                    Some((term_id, node_id)) => {
                        if proposed_term_id > term_id {
                            return self.save_vote(proposed_term_id, sender_id);
                        }
                        if node_id == sender_id {
                            VoteYes { sender_id: self.id }
                        } else {
                            VoteNo { sender_id: self.id }
                        }
                    }
                    None => {
                        self.save_vote(proposed_term_id, sender_id)
                    }
                };
            }
            _ => {
                Reject
            }
        };
    }

    fn save_vote(&mut self, proposed_term_id: TermId, node_id: NodeId) -> Message {
        self.vote_for_future_term = Some((proposed_term_id, node_id));
        return VoteYes { sender_id: self.id };
    }
}

fn new_follower(id: NodeId, leader_id: NodeId, term_id: TermId) -> Follower {
    return Follower { id, leader_id, term_id, vote_for_future_term: Some((term_id, leader_id)) };
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MessageBuilder {
        id: NodeId,
        term_id: TermId,
    }

    impl MessageBuilder {
        fn append_log(&self) -> Message {
            AppendLog { sender_id: self.id, term_id: self.term_id, receiver_id: 981273461 } // TODO
        }
        fn request_vote(&self) -> Message {
            RequestVote { sender_id: self.id, proposed_term_id: self.term_id }
        }
    }

    fn default_message_builder() -> MessageBuilder {
        return MessageBuilder { id: 827349, term_id: 7812635 };
    }

    fn message_builder(id: NodeId, term_id: TermId) -> MessageBuilder {
        return MessageBuilder { id, term_id };
    }

    #[test]
    fn test_acknowledges_append_log_from_leader_of_same_term() {
        let leader = default_message_builder();
        let mut follower = new_follower(15, leader.id, leader.term_id);

        let result = follower.receive(leader.append_log());

        assert_eq!(result, Acknowledge { sender_id: follower.id });

        let mut other_follower = new_follower(16, leader.id, leader.term_id);
        let result = other_follower.receive(leader.append_log());
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
        let result = follower.receive(leader_in_old_term.append_log());
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
        let new_leader_in_new_term = message_builder(3142798, leader.term_id + 1);
        let mut follower = new_follower(15, leader.id, leader.term_id);

        let _result = follower.receive(new_leader_in_new_term.append_log());
        assert_eq!(follower.leader_id, new_leader_in_new_term.id);
        assert_eq!(follower.term_id, new_leader_in_new_term.term_id);
    }

    #[test]
    fn test_vote_for_past_term() {
        let leader = default_message_builder();
        let leader_in_old_term = message_builder(leader.id, leader.term_id - 1);
        let mut follower = new_follower(15, leader.id, leader.term_id);

        let result = follower.receive(leader_in_old_term.request_vote());
        assert_eq!(result, VoteNo { sender_id: follower.id });
    }

    #[test]
    fn test_votes_yes_for_future_term_for_first_node_no_for_second_even_if_replayed() {
        let prev_leader = default_message_builder();
        let mut follower = new_follower(15, prev_leader.id, prev_leader.term_id);

        let candidate_a = message_builder(987234, prev_leader.term_id + 1);
        let result = follower.receive(candidate_a.request_vote());
        assert_eq!(result, VoteYes { sender_id: follower.id });
        let result = follower.receive(candidate_a.request_vote());
        assert_eq!(result, VoteYes { sender_id: follower.id });

        let candidate_b = message_builder(109238, prev_leader.term_id + 1);
        let result = follower.receive(candidate_b.request_vote());
        assert_eq!(result, VoteNo { sender_id: follower.id });
        let result = follower.receive(candidate_b.request_vote());
        assert_eq!(result, VoteNo { sender_id: follower.id });
    }

    #[test]
    fn test_always_votes_yes_for_leader_in_same_term() {
        let leader = default_message_builder();
        let mut follower = new_follower(15, leader.id, leader.term_id);

        let result = follower.receive(leader.request_vote());
        assert_eq!(result, VoteYes { sender_id: follower.id });
    }

    #[test]
    fn test_votes_no_for_other_candidate_in_same_term() {
        let leader = default_message_builder();
        let candidate_a = message_builder(264785, leader.term_id);

        let mut follower = new_follower(15, leader.id, leader.term_id);

        let result = follower.receive(candidate_a.request_vote());
        assert_eq!(result, VoteNo { sender_id: follower.id });
    }

    #[test]
    fn test_when_voting_newest_term_always_wins() {
        let leader = default_message_builder();
        let mut follower = new_follower(15, leader.id, leader.term_id);

        let candidate_a = message_builder(987234, leader.term_id + 1);
        let result = follower.receive(candidate_a.request_vote());
        assert_eq!(result, VoteYes { sender_id: follower.id });

        let candidate_b = message_builder(287634, candidate_a.term_id + 1);
        let result = follower.receive(candidate_b.request_vote());
        assert_eq!(result, VoteYes { sender_id: follower.id });

        let result = follower.receive(candidate_a.request_vote());
        assert_eq!(result, VoteNo { sender_id: follower.id });
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
