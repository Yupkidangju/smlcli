use tokio::process::Command;

/// Checks if bubblewrap (bwrap) is available in the system PATH.
pub fn detect_backend() -> bool {
    std::process::Command::new("bwrap")
        .arg("--version")
        .output()
        .is_ok()
}

/// [v3.3.2] 감사 HIGH-1 수정: bubblewrap 래핑 셸 명령 실행.
/// 이전: `bash <script_path>`로 실행 → raw 명령 문자열이 파일 경로로 해석되어 실패.
/// 수정: `bash -c <cmd>`로 실행 → 실제 셸 명령이 정상 동작.
pub fn wrap_command_bwrap(
    cwd: &str,
    cmd: &str,
    allow_network: bool,
    extra_binds: &[String],
) -> Command {
    // [v3.3.2] 로컬 변수를 bwrap_cmd로 명명하여 파라미터 cmd(셸 명령)와 충돌 방지
    let mut bwrap_cmd = Command::new("bwrap");

    // 기본 샌드박스 마운트 설정
    bwrap_cmd
        .arg("--ro-bind")
        .arg("/usr")
        .arg("/usr")
        .arg("--ro-bind-try")
        .arg("/lib")
        .arg("/lib")
        .arg("--ro-bind-try")
        .arg("/lib64")
        .arg("/lib64")
        .arg("--ro-bind-try")
        .arg("/bin")
        .arg("/bin")
        .arg("--ro-bind-try")
        .arg("/sbin")
        .arg("/sbin")
        .arg("--ro-bind-try")
        .arg("/etc")
        .arg("/etc")
        .arg("--dev")
        .arg("/dev")
        .arg("--proc")
        .arg("/proc")
        .arg("--tmpfs")
        .arg("/tmp")
        .arg("--bind")
        .arg(cwd)
        .arg(cwd) // 워크스페이스 디렉터리 바인드
        .arg("--dir")
        .arg("/run/user");

    // 네트워크 격리
    if !allow_network {
        bwrap_cmd.arg("--unshare-net");
    }

    // 추가 바인드
    for bind in extra_binds {
        let parts: Vec<&str> = bind.split(':').collect();
        if parts.len() == 2 {
            bwrap_cmd.arg("--bind").arg(parts[0]).arg(parts[1]);
        } else if parts.len() == 1 {
            bwrap_cmd.arg("--bind").arg(parts[0]).arg(parts[0]);
        }
    }

    // [v3.3.2] 감사 HIGH-1 수정: bash -c 로 셸 명령 실행.
    // 이전: `bash <raw_cmd_string>` → 파일 경로로 해석되어 실패.
    // 수정: `bash -c <raw_cmd_string>` → 셸 명령으로 정상 해석.
    bwrap_cmd.arg("--chdir").arg(cwd);
    bwrap_cmd.arg("bash").arg("-c").arg(cmd);

    bwrap_cmd
}
