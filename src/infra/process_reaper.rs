use sysinfo::{Pid, System};

/// [v2.5.0] Phase 35: Orphan Process Reaper
/// 비정상 종료된 smlcli 프로세스의 자식(손자 포함) 프로세스들을 정리합니다.
/// SMLCLI_PID 환경변수를 기반으로 추적하며, 해당 PID의 부모 프로세스가
/// 실제로 종료되었는지(고아 상태인지) 반드시 확인한 후에만 종료합니다.
/// 다른 정상 실행 중인 smlcli 인스턴스의 자식은 절대 건드리지 않습니다.
pub fn reap_orphans() {
    let mut sys = System::new_all();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    let current_pid = sysinfo::get_current_pid().unwrap_or(Pid::from(0));
    let current_pid_str = current_pid.to_string();

    let mut to_kill = Vec::new();

    for (pid, process) in sys.processes() {
        if *pid == current_pid {
            continue;
        }

        // SMLCLI_PID 환경 변수에서 원래 부모 PID 추출
        let parent_smlcli_pid: Option<String> = process.environ().iter().find_map(|env_os| {
            let env = env_os.to_string_lossy();
            env.strip_prefix("SMLCLI_PID=").map(|v| v.to_string())
        });

        if let Some(ref parent_pid_str) = parent_smlcli_pid {
            // 현재 프로세스의 자식이면 정상이므로 건드리지 않음
            if parent_pid_str == &current_pid_str {
                continue;
            }

            // [v2.5.0] 핵심 안전 검증: SMLCLI_PID에 기록된 부모 프로세스가
            // 아직 살아있는지 확인. 살아있으면 다른 정상 인스턴스의 자식이므로 보존.
            let parent_pid = Pid::from(parent_pid_str.parse::<usize>().unwrap_or(0));
            if sys.process(parent_pid).is_some() {
                // 부모 smlcli 프로세스가 아직 살아있음 → 정상 자식이므로 건드리지 않음
                continue;
            }

            // 부모 PID가 시스템에 없음 → 고아 프로세스 확정 → 종료 대상
            to_kill.push(*pid);
        }
    }

    if !to_kill.is_empty() {
        for pid in to_kill {
            if let Some(p) = sys.process(pid) {
                let _ = p.kill();
            }
        }
    }
}
