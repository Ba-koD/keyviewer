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
        self.root.geometry("500x450")
        self.root.resizable(False, False)
        
        # 기본 설치 경로 설정
        self.default_install_path = self.get_default_install_path()
        self.install_path = tk.StringVar(value=self.default_install_path)
        
        # 옵션 변수들
        self.create_shortcut = tk.BooleanVar(value=True)
        self.create_desktop_shortcut = tk.BooleanVar(value=True)
        self.start_menu_shortcut = tk.BooleanVar(value=True)
        
        # GitHub 정보
        self.github_repo = "Ba-koD/keyviewer"  # 실제 GitHub 저장소로 변경 필요
        self.latest_version = None
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
        
        # 버전 정보 표시
        self.version_frame = ttk.LabelFrame(main_frame, text="버전 정보", padding="10")
        self.version_frame.pack(fill=tk.X, pady=(0, 15))
        
        self.version_label = ttk.Label(self.version_frame, text="버전 확인 중...")
        self.version_label.pack()
        
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
        
        # 설치 버튼
        self.install_btn = ttk.Button(main_frame, text="설치 시작", 
                                     command=self.start_installation, style="Accent.TButton")
        self.install_btn.pack(pady=20)
        
        # 진행 상황 표시
        self.progress = ttk.Progressbar(main_frame, mode='indeterminate')
        self.progress.pack(fill=tk.X, pady=(0, 10))
        
        self.status_label = ttk.Label(main_frame, text="설치를 시작하려면 '설치 시작' 버튼을 클릭하세요.")
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
    
    def check_for_updates(self):
        """자동으로 업데이트를 확인합니다."""
        try:
            # GitHub API에서 최신 릴리즈 정보 가져오기
            api_url = f"https://api.github.com/repos/{self.github_repo}/releases/latest"
            with urllib.request.urlopen(api_url) as response:
                release_info = json.loads(response.read().decode())
                
            self.latest_version = release_info['tag_name']
            
            # KBQV-v{version}.zip 파일 찾기
            for asset in release_info.get('assets', []):
                if asset['name'].startswith('KBQV-v') and asset['name'].endswith('.zip'):
                    self.download_url = asset['browser_download_url']
                    break
            
            if self.download_url:
                self.version_label.config(text=f"최신 버전: {self.latest_version} (다운로드 준비 완료)")
                self.install_btn.config(state='normal')
            else:
                self.version_label.config(text=f"최신 버전: {self.latest_version} (다운로드 링크를 찾을 수 없음)")
                self.install_btn.config(state='disabled')
                
        except Exception as e:
            self.version_label.config(text="버전 확인 실패 - 인터넷 연결을 확인하세요")
            self.install_btn.config(state='disabled')
            print(f"업데이트 확인 실패: {e}")
    
    def check_for_updates_manual(self):
        """수동으로 업데이트를 확인합니다."""
        self.update_btn.config(state='disabled')
        self.status_label.config(text="업데이트 확인 중...")
        
        try:
            self.check_for_updates()
            if self.download_url:
                self.status_label.config(text=f"최신 버전 {self.latest_version} 확인됨")
            else:
                self.status_label.config(text="다운로드 링크를 찾을 수 없습니다")
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
        if not self.download_url:
            messagebox.showerror("오류", "다운로드 링크를 찾을 수 없습니다.\n업데이트 확인 버튼을 클릭해주세요.")
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
        
        # 설치 시작
        self.install_btn.config(state='disabled')
        self.progress.start()
        self.status_label.config(text="설치 중...")
        
        # 별도 스레드에서 설치 실행
        install_thread = threading.Thread(target=self.perform_installation, args=(install_path,))
        install_thread.daemon = True
        install_thread.start()
    
    def perform_installation(self, install_path):
        """실제 설치를 수행합니다."""
        try:
            # 설치 경로 생성
            os.makedirs(install_path, exist_ok=True)
            
            # GitHub에서 메인 프로그램 다운로드
            self.update_status("GitHub에서 프로그램 다운로드 중...")
            
            # 임시 폴더에 다운로드
            temp_dir = tempfile.mkdtemp(prefix='keyviewer_install_')
            zip_path = os.path.join(temp_dir, f"KBQV-v{self.latest_version}.zip")
            
            # 파일 다운로드
            urllib.request.urlretrieve(self.download_url, zip_path)
            
            # 압축 해제
            self.update_status("프로그램 파일 압축 해제 중...")
            with zipfile.ZipFile(zip_path, 'r') as zip_ref:
                zip_ref.extractall(temp_dir)
            
            # 압축 해제된 폴더 찾기
            extracted_dir = None
            # GitHub tag 형식 그대로 사용 (v1.2.3)
            target_folder = f"KBQV-{self.latest_version}"
            
            for item in os.listdir(temp_dir):
                if item == target_folder:
                    extracted_dir = os.path.join(temp_dir, item)
                    break
            
            if not extracted_dir:
                raise Exception("압축 해제된 프로그램 폴더를 찾을 수 없습니다")
            
            # 프로그램 파일 복사
            self.update_status("프로그램 파일 복사 중...")
            
            # 기존 설치 폴더 내용 삭제
            if os.path.exists(install_path):
                for item in os.listdir(install_path):
                    item_path = os.path.join(install_path, item)
                    if os.path.isdir(item_path):
                        shutil.rmtree(item_path)
                    else:
                        os.remove(item_path)
            
            # 새로운 파일들 복사
            for item in os.listdir(extracted_dir):
                source = os.path.join(extracted_dir, item)
                destination = os.path.join(install_path, item)
                
                if os.path.isdir(source):
                    shutil.copytree(source, destination)
                else:
                    shutil.copy2(source, destination)
            
            # 임시 파일 정리
            shutil.rmtree(temp_dir)
            
            # 바로가기 생성
            if self.create_shortcut.get():
                self.update_status("바로가기 생성 중...")
                
                if self.create_desktop_shortcut.get():
                    self.create_shortcut_file(install_path, "바탕화면")
                
                if self.start_menu_shortcut.get():
                    self.create_shortcut_file(install_path, "시작메뉴")
            
            # 레지스트리 등록
            self.update_status("레지스트리 등록 중...")
            self.register_program(install_path)
            
            # 설치 완료
            self.root.after(0, self.installation_complete, install_path)
            
        except Exception as e:
            self.root.after(0, self.installation_failed, str(e))
    
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
                if item.endswith(".exe") and item.startswith("KBQV-v"):
                    target_exe = os.path.join(install_path, item)
                    break
            
            if not target_exe:
                # 기본 이름으로 시도 (KBQV-v{version}.exe 형식)
                target_exe = os.path.join(install_path, f"KBQV-v{self.latest_version}.exe")
            
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
                winreg.SetValueEx(key, "DisplayVersion", 0, winreg.REG_SZ, self.latest_version or "1.0.0")
                winreg.SetValueEx(key, "Publisher", 0, winreg.REG_SZ, "KeyQueueViewer")
                winreg.SetValueEx(key, "InstallLocation", 0, winreg.REG_SZ, install_path)
                winreg.SetValueEx(key, "UninstallString", 0, winreg.REG_SZ, 
                                os.path.join(install_path, "uninstall.exe"))
                # 메인 실행 파일 이름 찾기
                main_exe = None
                for item in os.listdir(install_path):
                    if item.endswith(".exe") and item.startswith("KBQV-v"):
                        main_exe = item
                        break
                
                if not main_exe:
                    main_exe = f"KBQV-v{self.latest_version}.exe"
                
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
설치된 버전: {self.latest_version}

프로그램을 실행하려면 설치된 폴더의 KBQV-v{self.latest_version}.exe를 실행하거나
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