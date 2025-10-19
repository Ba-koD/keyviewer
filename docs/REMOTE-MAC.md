## macOS 원격 빌드/테스트 빠른 가이드

사전 준비(최초 1회, 맥에서)
- 시스템 설정 > 공유 > 원격 로그인 활성화
- Xcode 명령줄 도구 설치: `xcode-select --install` (필요 시)

윈도우/WSL에서 키 교환(권장)
```bash
ssh-keygen -t ed25519 -C "keyviewer-remote"
ssh-copy-id -p 2222 rudgh@ssh.rnen.kr
# 또는 수동으로 ~/.ssh/id_ed25519.pub 내용을 맥의 ~/.ssh/authorized_keys 에 추가
ssh-keyscan -p 2222 ssh.rnen.kr >> ~/.ssh/known_hosts
```

원격 셋업(툴체인 설치, 맥에 1회)
```bash
scripts/mac-remote.sh setup
```

프로젝트 동기화
```bash
scripts/mac-remote.sh sync
```

개발 실행(로그 확인용)
```bash
scripts/mac-remote.sh dev
```

패키징 빌드(App, DMG)
```bash
scripts/mac-remote.sh build
```

환경 변수 또는 .env로 접속 정보/경로 관리 가능
```bash
MAC_HOST=ssh.rnen.kr MAC_PORT=2222 MAC_USER=rudgh \
MAC_DIR=/Users/rudgh/work/keyviewer scripts/mac-remote.sh sync
```

.env 사용 예시
```bash
cp .env.example .env
# 편집 후 사용
scripts/mac-remote.sh sync
```

비밀번호 사용(sshpass)
- 가능한 SSH 키 사용을 권장합니다. 부득이하게 비밀번호가 필요하면 `.env`에 `MAC_PASS` 를 넣고 로컬에 `sshpass` 를 설치하세요.
- Bash/WSL: `sudo apt install sshpass` 또는 `brew install hudochenkov/sshpass/sshpass`

PowerShell 사용 시
```powershell
./scripts/mac-remote.ps1 -Action setup
./scripts/mac-remote.ps1 -Action sync
./scripts/mac-remote.ps1 -Action dev
./scripts/mac-remote.ps1 -Action build
```

참고
- 빌드 캐시를 위해 맥의 `~/.cargo`, 프로젝트 `target` 폴더는 유지됩니다.
- UI 확인이 필요하면 `dev` 실행 후 화면공유(스크린 공유)로 확인하세요.


