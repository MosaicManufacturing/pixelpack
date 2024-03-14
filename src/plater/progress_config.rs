pub enum ProgressMessage {
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
    },
}

pub struct ProgressMessenger<F2: Fn(ProgressMessage)> {
    receiver_function: F2,
}

impl<F2: Fn(ProgressMessage)> ProgressMessenger<F2> {
    pub fn new(f2: F2) -> Self {
        ProgressMessenger {
            receiver_function: f2,
        }
    }

    pub fn send_message<F: Fn() -> ProgressMessage>(&self, sender_function: F) {
        (self.receiver_function)(sender_function());
    }
}
