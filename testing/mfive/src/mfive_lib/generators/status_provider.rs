use chain_impl_mockchain::fragment::FragmentId;
use quibitous_automation::quibitous::FragmentNode;
use quibitous_automation::quibitous::RemoteQuibitous;
use quibitous_lib::{
    interfaces::{FragmentLog, FragmentStatus},
    time::SystemTime,
};
use quibitestkit::load::RequestStatusProvider;
use quibitestkit::load::{Id, Status};

pub struct FragmentStatusProvider {
    quibitous: RemoteQuibitous,
}

impl FragmentStatusProvider {
    pub fn new(quibitous: RemoteQuibitous) -> Self {
        Self { quibitous }
    }
}

impl RequestStatusProvider for FragmentStatusProvider {
    fn get_statuses(&self, ids: &[Id]) -> Vec<Status> {
        let fragment_logs = self.quibitous.fragment_logs().unwrap();
        fragment_logs
            .iter()
            .filter(|(id, _)| ids.contains(&id.to_string()))
            .map(|(id, fragment_log)| into_status(fragment_log, id))
            .collect()
    }
}

fn into_status(fragment_log: &FragmentLog, id: &FragmentId) -> Status {
    match fragment_log.status() {
        FragmentStatus::Pending => {
            let duration = SystemTime::now()
                .duration_since(*fragment_log.received_at())
                .unwrap();
            Status::new_pending(duration.into(), id.to_string())
        }
        FragmentStatus::Rejected { reason } => {
            let duration = fragment_log
                .last_updated_at()
                .duration_since(*fragment_log.received_at())
                .unwrap();
            Status::new_failure(duration.into(), id.to_string(), reason.to_string())
        }
        FragmentStatus::InABlock { .. } => {
            let duration = fragment_log
                .last_updated_at()
                .duration_since(*fragment_log.received_at())
                .unwrap();
            Status::new_success(duration.into(), id.to_string())
        }
    }
}
