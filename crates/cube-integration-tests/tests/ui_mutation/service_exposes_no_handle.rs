// The service owns its commit path privately; callers cannot take the handle out of it.
use cube_orchestration::OrchestrationService;

fn main() {
    let svc: OrchestrationService = todo!();
    let _ = svc.path;
}
