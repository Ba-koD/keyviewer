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
        self.root.geometry("500x550")
        self.root.resizable(False, False)
        
        # 기본 설치 경로 설정
        self.default_install_path = self.get_default_install_path()
        self.install_path = tk.StringVar(value=self.default_install_path)
        
        # 옵션 변수들
        self.create_shortcut = tk.BooleanVar(value=True)
        self.create_desktop_shortcut = tk.BooleanVar(value=True)
        self.start_menu_shortcut = tk.BooleanVar(value=True)
        self.debug_mode = tk.BooleanVar(value=False)  # 디버그 모드 추가
        
        # GitHub 정보
        self.github_repo = "Ba-koD/keyviewer"  # 실제 GitHub 저장소로 변경 필요
        self.all_releases = []  # 모든 릴리즈 정보
        self.selected_version = None
        self.download_url = None
        
        self.setup_ui()
        
        # 시작 시 업데이트 확인
        self.check_for_updates()
        
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
        
        # 디버그 옵션 추가
        debug_frame = ttk.LabelFrame(main_frame, text="디버그 옵션", padding="10")
        debug_frame.pack(fill=tk.X, pady=(0, 15))
        
        debug_check = ttk.Checkbutton(debug_frame, text="디버그 모드 (콘솔 창 표시)", 
                                     variable=self.debug_mode, 
                                     command=self.toggle_debug_mode)
        debug_check.pack(anchor=tk.W)
        
        # 디버그 모드 설명
        debug_info = ttk.Label(debug_frame, text="설치 과정의 상세 로그를 콘솔 창에서 확인할 수 있습니다", 
                              font=("Arial", 8), foreground="gray")
        debug_info.pack(anchor=tk.W, pady=(5, 0))
        
        # 설치 버튼
        self.install_btn = ttk.Button(main_frame, text="설치 시작", 
                                     command=self.start_installation, style="Accent.TButton")
        self.install_btn.pack(pady=20)
        
        # 진행 상황 표시
        self.progress = ttk.Progressbar(main_frame, mode='indeterminate')
        self.progress.pack(fill=tk.X, pady=(0, 10))
        
        self.status_label = ttk.Label(main_frame, text="설치할 버전을 선택하고 '설치 시작' 버튼을 클릭하세요.")
        self.status_label.pack()
        
        # 스타일 설정
        style = ttk.Style()
        style.configure("Accent.TButton", font=("Arial", 10, "bold"))
        
        # 수동 업데이트 확인 버튼
        update_frame = ttk.Frame(main_frame)
        update_frame.pack(fill=tk.X, pady=(10, 0))
        self.update_btn = ttk.Button(update_frame, text="업데이트 확인", 
                                    command=self.check_for_updates_manual)
        self.update_btn.pack(side=tk.RIGHT)
    
    def toggle_debug_mode(self):
        """디버그 모드를 토글합니다."""
        if self.debug_mode.get():
            # 콘솔 창 표시
            if hasattr(sys, '_getframe'):
                # Windows에서 콘솔 창 표시
                try:
                    import ctypes
                    ctypes.windll.kernel32.AllocConsole()
                    sys.stdout = open('CONOUT$', 'w')
                    sys.stderr = open('CONOUT$', 'w')
                    print("=== 디버그 모드 활성화 ===")
                    print("설치 과정의 상세 로그가 이 창에 표시됩니다.")
                    print("=" * 40)
                except:
                    pass
        else:
            # 콘솔 창 숨기기 (설치 중에는 변경 불가)
            pass
    
    def check_for_updates(self):
        """자동으로 업데이트를 확인합니다."""
        try:
            # GitHub API에서 모든 릴리즈 정보 가져오기
            api_url = f"https://api.github.com/repos/{self.github_repo}/releases"
            with urllib.request.urlopen(api_url) as response:
                releases = json.loads(response.read().decode())
            
            # 릴리즈 정보 정리 (1.0.4 이상만)
            self.all_releases = []
            for release in releases:
                tag_name = release['tag_name']
                # v1.0.4 이상인지 확인
                if self.is_version_greater_or_equal(tag_name, "v1.0.4"):
                    # KBQV-v{version}.zip 파일 찾기
                    download_url = None
                    for asset in release.get('assets', []):
                        if asset['name'].startswith('KBQV-v') and asset['name'].endswith('.zip'):
                            download_url = asset['browser_download_url']
                            break
                    
                    if download_url:
                        self.all_releases.append({
                            'tag_name': tag_name,
                            'name': release.get('name', tag_name),
                            'download_url': download_url,
                            'published_at': release['published_at']
                        })
            
            # 최신 버전 순으로 정렬
            self.all_releases.sort(key=lambda x: x['tag_name'], reverse=True)
            
            # 콤보박스에 버전 목록 설정
            version_list = []
            for release in self.all_releases:
                if release == self.all_releases[0]:  # 최신 버전
                    version_list.append(f"{release['tag_name']} (최신버전)")
                else:
                    version_list.append(release['tag_name'])
            
            self.version_combo['values'] = version_list
            
            if self.all_releases:
                # 기본적으로 최신 버전 선택
                self.version_combo.set(version_list[0])
                self.on_version_selected()
                self.status_label.config(text=f"사용 가능한 버전: {len(self.all_releases)}개")
            else:
                self.version_info_label.config(text="사용 가능한 버전이 없습니다")
                self.install_btn.config(state='disabled')
                
        except Exception as e:
            self.version_info_label.config(text="버전 확인 실패 - 인터넷 연결을 확인하세요")
            self.install_btn.config(state='disabled')
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
        selected = self.version_var.get()
        if not selected:
            return
        
        # (최신버전) 텍스트 제거
        clean_version = selected.replace(" (최신버전)", "")
        
        # 선택된 릴리즈 찾기
        for release in self.all_releases:
            if release['tag_name'] == clean_version:
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
                break
    
    def check_for_updates_manual(self):
        """수동으로 업데이트를 확인합니다."""
        self.update_btn.config(state='disabled')
        self.status_label.config(text="업데이트 확인 중...")
        
        try:
            self.check_for_updates()
            if self.all_releases:
                self.status_label.config(text=f"사용 가능한 버전 {len(self.all_releases)}개 확인됨")
            else:
                self.status_label.config(text="사용 가능한 버전이 없습니다")
        except Exception as e:
            messagebox.showerror("업데이트 실패", f"업데이트 확인에 실패했습니다:\n{e}")
            self.status_label.config(text="업데이트 확인 실패")
        finally:
            self.update_btn.config(state='normal')
    
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
        if not self.selected_version or not self.download_url:
            messagebox.showerror("오류", "설치할 버전을 선택해주세요.")
            return
            
        install_path = self.install_path.get().strip()
        
        if not install_path:
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
        
        # 디버그 모드가 활성화되어 있으면 콘솔 창 표시
        if self.debug_mode.get():
            self.toggle_debug_mode()
        
        # 설치 시작
        self.install_btn.config(state='disabled')
        self.progress.start()
        self.status_label.config(text="설치 중...")
        
        # 별도 스레드에서 설치 실행
        install_thread = threading.Thread(target=self.perform_installation, args=(install_path,))
        install_thread.daemon = True
        install_thread.start()
    
    def perform_installation(self, install_path):
        """실제 설치 수행"""
        try:
            # 디버그: 시작 로그
            print(f"[DEBUG] Installation started for version: {self.selected_version['tag_name']}")
            print(f"[DEBUG] Download URL: {self.download_url}")
            print(f"[DEBUG] Install path: {install_path}")
            print(f"[DEBUG] Debug mode: {self.debug_mode.get()}")
            
            # 임시 디렉토리 생성
            temp_dir = tempfile.mkdtemp(prefix='keyviewer_install_')
            print(f"[DEBUG] Temporary directory created: {temp_dir}")
            
            # ZIP 파일 다운로드
            zip_path = os.path.join(temp_dir, f"KBQV-{self.selected_version['tag_name']}.zip")
            print(f"[DEBUG] Downloading ZIP to: {zip_path}")
            
            urllib.request.urlretrieve(self.download_url, zip_path)
            print(f"[DEBUG] ZIP download completed. File size: {os.path.getsize(zip_path)} bytes")
            
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
            messagebox.showerror("설치 실패", f"설치 중 오류가 발생했습니다:\n{str(e)}")
    
    def update_status(self, message):
        """상태 메시지를 업데이트합니다."""
        self.root.after(0, lambda: self.status_label.config(text=message))
        time.sleep(0.5)  # 사용자가 메시지를 볼 수 있도록 잠시 대기
    
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
        self.progress.stop()
        self.install_btn.config(state='normal')
        
        # 완료 메시지
        message = f"""설치가 완료되었습니다!

설치 경로: {install_path}
설치된 버전: {self.selected_version['tag_name']}

프로그램을 실행하려면 설치된 폴더의 KBQV-{self.selected_version['tag_name']}.exe를 실행하거나
생성된 바로가기를 사용하세요."""

        messagebox.showinfo("설치 완료", message)
        
        # 설치된 폴더 열기 옵션
        if messagebox.askyesno("폴더 열기", "설치된 폴더를 열까요?"):
            try:
                os.startfile(install_path)
            except:
                subprocess.run(["explorer", install_path])
        
        # 프로그램 종료
        self.root.quit()
    
    def installation_failed(self, error_message):
        """설치 실패 처리를 합니다."""
        self.progress.stop()
        self.install_btn.config(state='normal')
        self.status_label.config(text="설치 실패")
        
        messagebox.showerror("설치 실패", 
                           f"설치 중 오류가 발생했습니다:\n\n{error_message}\n\n"
                           "관리자 권한으로 실행하거나 다른 경로를 선택해보세요.")
    
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