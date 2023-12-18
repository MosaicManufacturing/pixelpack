use std::future::Future;

pub trait FutureKillSwitch: Future {}

impl<T: Future> FutureKillSwitch for T {}

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
}

pub struct ProgressConfig<F2: Fn(ProgressMessage)> {
    on_progress: F2,
}

impl<F2: Fn(ProgressMessage)> ProgressConfig<F2> {
    pub fn new(f2: F2) -> Self {
        ProgressConfig { on_progress: f2 }
    }

    pub fn on_prog<F: Fn() -> ProgressMessage>(&self, f: F) {
        (self.on_progress)(f());
    }
}
