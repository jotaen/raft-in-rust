use crate::message::{Message, Message::*, NodeId, TermId};

struct Leader {
    id: NodeId,
    term_id: TermId,
}

impl Leader {
    fn receive(&mut self, m: Message) -> Message {
        return Reject
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_do_nothing() {

    }
}
