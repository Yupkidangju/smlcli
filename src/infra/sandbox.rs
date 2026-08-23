use tokio::process::Command;

static BWRAP_CAPABILITY: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

/// `bwrap --version`이 아니라 실제 namespace 생성 capability를 확인합니다.
pub fn detect_backend() -> bool {
    *BWRAP_CAPABILITY.get_or_init(|| {
        std::process::Command::new("bwrap")
            .args(["--ro-bind", "/", "/", "--", "/bin/true"])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
    })
}

pub fn ensure_backend_usable() -> anyhow::Result<()> {
    if detect_backend() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "bubblewrap namespace capability를 사용할 수 없어 sandbox command를 거부합니다"
        ))
    }
}

pub fn validate_extra_binds(
    extra_binds: &[String],
    trusted_extra_roots: &[String],
) -> anyhow::Result<Vec<(std::path::PathBuf, std::path::PathBuf)>> {
    let trusted = trusted_extra_roots
        .iter()
        .map(std::fs::canonicalize)
        .collect::<std::io::Result<Vec<_>>>()?;
    let mut validated = Vec::new();

    for (index, bind) in extra_binds.iter().enumerate() {
        let (host, guest) = bind.split_once(':').map_or_else(
            || (bind.as_str(), format!("/workspace-extra/{index}")),
            |(host, guest)| (host, guest.to_string()),
        );
        let host = std::fs::canonicalize(host).map_err(|error| {
            anyhow::anyhow!("sandbox extra bind host 확인 실패 ({host}): {error}")
        })?;
        if !host.is_dir() || !trusted.iter().any(|root| host.starts_with(root)) {
            return Err(anyhow::anyhow!(
                "sandbox extra bind는 trusted extra workspace directory만 허용합니다: {}",
                host.display()
            ));
        }

        let guest = std::path::PathBuf::from(guest);
        let safe_components = guest.is_absolute()
            && guest.starts_with("/workspace-extra")
            && guest.components().all(|component| {
                matches!(
                    component,
                    std::path::Component::RootDir | std::path::Component::Normal(_)
                )
            });
        if !safe_components || guest == std::path::Path::new("/workspace-extra") {
            return Err(anyhow::anyhow!(
                "sandbox extra bind guest path는 /workspace-extra/<name> 아래여야 합니다: {}",
                guest.display()
            ));
        }
        validated.push((host, guest));
    }
    Ok(validated)
}

/// [v3.3.2] 감사 HIGH-1 수정: bubblewrap 래핑 셸 명령 실행.
/// 이전: `bash <script_path>`로 실행 → raw 명령 문자열이 파일 경로로 해석되어 실패.
/// 수정: `bash -c <cmd>`로 실행 → 실제 셸 명령이 정상 동작.
pub fn wrap_command_bwrap(
    cwd: &str,
    cmd: &str,
    allow_network: bool,
    extra_binds: &[String],
    trusted_extra_roots: &[String],
) -> anyhow::Result<Command> {
    ensure_backend_usable()?;
    let validated_extra_binds = validate_extra_binds(extra_binds, trusted_extra_roots)?;
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
        .arg(crate::infra::workspace_harness::WORKSPACE_GUEST_ROOT)
        .arg("--dir")
        .arg("/run/user");

    // 네트워크 격리
    if !allow_network {
        bwrap_cmd.arg("--unshare-net");
    }

    // 추가 바인드
    for (host, guest) in validated_extra_binds {
        bwrap_cmd.arg("--ro-bind").arg(host).arg(guest);
    }

    // [v3.3.2] 감사 HIGH-1 수정: bash -c 로 셸 명령 실행.
    // 이전: `bash <raw_cmd_string>` → 파일 경로로 해석되어 실패.
    // 수정: `bash -c <raw_cmd_string>` → 셸 명령으로 정상 해석.
    bwrap_cmd
        .arg("--chdir")
        .arg(crate::infra::workspace_harness::WORKSPACE_GUEST_ROOT);
    bwrap_cmd.arg("bash").arg("-c").arg(cmd);

    Ok(bwrap_cmd)
}
