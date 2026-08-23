/// 환경변수만으로 process ownership을 증명할 수 없으므로 global scan/kill은
/// 비활성화되어 있습니다. 현재 runtime이 spawn한 shell/MCP process group은
/// 각 owner가 cancellation/shutdown에서 직접 종료합니다.
pub fn is_global_reaping_enabled() -> bool {
    false
}

pub fn reap_orphans() {
    // Compatibility no-op. A future lease registry must use an unforgeable
    // owner token and executable/start-time identity before global reaping returns.
}
