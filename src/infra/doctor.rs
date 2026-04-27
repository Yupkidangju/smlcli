use std::process::Command;

pub enum DiagnosticStatus {
    Ok(String),
    Warn(String),
    Error(String),
}

impl DiagnosticStatus {
    pub fn display(&self) {
        match self {
            DiagnosticStatus::Ok(msg) => println!("  [OK] {}", msg),
            DiagnosticStatus::Warn(msg) => println!("[WARN] {}", msg),
            DiagnosticStatus::Error(msg) => println!("[ERROR] {}", msg),
        }
    }
}

pub struct DoctorReport {
    pub api_status: DiagnosticStatus,
    pub git_status: DiagnosticStatus,
    pub config_status: DiagnosticStatus,
    pub term_status: DiagnosticStatus,
    pub sandbox_status: DiagnosticStatus,
}

impl DoctorReport {
    pub async fn run_diagnostics() -> Self {
        let config_status = Self::check_config().await;
        let api_status = Self::check_api(&config_status).await;
        let git_status = Self::check_git();
        let term_status = Self::check_terminal();
        let sandbox_status = Self::check_sandbox();

        Self {
            api_status,
            git_status,
            config_status,
            term_status,
            sandbox_status,
        }
    }

    pub fn has_issues(&self) -> bool {
        matches!(
            self.api_status,
            DiagnosticStatus::Warn(_) | DiagnosticStatus::Error(_)
        ) || matches!(self.git_status, DiagnosticStatus::Error(_))
            || matches!(
                self.config_status,
                DiagnosticStatus::Warn(_) | DiagnosticStatus::Error(_)
            )
            || matches!(self.term_status, DiagnosticStatus::Error(_))
            || matches!(self.sandbox_status, DiagnosticStatus::Error(_))
    }

    fn check_sandbox() -> DiagnosticStatus {
        if crate::infra::sandbox::detect_backend() {
            DiagnosticStatus::Ok("bubblewrap 샌드박스 백엔드 설치됨".into())
        } else {
            DiagnosticStatus::Warn("bubblewrap이 설치되지 않았습니다. 샌드박스 기능을 사용할 수 없습니다. (제안: apt install bubblewrap)".into())
        }
    }

    async fn check_config() -> DiagnosticStatus {
        match crate::infra::config_store::load_config().await {
            Ok(Some(settings)) => DiagnosticStatus::Ok(format!(
                "설정 파일 정상 로드 (공급자: {}, 버전: {})",
                settings.default_provider, settings.version
            )),
            Ok(None) => DiagnosticStatus::Warn(
                "설정 파일이 없습니다. 'smlcli run'을 통해 초기 마법사를 진행하세요.".into(),
            ),
            Err(e) => DiagnosticStatus::Error(format!(
                "설정 파일 파싱 또는 접근 오류: {}\n(제안: ~/.smlcli/config.toml 권한 확인)",
                e
            )),
        }
    }

    async fn check_api(config_status: &DiagnosticStatus) -> DiagnosticStatus {
        if let DiagnosticStatus::Ok(_) = config_status {
            // Check if any keys are saved
            let settings = crate::infra::config_store::load_config()
                .await
                .unwrap_or(None);
            if let Some(s) = settings {
                if s.encrypted_keys.is_empty() {
                    return DiagnosticStatus::Warn("저장된 API 키가 없습니다. API를 사용할 수 없습니다.\n(제안: 'smlcli run' 설정 마법사에서 키 입력)".into());
                }

                // [v2.3.0] Phase 31: Doctor Timeout & Network Check
                let client = reqwest::Client::new();
                let ping = tokio::time::timeout(
                    std::time::Duration::from_secs(5),
                    client.get("https://openrouter.ai/api/v1/auth/key").send(),
                )
                .await;

                match ping {
                    Ok(Ok(_resp)) => {
                        // 200 or 401(unauthorized since we didn't send a key) is fine, it means network is reachable
                        DiagnosticStatus::Ok(format!(
                            "API 키 {}개 보관 중 (네트워크 정상)",
                            s.encrypted_keys.len()
                        ))
                    }
                    Ok(Err(_)) => {
                        // Network error but not timeout
                        DiagnosticStatus::Warn(format!(
                            "API 키 {}개 보관 중이나, 네트워크 연결 불안정",
                            s.encrypted_keys.len()
                        ))
                    }
                    Err(_) => {
                        // Timeout
                        DiagnosticStatus::Error("API 연결 실패(Timeout): 5초 초과".into())
                    }
                }
            } else {
                DiagnosticStatus::Warn("설정이 없어 API 키를 확인할 수 없습니다.".into())
            }
        } else {
            DiagnosticStatus::Warn("설정 오류로 인해 API 키를 확인할 수 없습니다.".into())
        }
    }

    fn check_git() -> DiagnosticStatus {
        match Command::new("git").arg("--version").output() {
            Ok(out) if out.status.success() => {
                let v = String::from_utf8_lossy(&out.stdout).trim().to_string();

                // Check workspace
                match Command::new("git")
                    .arg("rev-parse")
                    .arg("--is-inside-work-tree")
                    .output()
                {
                    Ok(w_out) if w_out.status.success() => {
                        DiagnosticStatus::Ok(format!("{} (Git 워크스페이스 활성)", v))
                    }
                    _ => DiagnosticStatus::Warn(format!(
                        "{} (현재 디렉토리는 Git 워크스페이스가 아닙니다. 체크포인트 기능 제한됨)",
                        v
                    )),
                }
            }
            _ => DiagnosticStatus::Error(
                "Git이 설치되어 있지 않거나 PATH에 없습니다.\n(제안: Git을 설치해주세요)".into(),
            ),
        }
    }

    fn check_terminal() -> DiagnosticStatus {
        let is_tty = crossterm::tty::IsTty::is_tty(&std::io::stdout());
        if is_tty {
            DiagnosticStatus::Ok("표준 출력(TTY) 터미널 연동 정상 (ANSI 지원)".into())
        } else {
            DiagnosticStatus::Warn(
                "터미널이 TTY 환경이 아닙니다. TUI가 정상적으로 표시되지 않을 수 있습니다.".into(),
            )
        }
    }

    pub fn print_report(&self) {
        println!("🩺 smlcli doctor — 시스템 진단 리포트");
        println!(
            "빌드 버전: v{} ({} - {})",
            env!("CARGO_PKG_VERSION"),
            crate::shadow::build::SHORT_COMMIT,
            crate::shadow::build::BUILD_TIME
        );
        println!();
        println!("--- 설정(Config) 상태 ---");
        self.config_status.display();
        println!("\n--- API 및 인증(Auth) 상태 ---");
        self.api_status.display();
        println!("\n--- Git 환경 상태 ---");
        self.git_status.display();
        println!("\n--- 샌드박스 상태 ---");
        self.sandbox_status.display();
        println!("\n--- 터미널 환경 상태 ---");
        self.term_status.display();
        println!("\n진단 완료.");
    }
}
