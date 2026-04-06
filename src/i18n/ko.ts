export const ko = {
  // Status
  "status.connected": "연결됨",
  "status.connecting": "연결 중...",
  "status.disconnected": "연결 안됨",
  "status.error": "오류",
  "status.unknown": "알 수 없음",

  // Actions
  "action.connect": "연결",
  "action.disconnect": "연결 해제",
  "action.reconnect": "재연결",
  "action.edit": "편집",
  "action.delete": "삭제",
  "action.save": "저장",
  "action.cancel": "취소",
  "action.browse": "찾기",
  "action.addRule": "+ 규칙 추가",

  // Connection form
  "form.newConnection": "새 연결",
  "form.editConnection": "연결 편집",
  "form.serverInfo": "서버 정보",
  "form.name": "이름",
  "form.host": "호스트",
  "form.port": "포트",
  "form.username": "사용자",
  "form.auth": "인증",
  "form.authMethod": "인증 방식",
  "form.password": "비밀번호",
  "form.keyFile": "키 파일",
  "form.keyFilePassphrase": "키 파일 + 암호",
  "form.keyFilePath": "키 파일 경로",
  "form.keyPassphrase": "키 파일 암호",
  "form.passwordStored": "저장된 비밀번호 있음 (변경 시 입력)",
  "form.passwordEnter": "비밀번호 입력",
  "form.deleteStoredPassword": "저장된 비밀번호 삭제",
  "form.forwardingRules": "포워딩 규칙",
  "form.noRules": "포워딩 규칙이 없습니다",
  "form.options": "옵션",
  "form.autoConnect": "시작 시 자동 연결",
  "form.saveFailed": "저장 실패",

  // Forwarding
  "forward.type": "유형",
  "forward.bindAddress": "바인드 주소",
  "forward.bindPort": "바인드 포트",
  "forward.remoteHost": "리모트 호스트",
  "forward.remotePort": "리모트 포트",
  "forward.socks5": "SOCKS5 프록시",
  "forward.socks5Bind": "바인드 주소 (SOCKS5)",
  "forward.port": "포트",
  "forward.disabled": "비활성",

  // Settings
  "settings.title": "설정",
  "settings.general": "일반",
  "settings.autostart": "Windows 시작 시 자동 실행",
  "settings.autostartDesc": "활성화하면 Windows 로그인 시 트레이에서 자동으로 시작됩니다. 개별 프로파일의 \"자동 연결\" 설정과 함께 사용하세요.",
  "settings.data": "데이터",
  "settings.export": "설정 내보내기",
  "settings.import": "설정 가져오기",
  "settings.importExportDesc": "프로파일 설정을 JSON 파일로 내보내거나 가져올 수 있습니다. 비밀번호는 포함되지 않습니다.",
  "settings.exported": "설정을 내보냈습니다.",
  "settings.imported": "설정을 가져왔습니다. 새로고침하면 반영됩니다.",
  "settings.exportFailed": "내보내기 실패",
  "settings.importFailed": "가져오기 실패",

  // Layout
  "layout.appName": "SSH Forwarder",
  "layout.loading": "로딩 중...",
  "layout.emptyState": "연결을 선택하거나 새로 추가하세요",
  "layout.noProfiles": "연결 프로파일이 없습니다",
  "layout.confirmDelete": "이 연결 프로파일을 삭제하시겠습니까?",
  "layout.unnamed": "이름 없음",

  // Forwarding direction hints
  "forward.localBind": "로컬 바인드",
  "forward.localBindHint": "앱 호스트에서 listen",
  "forward.serverBind": "서버 바인드",
  "forward.serverBindHint": "SSH 서버에서 listen",
  "forward.remoteTarget": "원격 대상",
  "forward.remoteTargetHint": "SSH 서버 기준",
  "forward.localTarget": "로컬 대상",
  "forward.localTargetHint": "앱 호스트 기준",
  "forward.swap": "바인드/대상 바꾸기",
  "forward.remove": "규칙 삭제",

  // Theme / Language
  "theme.title": "테마",
  "theme.light": "라이트",
  "theme.dark": "다크",
  "theme.system": "시스템",
  "language.title": "언어 / Language",

  // Tray (Rust-side, for reference)
  "tray.show": "창 열기",
  "tray.quit": "종료",

  // Misc
  "action.addConnection": "새 연결 추가",
  "error.unknown": "알 수 없는 오류",
} as const;
