use crate::plater::solution::Solution;

pub enum ProgressMessage<'request> {
    PreRun {
        total_placers: u32,
    },
    Placement {
        placer_index: u32,
        percentage: f64,
        total_placers: u32,
    },
    StringMessage(String),
    SolutionFound {
        placer_index: u32,
        solution: Solution<'request>,
    },
}

pub struct ProgressMessenger<F2: Fn(ProgressMessage)> {
    receiver_function: F2,
}

impl<'request, F2: Fn(ProgressMessage)> ProgressMessenger<F2> {
    pub fn new(f2: F2) -> Self {
        ProgressMessenger {
            receiver_function: f2,
        }
    }

    pub fn send_message<F: Fn() -> ProgressMessage<'request>>(&self, sender_function: F) {
        (self.receiver_function)(sender_function());
    }
}
