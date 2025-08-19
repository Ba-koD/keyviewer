import os
import sys
import shutil
import winreg
import ctypes
import tkinter as tk
from tkinter import ttk, messagebox, filedialog
from pathlib import Path
import subprocess
import threading
import time
import urllib.request
import json
import zipfile
import tempfile

class InstallerApp:
    def __init__(self):
        self.root = tk.Tk()
        self.root.title("KeyQueueViewer 자동설치기")
        self.root.geometry("500x600")
        self.root.resizable(False, False)
        
        # 아이콘 설정
        try:
            icon_path = os.path.join(os.path.dirname(__file__), "web", "favicon.ico")
            if os.path.exists(icon_path):
                self.root.iconbitmap(icon_path)
        except:
            pass  # 아이콘 설정 실패 시 무시
        
        # 기본 설치 경로 설정
        self.default_install_path = self.get_default_install_path()
        self.install_path = tk.StringVar(value=self.default_install_path)
        
        # 옵션 변수들
        self.create_shortcut = tk.BooleanVar(value=True)
        self.create_desktop_shortcut = tk.BooleanVar(value=True)
        self.start_menu_shortcut = tk.BooleanVar(value=True)
        
        # GitHub 정보
        self.github_repo = "Ba-koD/keyviewer"  # 실제 GitHub 저장소로 변경 필요
        self.all_releases = []  # 모든 릴리즈 정보
        self.selected_version = None
        self.download_url = None
        
        self.setup_ui()
        
        # UI 설정 완료 후 업데이트 확인
        self.root.after(100, self.check_for_updates)
        
    def get_default_install_path(self):
        """기본 설치 경로를 반환합니다."""
        try:
            # Program Files 경로 확인
            program_files = os.environ.get('PROGRAMFILES', 'C:\\Program Files')
            return os.path.join(program_files, 'KeyQueueViewer')
        except:
            return 'C:\\Program Files\\KeyQueueViewer'
    
    def setup_ui(self):
        """UI를 설정합니다."""
        # 메인 프레임
        main_frame = ttk.Frame(self.root, padding="20")
        main_frame.pack(fill=tk.BOTH, expand=True)
        
        # 제목
        title_label = ttk.Label(main_frame, text="KeyQueueViewer 자동설치기", 
                               font=("Arial", 16, "bold"))
        title_label.pack(pady=(0, 20))
        
        # 버전 선택
        version_frame = ttk.LabelFrame(main_frame, text="설치할 버전 선택", padding="10")
        version_frame.pack(fill=tk.X, pady=(0, 15))
        
        # 버전 선택 콤보박스
        self.version_var = tk.StringVar()
        self.version_combo = ttk.Combobox(version_frame, textvariable=self.version_var, 
                                         state="readonly", width=30)
        self.version_combo.pack(pady=(0, 5))
        self.version_combo.bind('<<ComboboxSelected>>', self.on_version_selected)
        
        # 버전 정보 표시
        self.version_info_label = ttk.Label(version_frame, text="버전을 선택해주세요")
        self.version_info_label.pack()
        
        # 상태 표시 라벨 추가
        self.status_label = ttk.Label(version_frame, text="", foreground="blue")
        self.status_label.pack(pady=(5, 0))
        
        # 설치 경로 선택
        path_frame = ttk.LabelFrame(main_frame, text="설치 경로", padding="10")
        path_frame.pack(fill=tk.X, pady=(0, 15))
        
        path_entry_frame = ttk.Frame(path_frame)
        path_entry_frame.pack(fill=tk.X)
        
        self.path_entry = ttk.Entry(path_entry_frame, textvariable=self.install_path, width=50)
        self.path_entry.pack(side=tk.LEFT, fill=tk.X, expand=True, padx=(0, 10))
        
        browse_btn = ttk.Button(path_entry_frame, text="찾아보기", command=self.browse_path)
        browse_btn.pack(side=tk.RIGHT)
        
        # 바로가기 옵션
        shortcut_frame = ttk.LabelFrame(main_frame, text="바로가기 옵션", padding="10")
        shortcut_frame.pack(fill=tk.X, pady=(0, 15))
        
        ttk.Checkbutton(shortcut_frame, text="바로가기 생성", 
                       variable=self.create_shortcut).pack(anchor=tk.W)
        ttk.Checkbutton(shortcut_frame, text="바탕화면 바로가기", 
                       variable=self.create_desktop_shortcut).pack(anchor=tk.W)
        ttk.Checkbutton(shortcut_frame, text="시작 메뉴 바로가기", 
                       variable=self.start_menu_shortcut).pack(anchor=tk.W)
        
        # 다운로드 및 설치 진행상황 표시
        progress_frame = ttk.LabelFrame(main_frame, text="진행상황", padding="10")
        progress_frame.pack(fill=tk.X, pady=(0, 15))
        
        # 다운로드 진행상황
        self.download_progress_var = tk.StringVar(value="다운로드 대기 중")
        self.download_progress_label = ttk.Label(progress_frame, textvariable=self.download_progress_var)
        self.download_progress_label.pack(pady=(0, 5))
        
        self.download_progress_bar = ttk.Progressbar(progress_frame, mode='determinate', length=400)
        self.download_progress_bar.pack(fill=tk.X)
        
        self.download_speed_label = ttk.Label(progress_frame, text="", font=("Arial", 8), foreground="gray")
        self.download_speed_label.pack(pady=(5, 0))
        
        # 설치 진행상황 (초기에는 숨김)
        self.install_progress_var = tk.StringVar(value="설치 대기 중")
        self.install_progress_label = ttk.Label(progress_frame, textvariable=self.install_progress_var)
        self.install_progress_bar = ttk.Progressbar(progress_frame, mode='determinate', length=400)
        
        # 설치 버튼
        self.install_btn = ttk.Button(main_frame, text="설치 시작", 
                                     command=self.start_installation, style="Accent.TButton")
        self.install_btn.pack(pady=20)
    
    def check_for_updates(self):
        """자동으로 업데이트를 확인합니다."""
        try:
            print("[DEBUG] check_for_updates 시작")
            # GitHub API에서 모든 릴리즈 정보 가져오기
            api_url = f"https://api.github.com/repos/{self.github_repo}/releases"
            print(f"[DEBUG] API URL: {api_url}")
            
            with urllib.request.urlopen(api_url) as response:
                releases = json.loads(response.read().decode())
            print(f"[DEBUG] API 응답 받음: {len(releases)}개 릴리즈")
            
            self.all_releases = []
            for release in releases:
                tag_name = release['tag_name']
                print(f"[DEBUG] 릴리즈 처리 중: {tag_name}")
                # KBQV{globing}.zip 파일 찾기
                download_url = None
                for asset in release.get('assets', []):
                    if asset['name'].startswith('KBQV') and asset['name'].endswith('.zip'):
                        download_url = asset['browser_download_url']
                        print(f"[DEBUG] 다운로드 URL 찾음: {download_url}")
                        break
                
                if download_url:
                    self.all_releases.append({
                        'tag_name': tag_name,
                        'name': release.get('name', tag_name),
                        'download_url': download_url,
                        'published_at': release['published_at']
                    })
                    print(f"[DEBUG] 릴리즈 추가됨: {tag_name}")
            
            print(f"[DEBUG] 총 {len(self.all_releases)}개 릴리즈 수집됨")
            
            # 최신 버전 순으로 정렬
            self.all_releases.sort(key=lambda x: x['tag_name'], reverse=True)
            
            # 콤보박스에 버전 목록 설정
            version_list = []
            for release in self.all_releases:
                if release == self.all_releases[0]:  # 최신 버전
                    version_list.append(f"{release['tag_name']} (최신버전)")
                else:
                    version_list.append(release['tag_name'])
            
            print(f"[DEBUG] 버전 목록: {version_list}")
            self.version_combo['values'] = version_list
            
            if self.all_releases:
                # 기본적으로 최신 버전 선택
                self.version_combo.set(version_list[0])
                self.on_version_selected()
                if hasattr(self, 'status_label'):
                    self.status_label.config(text=f"사용 가능한 버전: {len(self.all_releases)}개")
                print(f"[DEBUG] 상태 라벨 업데이트 완료: {len(self.all_releases)}개 버전")
            else:
                self.version_info_label.config(text="사용 가능한 버전이 없습니다")
                self.install_btn.config(state='disabled')
                print("[DEBUG] 사용 가능한 버전 없음")
                
        except Exception as e:
            print(f"[DEBUG] 예외 발생: {type(e).__name__}: {e}")
            import traceback
            traceback.print_exc()
            self.version_info_label.config(text="버전 확인 실패 - 인터넷 연결을 확인하세요")
            self.install_btn.config(state='disabled')
            if hasattr(self, 'status_label'):
                self.status_label.config(text="버전 확인 실패")
            print(f"업데이트 확인 실패: {e}")
    
    def is_version_greater_or_equal(self, version1, version2):
        """버전 비교 함수"""
        try:
            # v 제거하고 숫자만 추출
            v1 = version1.lstrip('v').split('.')
            v2 = version2.lstrip('v').split('.')
            
            # 각 부분을 정수로 변환하여 비교
            for i in range(max(len(v1), len(v2))):
                num1 = int(v1[i]) if i < len(v1) else 0
                num2 = int(v2[i]) if i < len(v2) else 0
                if num1 > num2:
                    return True
                elif num1 < num2:
                    return False
            return True  # 같음
        except:
            return False
    
    def on_version_selected(self, event=None):
        """버전이 선택되었을 때 호출됩니다."""
        print(f"[DEBUG] on_version_selected 호출됨, event: {event}")
        selected = self.version_var.get()
        print(f"[DEBUG] 선택된 버전: {selected}")
        if not selected:
            print("[DEBUG] 선택된 버전이 없음")
            return
        
        # (최신버전) 텍스트 제거
        clean_version = selected.replace(" (최신버전)", "")
        print(f"[DEBUG] 정리된 버전: {clean_version}")
        
        # 선택된 릴리즈 찾기
        for release in self.all_releases:
            if release['tag_name'] == clean_version:
                print(f"[DEBUG] 릴리즈 찾음: {release}")
                self.selected_version = release
                self.download_url = release['download_url']
                
                # 버전 정보 업데이트
                if release == self.all_releases[0]:  # 최신 버전
                    self.version_info_label.config(text=f"선택된 버전: {clean_version} (최신버전)")
                else:
                    self.version_info_label.config(text=f"선택된 버전: {clean_version}")
                
                # 설치 버튼 활성화
                self.install_btn.config(state='normal')
                self.status_label.config(text=f"'{clean_version}' 버전 설치 준비 완료")
                print(f"[DEBUG] 버전 선택 완료: {clean_version}")
                break
        else:
            print(f"[DEBUG] 릴리즈를 찾을 수 없음: {clean_version}")
    
    def browse_path(self):
        """설치 경로를 선택하는 대화상자를 엽니다."""
        path = filedialog.askdirectory(
            title="설치 폴더 선택",
            initialdir=self.install_path.get()
        )
        if path:
            self.install_path.set(path)
    
    def start_installation(self):
        """설치를 시작합니다."""
        print("[DEBUG] start_installation 호출됨")
        if not self.selected_version or not self.download_url:
            print("[DEBUG] 설치할 버전이 선택되지 않음")
            messagebox.showerror("오류", "설치할 버전을 선택해주세요.")
            return
            
        install_path = self.install_path.get().strip()
        print(f"[DEBUG] 설치 경로: {install_path}")
        
        if not install_path:
            print("[DEBUG] 설치 경로가 입력되지 않음")
            messagebox.showerror("오류", "설치 경로를 입력해주세요.")
            return
        
        # 설치 경로 유효성 검사
        try:
            install_path = os.path.abspath(install_path)
            if os.path.exists(install_path):
                if not os.access(install_path, os.W_OK):
                    messagebox.showerror("오류", "선택한 폴더에 쓰기 권한이 없습니다.\n다른 경로를 선택하거나 관리자 권한으로 실행해주세요.")
                    return
        except Exception as e:
            messagebox.showerror("오류", f"설치 경로가 유효하지 않습니다: {e}")
            return
        
        # 설치 시작
        print("[DEBUG] 설치 시작 - UI 업데이트")
        self.install_btn.config(state='disabled')
        self.status_label.config(text="설치 중...")
        
        # 별도 스레드에서 설치 실행
        print("[DEBUG] 설치 스레드 시작")
        install_thread = threading.Thread(target=self.perform_installation, args=(install_path,))
        install_thread.daemon = True
        install_thread.start()
        print("[DEBUG] 설치 스레드 시작됨")
    
    def perform_installation(self, install_path):
        """실제 설치 수행"""
        try:
            # 디버그: 시작 로그
            print(f"[DEBUG] Installation started for version: {self.selected_version['tag_name']}")
            print(f"[DEBUG] Download URL: {self.download_url}")
            print(f"[DEBUG] Install path: {install_path}")
            
            # 임시 디렉토리 생성
            temp_dir = tempfile.mkdtemp(prefix='keyviewer_install_')
            print(f"[DEBUG] Temporary directory created: {temp_dir}")
            
            # ZIP 파일 다운로드
            zip_path = os.path.join(temp_dir, f"KBQV-{self.selected_version['tag_name']}.zip")
            print(f"[DEBUG] Downloading ZIP to: {zip_path}")
            
            # 다운로드 진행상황 초기화
            self.reset_download_progress()
            
            # 파일 크기 확인
            try:
                response = urllib.request.urlopen(self.download_url)
                total_size = int(response.headers.get('content-length', 0))
                response.close()
            except:
                total_size = 0
            
            # 진행상황 콜백을 사용한 다운로드
            def download_progress_hook(count, block_size, total_size):
                downloaded = count * block_size
                self.update_download_progress(downloaded, total_size)
                self.root.update_idletasks()
            
            urllib.request.urlretrieve(self.download_url, zip_path, download_progress_hook)
            print(f"[DEBUG] ZIP download completed. File size: {os.path.getsize(zip_path)} bytes")
            
            # 다운로드 완료
            self.download_progress_var.set("다운로드 완료!")
            self.download_progress_bar['value'] = 100
            
            # ZIP 파일 내용 확인
            print(f"[DEBUG] ZIP file contents:")
            with zipfile.ZipFile(zip_path, 'r') as zip_ref:
                zip_ref.printdir()
            
            # ZIP 파일 압축 해제
            print(f"[DEBUG] Extracting ZIP file...")
            with zipfile.ZipFile(zip_path, 'r') as zip_ref:
                zip_ref.extractall(temp_dir)
            
            # 압축 해제된 내용 확인
            print(f"[DEBUG] Extracted contents:")
            for root, dirs, files in os.walk(temp_dir):
                level = root.replace(temp_dir, '').count(os.sep)
                indent = ' ' * 2 * level
                print(f"{indent}{os.path.basename(root)}/")
                subindent = ' ' * 2 * (level + 1)
                for file in files:
                    print(f"{subindent}{file}")
            
            # 압축 해제된 폴더 찾기
            target_folder = f"KBQV-{self.selected_version['tag_name']}"
            print(f"[DEBUG] Looking for target folder: {target_folder}")
            
            extracted_dir = None
            for item in os.listdir(temp_dir):
                item_path = os.path.join(temp_dir, item)
                if os.path.isdir(item_path) and item == target_folder:
                    extracted_dir = item_path
                    print(f"[DEBUG] Found target folder: {extracted_dir}")
                    break
            
            if not extracted_dir:
                print(f"[DEBUG] Target folder not found. Available items:")
                for item in os.listdir(temp_dir):
                    item_path = os.path.join(temp_dir, item)
                    if os.path.isdir(item_path):
                        print(f"[DEBUG]   Directory: {item} (contains {len(os.listdir(item_path))} items)")
                    else:
                        print(f"[DEBUG]   File: {item}")
                raise Exception(f"Target folder '{target_folder}' not found in extracted contents")
            
            # 설치 경로에 파일 복사
            print(f"[DEBUG] Copying files from {extracted_dir} to {install_path}")
            
            # 기존 파일 삭제
            if os.path.exists(install_path):
                print(f"[DEBUG] Removing existing installation directory")
                shutil.rmtree(install_path)
            
            # 새로 생성
            os.makedirs(install_path, exist_ok=True)
            print(f"[DEBUG] Created installation directory: {install_path}")
            
            # 파일 복사
            copied_files = 0
            for item in os.listdir(extracted_dir):
                src = os.path.join(extracted_dir, item)
                dst = os.path.join(install_path, item)
                if os.path.isdir(src):
                    shutil.copytree(src, dst)
                    print(f"[DEBUG] Copied directory: {item}")
                else:
                    shutil.copy2(src, dst)
                    print(f"[DEBUG] Copied file: {item}")
                copied_files += 1
            
            print(f"[DEBUG] Total items copied: {copied_files}")
            
            # 설치된 파일 확인
            print(f"[DEBUG] Installed files:")
            for root, dirs, files in os.walk(install_path):
                level = root.replace(install_path, '').count(os.sep)
                indent = ' ' * 2 * level
                print(f"{indent}{os.path.basename(root)}/")
                subindent = ' ' * 2 * (level + 1)
                for file in files:
                    print(f"{subindent}{file}")
            
            # 임시 디렉토리 정리
            print(f"[DEBUG] Cleaning up temporary directory: {temp_dir}")
            shutil.rmtree(temp_dir)
            
            # 레지스트리 등록
            print(f"[DEBUG] Registering program in registry")
            self.register_program(install_path)
            
            # 바로가기 생성
            if self.create_shortcut.get():
                print(f"[DEBUG] Creating shortcuts")
                self.create_shortcut_file(install_path, "바탕화면")
                self.create_shortcut_file(install_path, "시작메뉴")
            
            print(f"[DEBUG] Installation completed successfully")
            self.installation_complete(install_path)
            
        except Exception as e:
            print(f"[DEBUG] Installation failed with error: {str(e)}")
            print(f"[DEBUG] Error type: {type(e).__name__}")
            import traceback
            traceback.print_exc()
            self.installation_failed(str(e))
    
    def update_status(self, message):
        """상태 메시지를 업데이트합니다."""
        self.root.after(0, lambda: self.status_label.config(text=message))
        time.sleep(0.5)  # 사용자가 메시지를 볼 수 있도록 잠시 대기
    
    def update_download_progress(self, downloaded_bytes, total_bytes, speed=None):
        """다운로드 진행상황을 업데이트합니다."""
        if total_bytes > 0:
            progress = (downloaded_bytes / total_bytes) * 100
            self.download_progress_bar['value'] = progress
            
            # 진행률 텍스트 업데이트
            downloaded_mb = downloaded_bytes / (1024 * 1024)
            total_mb = total_bytes / (1024 * 1024)
            self.download_progress_var.set(f"다운로드 중... {downloaded_mb:.1f}MB / {total_mb:.1f}MB ({progress:.1f}%)")
            
            # 속도 표시
            if speed:
                speed_mb = speed / (1024 * 1024)
                self.download_speed_label.config(text=f"다운로드 속도: {speed_mb:.1f} MB/s")
        else:
            self.download_progress_var.set("다운로드 중...")
    
    def reset_download_progress(self):
        """다운로드 진행상황을 초기화합니다."""
        self.download_progress_bar['value'] = 0
        self.download_progress_var.set("다운로드 대기 중")
        self.download_speed_label.config(text="")
    
    def create_shortcut_file(self, install_path, shortcut_type):
        """바로가기 파일을 생성합니다."""
        try:
            if shortcut_type == "바탕화면":
                # 바탕화면 경로
                desktop = os.path.join(os.path.expanduser("~"), "Desktop")
                shortcut_path = os.path.join(desktop, "KeyQueueViewer.lnk")
            else:  # 시작메뉴
                # 시작 메뉴 경로
                start_menu = os.path.join(os.environ.get('APPDATA', ''), 
                                        'Microsoft', 'Windows', 'Start Menu', 'Programs')
                os.makedirs(start_menu, exist_ok=True)
                shortcut_path = os.path.join(start_menu, "KeyQueueViewer.lnk")
            
            # 바로가기 생성 (PowerShell 사용)
            # 메인 실행 파일 이름 찾기 (KBQV-v{version}.exe)
            target_exe = None
            for item in os.listdir(install_path):
                if item.endswith(".exe") and item.startswith("KBQV-"):
                    target_exe = os.path.join(install_path, item)
                    break
            
            if not target_exe:
                # 기본 이름으로 시도 (KBQV-{version}.exe 형식)
                target_exe = os.path.join(install_path, f"KBQV-{self.selected_version['tag_name']}.exe")
            
            if os.path.exists(target_exe):
                ps_script = f'''
                $WshShell = New-Object -comObject WScript.Shell
                $Shortcut = $WshShell.CreateShortcut("{shortcut_path}")
                $Shortcut.TargetPath = "{target_exe}"
                $Shortcut.WorkingDirectory = "{install_path}"
                $Shortcut.Description = "KeyQueueViewer - 키 입력 모니터링 도구"
                $Shortcut.Save()
                '''
                
                subprocess.run(["powershell", "-Command", ps_script], 
                             capture_output=True, text=True, check=True)
                
        except Exception as e:
            print(f"바로가기 생성 실패 ({shortcut_type}): {e}")
    
    def register_program(self, install_path):
        """프로그램을 레지스트리에 등록합니다."""
        try:
            # 프로그램 등록 정보
            program_key = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\KeyQueueViewer"
            
            with winreg.CreateKey(winreg.HKEY_LOCAL_MACHINE, program_key) as key:
                winreg.SetValueEx(key, "DisplayName", 0, winreg.REG_SZ, "KeyQueueViewer")
                winreg.SetValueEx(key, "DisplayVersion", 0, winreg.REG_SZ, self.selected_version['tag_name'])
                winreg.SetValueEx(key, "Publisher", 0, winreg.REG_SZ, "KeyQueueViewer")
                winreg.SetValueEx(key, "InstallLocation", 0, winreg.REG_SZ, install_path)
                winreg.SetValueEx(key, "UninstallString", 0, winreg.REG_SZ, 
                                os.path.join(install_path, "uninstall.exe"))
                # 메인 실행 파일 이름 찾기
                main_exe = None
                for item in os.listdir(install_path):
                    if item.endswith(".exe") and item.startswith("KBQV-"):
                        main_exe = item
                        break
                
                if not main_exe:
                    main_exe = f"KBQV-{self.selected_version['tag_name']}.exe"
                
                winreg.SetValueEx(key, "DisplayIcon", 0, winreg.REG_SZ, 
                                os.path.join(install_path, main_exe))
                winreg.SetValueEx(key, "EstimatedSize", 0, winreg.REG_DWORD, 10240)
                winreg.SetValueEx(key, "NoModify", 0, winreg.REG_DWORD, 1)
                winreg.SetValueEx(key, "NoRepair", 0, winreg.REG_DWORD, 1)
                
        except Exception as e:
            print(f"레지스트리 등록 실패: {e}")
    
    def installation_complete(self, install_path):
        """설치 완료 처리를 합니다."""
        print("[DEBUG] installation_complete 호출됨")
        self.root.after(0, lambda: self.install_btn.config(state='normal'))
        self.root.after(0, lambda: self.status_label.config(text="설치 완료!"))
        
        # 완료 메시지
        message = f"""설치가 완료되었습니다!

설치 경로: {install_path}
설치된 버전: {self.selected_version['tag_name']}

프로그램을 실행하려면 설치된 폴더의 KBQV-{self.selected_version['tag_name']}.exe를 실행하거나
생성된 바로가기를 사용하세요."""

        self.root.after(0, lambda: messagebox.showinfo("설치 완료", message))
        
        # 설치된 폴더 열기 옵션
        def ask_open_folder():
            if messagebox.askyesno("폴더 열기", "설치된 폴더를 열까요?"):
                try:
                    os.startfile(install_path)
                except:
                    subprocess.run(["explorer", install_path])
            # 프로그램 종료
            self.root.after(1000, self.root.quit)
        
        self.root.after(100, ask_open_folder)
    
    def installation_failed(self, error_message):
        """설치 실패 처리를 합니다."""
        print(f"[DEBUG] installation_failed 호출됨: {error_message}")
        self.root.after(0, lambda: self.install_btn.config(state='normal'))
        self.root.after(0, lambda: self.status_label.config(text="설치 실패"))
        
        self.root.after(0, lambda: messagebox.showerror("설치 실패", 
                           f"설치 중 오류가 발생했습니다:\n\n{error_message}\n\n"
                           "관리자 권한으로 실행하거나 다른 경로를 선택해보세요."))
    
    def run(self):
        """애플리케이션을 실행합니다."""
        self.root.mainloop()

def main():
    # 관리자 권한 확인
    if not ctypes.windll.shell32.IsUserAnAdmin():
        # 관리자 권한으로 재실행
        ctypes.windll.shell32.ShellExecuteW(None, "runas", sys.executable, 
                                           " ".join(sys.argv), None, 1)
        return
    
    app = InstallerApp()
    app.run()

if __name__ == "__main__":
    main() 