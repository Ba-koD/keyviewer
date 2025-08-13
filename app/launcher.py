import threading
import time
import webbrowser
import socket
import sys
import os
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
	try:
		ctypes.windll.kernel32.AllocConsole()
		# Rebind stdio to the new console
		try:
			sys.stdout = open("CONOUT$", "w", encoding="utf-8", buffering=1)
			sys.stderr = open("CONOUT$", "w", encoding="utf-8", buffering=1)
		except Exception:
			pass
	except Exception:
		pass


def _free_console() -> None:
	try:
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
		self.title("Key Queue Viewer")
		self.geometry("520x380")
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

		frm = ttk.Frame(self, padding=14)
		frm.pack(fill=tk.BOTH, expand=True)

		row0 = ttk.Frame(frm)
		row0.pack(fill=tk.X, pady=(0, 8))
		self.lbl = ttk.Label(row0, text="Server status:")
		self.lbl.pack(side=tk.LEFT)
		self.status = ttk.Label(row0, textvariable=self.status_var, foreground="#0a7")
		self.status.pack(side=tk.RIGHT)

		row1 = ttk.Frame(frm)
		row1.pack(fill=tk.X, pady=6)
		self.btn_start = ttk.Button(row1, text="Start Server", command=self._on_start)
		self.btn_start.pack(side=tk.LEFT)
		self.btn_stop = ttk.Button(row1, text="Stop Server", command=self._on_stop)
		self.btn_stop.pack(side=tk.LEFT, padx=8)

		row1b = ttk.Frame(frm)
		row1b.pack(fill=tk.X, pady=6)
		self.chk_console = ttk.Checkbutton(row1b, text="Show console (logs)", variable=self.console_var, command=self._on_toggle_console)
		self.chk_console.pack(side=tk.LEFT)

		row2 = ttk.Frame(frm)
		row2.pack(fill=tk.X, pady=6)
		self.btn_control = ttk.Button(row2, text="Open Control", command=self._open_control)
		self.btn_control.pack(side=tk.LEFT)
		self.btn_overlay = ttk.Button(row2, text="Open Overlay", command=self._open_overlay)
		self.btn_overlay.pack(side=tk.LEFT, padx=8)

		row4 = ttk.Frame(frm)
		row4.pack(fill=tk.BOTH, expand=True, pady=(16,0))
		txt = (
			"Tips:\n"
			"- Run as Administrator to capture global keystrokes.\n"
			"- Select target window at Control page.\n"
			"- Overlay is a Browser Source in OBS.\n"
			"- Port change requires server restart.\n"
			"- You can toggle console window to see live logs. Logs apply after (re)start."
		)
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
			self.status_var.set("Running" if running else "Stopped")
		except Exception:
			self.status_var.set("Unknown")
		self.after(1000, self._poll_status)

	def _on_start(self) -> None:
		if not is_user_admin():
			if messagebox.askyesno("Administrator", "전역 키 후킹을 위해 관리자 권한이 필요할 수 있습니다.\n관리자 권한으로 다시 실행할까요?"):
				relaunch_as_admin()
				return
		try:
			# Always read latest configured port from config file
			cfg = load_app_config() if load_app_config else None
			self.controller.port = int(getattr(cfg, 'port', self.controller.port)) if cfg else self.controller.port
			self.controller.use_logging = bool(self.console_var.get())
			self.controller.use_access_log = bool(self.console_var.get())
			if self.console_var.get():
				_alloc_console()
				logging.basicConfig(level=logging.INFO)
			self.controller.start()
		except RuntimeError as e:
			# If port is in use, open control on that port
			if "Port" in str(e) and "in use" in str(e):
				messagebox.showinfo("Already running", f"이미 포트 {self.controller.port}에서 실행 중인 인스턴스가 있습니다. 컨트롤 페이지를 엽니다.")
				self._open_control()
				return
		except Exception as e:
			messagebox.showerror("Error", f"Failed to start server:\n{e}")

	def _on_stop(self) -> None:
		try:
			self.controller.stop()
			if self.console_var.get():
				_free_console()
		except Exception as e:
			messagebox.showerror("Error", f"Failed to stop server:\n{e}")

	def _on_toggle_startup(self) -> None:
		try:
			StartupRegistry.set_enabled(self.startup_var.get())
		except Exception as e:
			messagebox.showerror("Error", f"Failed to update startup option:\n{e}")
			self.startup_var.set(StartupRegistry.is_enabled())

	def _on_toggle_console(self) -> None:
		if self.controller.is_running():
			messagebox.showinfo("Console", "콘솔 가시성은 즉시 반영되지만, 상세 로그는 서버 재시작 후 적용됩니다.")
			if self.console_var.get():
				_alloc_console()
			else:
				_free_console()


def main() -> None:
	app = App()
	app.mainloop()


if __name__ == "__main__":
	main()