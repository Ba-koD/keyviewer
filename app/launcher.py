import threading
import time
import webbrowser
import socket
import sys
import os
import json
from pathlib import Path
from typing import Optional

import tkinter as tk
from tkinter import ttk, messagebox

import winreg

import uvicorn
import ctypes
import logging

# Ensure taskbar pin uses our app identity (affects taskbar icon)
try:
	ctypes.windll.shell32.SetCurrentProcessExplicitAppUserModelID("KeyQueueViewer")
except Exception:
	pass

# Language Manager
class LanguageManager:
	LANGUAGES = {
		"en": "English",
		"ko": "한국어"
	}
	DEFAULT_LANGUAGE = "en"
	
	@classmethod
	def get_text(cls, language: str, key: str) -> str:
		texts = {
			"en": {
				"title": "Key Queue Viewer",
				"start_server": "Start Server",
				"stop_server": "Stop Server",
				"toggle_console": "Toggle Console",
				"toggle_startup": "Start with Windows",
				"web_control": "Web Control",
				"overlay": "Overlay",
				"language": "Language",
				"port": "Port",
				"save_port": "Save Port",
				"tips": "Tips",
				"tips_content": "1. Start the server first\n2. Open Web Control to configure target windows\n3. Open Overlay in OBS Browser Source\n4. Set URL to: http://localhost:PORT/overlay",
				"server_running": "Server is running",
				"server_stopped": "Server is stopped",
				"port_saved": "Port saved successfully",
				"port_invalid": "Port must be between 1000-65535",
				"console_allocated": "Console allocated",
				"console_freed": "Console freed",
				"console_failed": "Failed to allocate console",
				"admin_required": "Administrator Required",
				"admin_message": "Administrator privileges are required for global key hooking.\nWould you like to re-run as administrator?",
				"already_running": "Already running",
				"port_in_use": "Port {port} is already in use. Another instance may be running. Opening control page.",
				"error": "Error",
				"start_failed": "Failed to start server: {error}",
				"stop_failed": "Failed to stop server: {error}",
				"startup_failed": "Failed to update startup option: {error}",
				"console": "Console",
				"console_visibility_info": "Console visibility is applied immediately, but detailed logs require server restart."
			},
			"ko": {
				"title": "키 큐 뷰어",
				"start_server": "서버 시작",
				"stop_server": "서버 중지",
				"toggle_console": "콘솔 토글",
				"toggle_startup": "Windows 시작 시 실행",
				"web_control": "웹 제어",
				"overlay": "오버레이",
				"language": "언어",
				"port": "포트",
				"save_port": "포트 저장",
				"tips": "사용법",
				"tips_content": "1. 먼저 서버를 시작하세요\n2. 웹 제어를 열어 대상 창을 설정하세요\n3. OBS 브라우저 소스에서 오버레이를 여세요\n4. URL을 다음으로 설정: http://localhost:포트/overlay",
				"server_running": "서버가 실행 중입니다",
				"server_stopped": "서버가 중지되었습니다",
				"port_saved": "포트가 성공적으로 저장되었습니다",
				"port_invalid": "포트는 1000-65535 범위여야 합니다",
				"console_allocated": "콘솔이 할당되었습니다",
				"console_freed": "콘솔이 해제되었습니다",
				"console_failed": "콘솔 할당에 실패했습니다",
				"admin_required": "관리자 필요",
				"admin_message": "전역 키 후킹을 위해 관리자 권한이 필요할 수 있습니다.\n관리자 권한으로 다시 실행할까요?",
				"already_running": "이미 실행 중",
				"port_in_use": "포트 {port}가 이미 사용 중입니다. 다른 인스턴스가 실행 중일 수 있습니다. 컨트롤 페이지를 엽니다.",
				"error": "오류",
				"start_failed": "서버 시작에 실패했습니다: {error}",
				"stop_failed": "서버 중지에 실패했습니다: {error}",
				"startup_failed": "시작 옵션 업데이트에 실패했습니다: {error}",
				"console": "콘솔",
				"console_visibility_info": "콘솔 가시성은 즉시 적용되지만, 자세한 로그는 서버 재시작이 필요합니다."
			}
		}
		return texts.get(language, texts["en"]).get(key, key)
	
	@classmethod
	def get_language_names(cls) -> list:
		return list(cls.LANGUAGES.values())
	
	@classmethod
	def get_language_code(cls, language_name: str) -> str:
		for code, name in cls.LANGUAGES.items():
			if name == language_name:
				return code
		return cls.DEFAULT_LANGUAGE

# Configuration management
def _config_dir() -> Path:
	appdata = os.getenv("APPDATA")
	if not appdata:
		return Path.home() / ".keyqueueviewer"
	return Path(appdata) / "KeyQueueViewer"

def _config_path() -> Path:
	cfg_dir = _config_dir()
	cfg_dir.mkdir(parents=True, exist_ok=True)
	return cfg_dir / "launcher_config.json"

def load_launcher_config() -> dict:
	config_path = _config_path()
	if config_path.exists():
		try:
			with open(config_path, 'r', encoding='utf-8') as f:
				return json.load(f)
		except Exception:
			pass
	return {"language": LanguageManager.DEFAULT_LANGUAGE, "port": 8000}

def save_launcher_config(config: dict) -> None:
	config_path = _config_path()
	try:
		with open(config_path, 'w', encoding='utf-8') as f:
			json.dump(config, f, ensure_ascii=False, indent=2)
	except Exception:
		pass

# Import FastAPI app and config helpers
try:
	from server.main import app as fastapi_app
	from server.main import load_app_config, save_app_config
except Exception as e:
	fastapi_app = None
	_import_error = e
	load_app_config = None
	save_app_config = None
else:
	_import_error = None


def is_user_admin() -> bool:
	try:
		return bool(ctypes.windll.shell32.IsUserAnAdmin())
	except Exception:
		return False


def relaunch_as_admin() -> None:
	try:
		params = ' '
		if len(sys.argv) > 1:
			params = ' '.join(sys.argv[1:])
		ctypes.windll.shell32.ShellExecuteW(None, "runas", sys.executable, params, None, 1)
		os._exit(0)
	except Exception as e:
		messagebox.showerror("Admin", f"관리자 권한 실행에 실패했습니다:\n{e}")


def _alloc_console() -> None:
	"""Allocate console for output if not already allocated."""
	try:
		# Check if console is already allocated
		if hasattr(sys.stdout, 'name') and sys.stdout.name == 'CONOUT$':
			return
		
		# Try to allocate new console
		if ctypes.windll.kernel32.AllocConsole():
			try:
				sys.stdout = open("CONOUT$", "w", encoding="utf-8", buffering=1)
				sys.stderr = open("CONOUT$", "w", encoding="utf-8", buffering=1)
			except Exception:
				pass
	except Exception:
		pass


def _free_console() -> None:
	"""Free console if it was allocated by us."""
	try:
		# Only free if stdout/stderr are actually bound to CONOUT$
		if (hasattr(sys.stdout, 'name') and sys.stdout.name == 'CONOUT$') or \
		   (hasattr(sys.stderr, 'name') and sys.stderr.name == 'CONOUT$'):
			ctypes.windll.kernel32.FreeConsole()
	except Exception:
		pass


def _resource_path(*relative_parts: str) -> str:
	"""Return absolute path for resources both in dev and PyInstaller bundle."""
	try:
		if getattr(sys, "frozen", False) and hasattr(sys, "_MEIPASS"):
			base = Path(sys._MEIPASS)  # type: ignore
		else:
			base = Path(__file__).resolve().parents[1]
		return str(base.joinpath(*relative_parts))
	except Exception:
		return str(Path(*relative_parts))


class UvicornController:
	def __init__(self) -> None:
		self._server: Optional[uvicorn.Server] = None
		self._thread: Optional[threading.Thread] = None
		self._lock = threading.Lock()
		self.port = 8000
		self.use_logging = False
		self.use_access_log = False

	def is_running(self) -> bool:
		with self._lock:
			if self._server is not None and self._server.started:
				return True
		return self._is_port_in_use("127.0.0.1", self.port)

	def _is_port_in_use(self, host: str, port: int) -> bool:
		with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
			sock.settimeout(0.2)
			try:
				return sock.connect_ex((host, port)) == 0
			except Exception:
				return False

	def start(self) -> None:
		if fastapi_app is None:
			raise RuntimeError(f"FastAPI app import failed: {_import_error}")
		with self._lock:
			if self._server is not None and self._server.started:
				return
			# Avoid starting when port is already in use by another instance
			if self._is_port_in_use("127.0.0.1", self.port):
				raise RuntimeError(f"Port {self.port} already in use. Another instance may be running.")
			log_level = "info" if self.use_logging else "warning"
			config = uvicorn.Config(
				fastapi_app,
				host="127.0.0.1",
				port=self.port,
				log_level=log_level,
				log_config=None,
				access_log=self.use_access_log,
			)
			server = uvicorn.Server(config)
			self._server = server

			def _run() -> None:
				server.run()

			thread = threading.Thread(target=_run, daemon=True)
			self._thread = thread
			thread.start()

			for _ in range(50):
				if server.started or self._is_port_in_use("127.0.0.1", self.port):
					break
				time.sleep(0.1)

	def stop(self) -> None:
		with self._lock:
			if self._server is None:
				return
			self._server.should_exit = True
			self._server.force_exit = True
			self._server = None


class StartupRegistry:
	RUN_KEY = r"Software\Microsoft\Windows\CurrentVersion\Run"
	VALUE_NAME = "KeyQueueViewer"

	@staticmethod
	def exe_path() -> str:
		# If frozen, this is the exe path; otherwise current python/script path
		if getattr(sys, "frozen", False):
			return os.path.abspath(sys.executable)
		return os.path.abspath(sys.argv[0])

	@classmethod
	def is_enabled(cls) -> bool:
		try:
			with winreg.OpenKey(winreg.HKEY_CURRENT_USER, cls.RUN_KEY, 0, winreg.KEY_READ) as key:
				val, _ = winreg.QueryValueEx(key, cls.VALUE_NAME)
				return bool(val)
		except FileNotFoundError:
			return False
		except OSError:
			return False

	@classmethod
	def set_enabled(cls, enabled: bool) -> None:
		path = cls.exe_path()
		try:
			with winreg.OpenKey(winreg.HKEY_CURRENT_USER, cls.RUN_KEY, 0, winreg.KEY_SET_VALUE) as key:
				if enabled:
					winreg.SetValueEx(key, cls.VALUE_NAME, 0, winreg.REG_SZ, f'"{path}"')
				else:
					try:
						winreg.DeleteValue(key, cls.VALUE_NAME)
					except FileNotFoundError:
						pass
		except FileNotFoundError:
			# Create the Run key if missing when enabling
			if enabled:
				with winreg.CreateKey(winreg.HKEY_CURRENT_USER, cls.RUN_KEY) as key:
					winreg.SetValueEx(key, cls.VALUE_NAME, 0, winreg.REG_SZ, f'"{path}"')


class App(tk.Tk):
	def __init__(self) -> None:
		super().__init__()
		
		# Load configuration
		self.config = load_launcher_config()
		self.current_language = self.config.get("language", LanguageManager.DEFAULT_LANGUAGE)
		
		self.title(LanguageManager.get_text(self.current_language, "title"))
		self.geometry("520x480")
		self.resizable(False, False)
		# Set window/taskbar icon from bundled favicon.ico
		try:
			icon_path = _resource_path("web", "favicon.ico")
			if os.path.exists(icon_path):
				self.iconbitmap(icon_path)
		except Exception:
			pass

		self.controller = UvicornController()
		self._load_port()

		self.status_var = tk.StringVar(value="Stopped")
		self.startup_var = tk.BooleanVar(value=StartupRegistry.is_enabled())
		self.port_var = tk.IntVar(value=self.controller.port)
		self.console_var = tk.BooleanVar(value=False)
		self.language_var = tk.StringVar(value=LanguageManager.LANGUAGES[self.current_language])

		frm = ttk.Frame(self, padding=14)
		frm.pack(fill=tk.BOTH, expand=True)

		# Language and Port Settings
		settings_frame = ttk.LabelFrame(frm, text="Settings", padding=8)
		settings_frame.pack(fill=tk.X, pady=(0, 8))
		
		# Language selection
		lang_row = ttk.Frame(settings_frame)
		lang_row.pack(fill=tk.X, pady=(0, 8))
		ttk.Label(lang_row, text=LanguageManager.get_text(self.current_language, "language") + ":").pack(side=tk.LEFT)
		self.language_combo = ttk.Combobox(lang_row, textvariable=self.language_var, values=LanguageManager.get_language_names(), state="readonly", width=15)
		self.language_combo.pack(side=tk.LEFT, padx=(8, 0))
		self.language_combo.bind('<<ComboboxSelected>>', self._on_language_change)
		
		# Port configuration
		port_row = ttk.Frame(settings_frame)
		port_row.pack(fill=tk.X)
		ttk.Label(port_row, text=LanguageManager.get_text(self.current_language, "port") + ":").pack(side=tk.LEFT)
		self.port_entry = ttk.Entry(port_row, textvariable=self.port_var, width=10)
		self.port_entry.pack(side=tk.LEFT, padx=(8, 8))
		self.btn_save_port = ttk.Button(port_row, text=LanguageManager.get_text(self.current_language, "save_port"), command=self._on_save_port)
		self.btn_save_port.pack(side=tk.LEFT)

		row0 = ttk.Frame(frm)
		row0.pack(fill=tk.X, pady=(0, 8))
		self.lbl = ttk.Label(row0, text="Server status:")
		self.lbl.pack(side=tk.LEFT)
		self.status = ttk.Label(row0, textvariable=self.status_var, foreground="#ff4444")
		self.status.pack(side=tk.RIGHT)

		row1 = ttk.Frame(frm)
		row1.pack(fill=tk.X, pady=6)
		self.btn_start = ttk.Button(row1, text=LanguageManager.get_text(self.current_language, "start_server"), command=self._on_start)
		self.btn_start.pack(side=tk.LEFT)
		self.btn_stop = ttk.Button(row1, text=LanguageManager.get_text(self.current_language, "stop_server"), command=self._on_stop)
		self.btn_stop.pack(side=tk.LEFT, padx=8)

		row1b = ttk.Frame(frm)
		row1b.pack(fill=tk.X, pady=6)
		self.chk_console = ttk.Checkbutton(row1b, text=LanguageManager.get_text(self.current_language, "toggle_console"), variable=self.console_var, command=self._on_toggle_console)
		self.chk_console.pack(side=tk.LEFT)

		row2 = ttk.Frame(frm)
		row2.pack(fill=tk.X, pady=6)
		self.btn_control = ttk.Button(row2, text=LanguageManager.get_text(self.current_language, "web_control"), command=self._open_control)
		self.btn_control.pack(side=tk.LEFT)
		self.btn_overlay = ttk.Button(row2, text=LanguageManager.get_text(self.current_language, "overlay"), command=self._open_overlay)
		self.btn_overlay.pack(side=tk.LEFT, padx=8)

		row4 = ttk.Frame(frm)
		row4.pack(fill=tk.BOTH, expand=True, pady=(16,0))
		txt = LanguageManager.get_text(self.current_language, "tips_content")
		self.info = tk.Text(row4, height=10, wrap=tk.WORD)
		self.info.insert("1.0", txt)
		self.info.configure(state=tk.DISABLED)
		self.info.pack(fill=tk.BOTH, expand=True)

		self.after(500, self._poll_status)

	def _open_control(self) -> None:
		webbrowser.open(f"http://127.0.0.1:{self.controller.port}/control")

	def _open_overlay(self) -> None:
		webbrowser.open(f"http://127.0.0.1:{self.controller.port}/overlay")

	def _load_port(self) -> None:
		try:
			cfg = load_app_config() if load_app_config else None
			if cfg:
				self.controller.port = int(getattr(cfg, 'port', 8000))
			else:
				self.controller.port = 8000
		except Exception:
			self.controller.port = 8000

	def _poll_status(self) -> None:
		try:
			running = self.controller.is_running()
			if running:
				self.status_var.set(LanguageManager.get_text(self.current_language, "server_running"))
				self.status.config(foreground="#0a7")  # green (running)
			else:
				self.status_var.set(LanguageManager.get_text(self.current_language, "server_stopped"))
				self.status.config(foreground="#ff4444")  # red (stopped)
		except Exception:
			self.status_var.set("Unknown")
			self.status.config(foreground="#ff8800")  # orange (error)
		self.after(1000, self._poll_status)

	def _on_start(self) -> None:
		if not is_user_admin():
			if messagebox.askyesno(
				LanguageManager.get_text(self.current_language, "admin_required"),
				LanguageManager.get_text(self.current_language, "admin_message")
			):
				relaunch_as_admin()
				return
		try:
			# Always read latest configured port from config file
			cfg = load_app_config() if load_app_config else None
			self.controller.port = int(getattr(cfg, 'port', self.controller.port)) if cfg else self.controller.port
			self.controller.use_logging = bool(self.console_var.get())
			self.controller.use_access_log = bool(self.console_var.get())
			if self.console_var.get():
				try:
					_alloc_console()
					logging.basicConfig(level=logging.INFO)
				except Exception:
					pass
			self.controller.start()
		except RuntimeError as e:
			# If port is in use, open control on that port
			if "Port" in str(e) and "in use" in str(e):
				messagebox.showinfo(
					LanguageManager.get_text(self.current_language, "already_running"),
					LanguageManager.get_text(self.current_language, "port_in_use").format(port=self.controller.port)
				)
				self._open_control()
				return
		except Exception as e:
			messagebox.showerror(
				LanguageManager.get_text(self.current_language, "error"),
				LanguageManager.get_text(self.current_language, "start_failed").format(error=str(e))
			)

	def _on_stop(self) -> None:
		try:
			self.controller.stop()
			if self.console_var.get():
				try:
					_free_console()
				except Exception:
					pass
		except Exception as e:
			messagebox.showerror(
				LanguageManager.get_text(self.current_language, "error"),
				LanguageManager.get_text(self.current_language, "stop_failed").format(error=str(e))
			)

	def _on_toggle_startup(self) -> None:
		try:
			StartupRegistry.set_enabled(self.startup_var.get())
		except Exception as e:
			messagebox.showerror(
				LanguageManager.get_text(self.current_language, "error"),
				LanguageManager.get_text(self.current_language, "startup_failed").format(error=str(e))
			)
			self.startup_var.set(StartupRegistry.is_enabled())

	def _on_toggle_console(self) -> None:
		if self.controller.is_running():
			messagebox.showinfo(
				LanguageManager.get_text(self.current_language, "console"),
				LanguageManager.get_text(self.current_language, "console_visibility_info")
			)
			if self.console_var.get():
				try:
					_alloc_console()
				except Exception:
					messagebox.showwarning(
						LanguageManager.get_text(self.current_language, "console"),
						LanguageManager.get_text(self.current_language, "console_failed")
					)
					self.console_var.set(False)
			else:
				try:
					_free_console()
				except Exception:
					pass

	def _on_language_change(self, event: tk.Event) -> None:
		selected_language = self.language_combo.get()
		self.current_language = LanguageManager.get_language_code(selected_language)
		
		# Save language setting
		self.config["language"] = self.current_language
		save_launcher_config(self.config)
		
		# Update UI language
		self._update_ui_language()
		
		# Update window title
		self.title(LanguageManager.get_text(self.current_language, "title"))

	def _update_ui_language(self) -> None:
		"""Update all UI elements to use current language"""
		# Update button texts
		self.btn_start.config(text=LanguageManager.get_text(self.current_language, "start_server"))
		self.btn_stop.config(text=LanguageManager.get_text(self.current_language, "stop_server"))
		self.chk_console.config(text=LanguageManager.get_text(self.current_language, "toggle_console"))
		self.btn_control.config(text=LanguageManager.get_text(self.current_language, "web_control"))
		self.btn_overlay.config(text=LanguageManager.get_text(self.current_language, "overlay"))
		
		# Update labels
		self.lbl.config(text="Server status:")
		
		# Update settings frame
		settings_frame = self.language_combo.master.master
		settings_frame.config(text="Settings")
		
		# Update language label
		lang_label = self.language_combo.master.winfo_children()[0]
		lang_label.config(text=LanguageManager.get_text(self.current_language, "language") + ":")
		
		# Update port label
		port_label = self.port_entry.master.winfo_children()[0]
		port_label.config(text=LanguageManager.get_text(self.current_language, "port") + ":")
		
		# Update save port button
		self.btn_save_port.config(text=LanguageManager.get_text(self.current_language, "save_port"))
		
		# Update tips content
		self.info.configure(state=tk.NORMAL)
		self.info.delete("1.0", tk.END)
		txt = LanguageManager.get_text(self.current_language, "tips_content")
		self.info.insert("1.0", txt)
		self.info.configure(state=tk.DISABLED)

	def _on_save_port(self) -> None:
		try:
			port = int(self.port_entry.get())
			if 1000 <= port <= 65535:
				# Update controller port
				self.controller.port = port
				
				# Save to launcher config
				self.config["port"] = port
				save_launcher_config(self.config)
				
				# Save to server config if available
				if save_app_config:
					try:
						server_config = load_app_config() if load_app_config else None
						if server_config:
							server_config.port = port
							save_app_config(server_config)
					except Exception:
						pass
				
				messagebox.showinfo("Port Saved", LanguageManager.get_text(self.current_language, "port_saved"))
			else:
				messagebox.showwarning("Invalid Port", LanguageManager.get_text(self.current_language, "port_invalid"))
		except ValueError:
			messagebox.showwarning("Invalid Input", "Please enter a valid port number (1000-65535).")


def main() -> None:
	app = App()
	app.mainloop()


if __name__ == "__main__":
	main()