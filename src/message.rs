pub type NodeId = u32;
pub type TermId = u64;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum Message {
    AppendLog { sender_id: NodeId, term_id: TermId, receiver_id: NodeId },
    Acknowledge { sender_id: NodeId },
    Reject,
    RequestLog { sender_id: NodeId },
    RequestVote { sender_id: NodeId, proposed_term_id: TermId },
    VoteNo { sender_id: NodeId },
    VoteYes { sender_id: NodeId },
    Heartbeat,
}
